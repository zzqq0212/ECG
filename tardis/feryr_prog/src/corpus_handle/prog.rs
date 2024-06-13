use super::{
    serialization::serialize,
    syscall::SyscallId,
    target::Target,
    ty::ResKind,
    value::{ResValue, ResValueId, Value, ValueKind},
    HashMap, HashSet, IN_SHM_SZ,
};
use failure::format_err;
use iota::iota;
use memmap2::{MmapMut, MmapOptions};
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, time::Duration};
use tokio::{runtime::Runtime, time};

/// Flag for controlling execution behavior.
pub type ExecFlags = u64;

iota! {
    const WAIT: u64 = iota;
        , READY
        , RUNNING
}

iota! {
    pub const FLAG_COLLECT_COVER : ExecFlags = 1 << (iota);       // collect coverage
    , FLAG_DEDUP_COVER                                 // deduplicate coverage in executor
    , FLAG_INJECT_FAULT                                // inject a fault in this execution (see ExecOpts)
    , FLAG_COLLECT_COMPS                               // collect KCOV comparisons
    , FLAG_THREADED                                    // use multiple threads to mitigate blocked syscalls
    , FLAG_COLLIDE                                     // collide syscalls to provoke data races
    , FLAG_ENABLE_COVERAGE_FILTER                      // setup and use bitmap to do coverage filter
}

/// Option for controlling execution behavior.
#[derive(Debug, Clone)]
pub struct ExecOpt {
    /// Options for this execution.
    pub flags: ExecFlags,
    /// Inject fault for 'fault_call'.
    pub fault_call: i32,
    /// Inject fault 'nth' for 'fault_call'
    pub fault_nth: i32,
}

impl Default for ExecOpt {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecOpt {
    pub const fn new() -> Self {
        Self {
            flags: FLAG_DEDUP_COVER | FLAG_THREADED | FLAG_COLLIDE,
            fault_call: 0,
            fault_nth: 0,
        }
    }

    #[inline]
    pub fn disable(&mut self, flag: u64) {
        self.flags &= u64::MAX ^ flag;
    }

    #[inline]
    pub fn enable(&mut self, flag: u64) {
        self.flags |= flag;
    }
}

/// Flag for execution result of one call.
pub type CallFlags = u32;

iota! {
    pub const CALL_EXECUTED : CallFlags = 1 << (iota); // started at all
    , CALL_FINISHED                                // finished executing (rather than blocked forever)
    , CALL_BLOCKED                                 // finished but blocked during execution
    , CALL_FAULT_INJECTED                          // fault was injected into this call
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
    sid: SyscallId,
    args: Box<[Value]>,
    pub ret: Option<Value>,
    #[serde(with = "vectorize")]
    pub generated_res: HashMap<ResKind, Vec<ResValueId>>,
    #[serde(with = "vectorize")]
    pub used_res: HashMap<ResKind, Vec<ResValueId>>,
}

impl Call {
    #[inline(always)]
    pub fn sid(&self) -> SyscallId {
        self.sid
    }

    #[inline(always)]
    pub fn args(&self) -> &[Value] {
        &self.args
    }

    #[inline(always)]
    pub fn args_mut(&mut self) -> &mut [Value] {
        &mut self.args
    }

    #[inline(always)]
    pub fn display<'a, 'b>(&'a self, target: &'b Target) -> CallDisplay<'a, 'b> {
        CallDisplay { call: self, target }
    }
}

pub struct CallDisplay<'a, 'b> {
    call: &'a Call,
    target: &'b Target,
}

impl<'a, 'b> std::fmt::Display for CallDisplay<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let syscall = self.target.syscall_of(self.call.sid);
        if let Some(ret) = self.call.ret.as_ref() {
            let ret = ret.checked_as_res();
            write!(f, "r{} = ", ret.res_val_id().unwrap())?;
        }
        write!(f, "{}(", syscall.name())?;
        for (i, arg) in self.call.args.iter().enumerate() {
            write!(f, "{}", arg.display(self.target))?;
            if i != self.call.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone)]
pub struct CallBuilder {
    pub(crate) sid: SyscallId,
    pub(crate) args: Vec<Value>,
    pub(crate) ret: Option<Value>,
    pub(crate) generated_res: HashMap<ResKind, HashSet<ResValueId>>,
    pub(crate) used_res: HashMap<ResKind, HashSet<ResValueId>>,
}

impl CallBuilder {
    pub fn new(sid: SyscallId) -> Self {
        Self {
            sid,
            args: Vec::new(),
            ret: None,
            generated_res: HashMap::default(),
            used_res: HashMap::default(),
        }
    }

    pub fn arg(&mut self, arg: Value) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub fn args<T: IntoIterator<Item = Value>>(&mut self, args: T) -> &mut Self {
        self.args.extend(args);
        self
    }

    pub fn ret(&mut self, ret: Option<Value>) -> &mut Self {
        self.ret = ret;
        self
    }

    pub fn record_res(&mut self, res: &ResKind, id: ResValueId) -> &mut Self {
        if !self.generated_res.contains_key(res) {
            self.generated_res.insert(res.clone(), HashSet::new());
        }
        self.generated_res.get_mut(res).unwrap().insert(id);
        self
    }

    pub fn used_res(&mut self, res: &ResKind, id: ResValueId) -> &mut Self {
        if !self.used_res.contains_key(res) {
            self.used_res.insert(res.clone(), HashSet::new());
        }
        self.used_res.get_mut(res).unwrap().insert(id);
        self
    }

    pub fn build(mut self) -> Call {
        self.args.shrink_to_fit();
        let mut generated_res: HashMap<ResKind, Vec<ResValueId>> = HashMap::default();
        for (kind, ids) in self.generated_res {
            generated_res.insert(kind, ids.into_iter().collect());
        }
        let mut used_res: HashMap<ResKind, Vec<ResValueId>> = HashMap::default();
        for (kind, ids) in self.used_res {
            used_res.insert(kind, ids.into_iter().collect());
        }
        generated_res.shrink_to_fit();
        used_res.shrink_to_fit();

        Call {
            sid: self.sid,
            args: self.args.into_boxed_slice(),
            ret: self.ret,
            generated_res,
            used_res,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Prog {
    pub(crate) calls: Vec<Call>,
}

impl Prog {
    #[inline(always)]
    pub fn new(calls: Vec<Call>) -> Self {
        Self { calls }
    }

    #[inline(always)]
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    #[inline(always)]
    pub fn calls_mut(&mut self) -> &mut [Call] {
        &mut self.calls
    }

    #[inline(always)]
    pub fn display<'a, 'b>(&'a self, target: &'b Target) -> ProgDisplay<'a, 'b> {
        ProgDisplay { prog: self, target }
    }

    pub fn remove_call(&self, i: usize) -> Prog {
        let mut new_p = self.clone();
        new_p.remove_call_inplace(i);
        new_p
    }

    pub fn remove_call_inplace(&mut self, i: usize) {
        if i == self.calls.len() - 1 {
            self.calls.pop();
            return;
        }

        let removed_call = self.calls.remove(i);
        let removed_res = removed_call.generated_res;
        for c in &mut self.calls[i..] {
            let mut ids_to_remove = HashSet::new();
            for (removed_res_kind, removed_ids) in removed_res.iter() {
                if let Some(used_ids) = c.used_res.get(removed_res_kind) {
                    for id in used_ids {
                        if removed_ids.contains(id) {
                            ids_to_remove.insert(*id);
                        }
                    }
                }
            }
            if !ids_to_remove.is_empty() {
                c.args
                    .iter_mut()
                    .for_each(|arg| Self::set_res_val_to_null(arg, &ids_to_remove));
            }
        }
    }

    fn set_res_val_to_null(val: &mut Value, ids: &HashSet<ResValueId>) {
        match val.kind() {
            ValueKind::Res => {
                let val = val.checked_as_res_mut();
                if let Some(id) = val.res_val_id() {
                    if ids.contains(&id) {
                        assert!(val.ref_res());
                        *val = ResValue::new_null(val.ty_id(), val.dir(), 0);
                    }
                }
            }
            ValueKind::Ptr => {
                let val = val.checked_as_ptr_mut();
                if let Some(pointee) = val.pointee.as_mut() {
                    Self::set_res_val_to_null(pointee, ids);
                }
            }
            ValueKind::Group => {
                let vals = val.checked_as_group_mut();
                for val in vals.inner.iter_mut() {
                    Self::set_res_val_to_null(val, ids);
                }
            }
            ValueKind::Union => {
                let val = val.checked_as_union_mut();
                Self::set_res_val_to_null(&mut val.option, ids);
            }
            _ => (),
        }
    }

    pub fn exec_input_prog(&self,
                           target: &Target,
                           qemu_mem: &Vec<(String, String)>,
                           tid: &usize) 
                           -> Result<bool, failure::Error> {

        // where the share memory are 
        let file_name = "/dev/shm/feryr-".to_string() + &tid.to_string();
        let file = OpenOptions::new().read(true).write(true).open(file_name);
        let file = match file {
            Ok(file) => file,
            Err(_) => return Err(format_err!("create file err!")),
        };

        // write qemu mem to file
        let mut corpus_shm = unsafe { MmapOptions::new().map_mut(&file)? };
        let call_len = self.calls.len();
        let mut call_idx = 0;
        let status_addr = usize::from_str_radix(&qemu_mem[3].1[1..], 16).unwrap();

        // TODO: here should add data transfer for return value, we now only consider syscall without return value
        // let _data_addr = usize::from_str_radix(&qemu_mem[1].1[1..], 16).unwrap();
        let start_index = usize::from_str_radix(&qemu_mem[0].1[1..], 16).unwrap();

        while call_idx <= call_len {
            let s1 = self.calls().get(call_idx).unwrap();
            let s2 = target.syscall_of(s1.sid());

            let func_name = s2.call_name();
            if func_name == "OSTaskCreate" ||
               func_name == "Str_ParseNbr_Int32U"||
               func_name == "Str_ParseNbr_Int32S" {
                
                return Err(format_err!("create file err!"));
            }
            
            // serialize single call
            let mut buf = [0; IN_SHM_SZ];
            serialize(&target, &self, &mut buf, call_idx).unwrap();

            // ready-wait status check detection
            // using asynchronous lock to wait for 5 seconds, if timeout -> there is a bug 
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let res = time::timeout(time::Duration::from_millis(5000), async {
                    wait_exec(&corpus_shm, status_addr.to_owned()).await
                });
                match res.await {
                    Err(_) => Err(format_err!("timeout!")),
                    Ok(_) => Ok(()),
                }
            })?;

            // write test case to guest vm
            let executor_status = &corpus_shm[status_addr..status_addr + 2].to_owned().clone();
            // check if is wait status
            if executor_status[0] == 'w' as u8 && executor_status[1] == 'a' as u8 {
                // rewrite to ready status or repre status 
                let one_buf = if call_idx == (call_len - 1) {
                    "cl".as_bytes()
                } else {
                    "re".as_bytes()
                };
                // write into shm and wait for execution
                corpus_shm[status_addr..status_addr + one_buf.len()]
                    .copy_from_slice(&one_buf.to_vec());
                corpus_shm[start_index..start_index + IN_SHM_SZ].copy_from_slice(&buf[0..]);
                call_idx += 1;
            }

            if call_idx == call_len {
                break;
            }
        }

        Ok(false)
    }

    pub fn stop_soon() -> bool {
        return true;
    }
}

async fn wait_exec(corpus_shm: &MmapMut, status_addr: usize) -> () {
    loop {
        let check_status = &corpus_shm[status_addr..status_addr + 2].to_owned().clone();
        if check_status[0] == 'w' as u8 && check_status[1] == 'a' as u8 {
            break;
        } else {
            time::sleep(Duration::from_millis(10)).await;
        }
    }
}

#[derive(Debug)]
pub struct ProgDisplay<'a, 'b> {
    pub prog: &'a Prog,
    target: &'b Target,
}

impl<'a, 'b> std::fmt::Display for ProgDisplay<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, call) in self.prog.calls.iter().enumerate() {
            // if verbose_mode() {
            //     write!(f, "({}) ", _idx)?;
            // }
            writeln!(f, "{}: {}", idx, call.display(self.target))?;
        }
        Ok(())
    }
}
