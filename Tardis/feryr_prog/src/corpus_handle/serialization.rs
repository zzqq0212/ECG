use super::{
    prog::{Call, Prog},
    target::Target,
    value::{PtrValue, ResValueKind, Value, ValueKind},
    // ty::{BinaryFormat, Dir, TypeKind},
    // value::{PtrValue, ResValueId, ResValueKind, Value, ValueKind, VmaValue},
    // HashMap,
};
use lazy_static::lazy_static;
use std::{collections::HashMap as otherHash, mem};

use crate::corpus_handle::{syscall::Syscall};
use bytes::BufMut;
use bytes::BytesMut;
use iota::iota;

lazy_static! {
    static ref UCOSHASHMAP: otherHash<String, u32> = {
        let mut ucos_struct_map = otherHash::new();
        ucos_struct_map.insert("MEM_DYN_POOL".to_string(), 0);
        ucos_struct_map.insert("OS_TMR".to_string(), 1);
        ucos_struct_map.insert("OS_MEM".to_string(), 2);
        ucos_struct_map.insert("MEM_POOL".to_string(), 3);
        ucos_struct_map.insert("OS_TCB".to_string(), 4);
        ucos_struct_map.insert("OS_FLAG_GRP".to_string(), 5);
        ucos_struct_map.insert("OS_MUTEX".to_string(), 6);
        ucos_struct_map.insert("OS_SEM".to_string(), 7);
        ucos_struct_map.insert("OS_Q".to_string(), 8);
        ucos_struct_map.insert("MEM_SEG".to_string(), 9);
        ucos_struct_map.insert("NET_IF".to_string(), 10);
        ucos_struct_map
    };
    static ref FREERTOSHASHMAP: otherHash<String, u32> = {
        let mut rtthread_struct_map = otherHash::new();
        rtthread_struct_map.insert("rt_timer".to_string(), 0);
        rtthread_struct_map
    };
    static ref RTTHREADHASHMAP: otherHash<String, u32> = {
        let mut rtthread_struct_map = otherHash::new();
        rtthread_struct_map.insert("rt_timer".to_string(), 0);
        rtthread_struct_map.insert("rt_mempool".to_string(), 1);
        rtthread_struct_map.insert("dfs_fd".to_string(), 2);
        rtthread_struct_map.insert("stat".to_string(), 3);
        rtthread_struct_map.insert("statfs".to_string(), 4);
        rtthread_struct_map.insert("dfs_filesystem_ops".to_string(), 5);
        rtthread_struct_map.insert("rt_device".to_string(), 6);
        rtthread_struct_map
    };
    static ref ZEPHYRHASHMAP: otherHash<String, u32> = {
        let mut rtthread_struct_map = otherHash::new();
        rtthread_struct_map.insert("rt_timer".to_string(), 0);
        rtthread_struct_map
    };
}

iota! {
    const EXEC_INSTR_EOF : u64 = (u64::MAX) ^ (iota);
        , EXEC_INSTR_COPY_IN
        , EXEC_INSTR_COPY_OUT
}

iota! {
    const EXEC_ARG_CONST: u64 = iota;
        , EXEC_ARG_RESULT
        , EXEC_ARG_DATA
        , EXEC_ARG_CSUM

    const EXEC_ARG_DATA_READABLE : u64 = 1<<63;
}

iota! {
    const EXEC_ARG_CSUM_INET: u64 = iota;
}

iota! {
    const EXEC_ARG_CSUM_CHUNK_DATA: u64 = iota;
        , EXEC_ARG_CSYM_CHUNK_CONST
}

iota! {
    const INT_TYPE: u64 = iota;     // int args
    , RES_TYPE_IN                   // copy_in resource
    , RES_TYPE_OUT                  // copy_out resource
    , PTR_TYPE                  // int pointer
    , STRC_TYPE                  // struct
    , DATA_TYPE                     // struct && array

    const GROUP_TYPE: u64 = 8;

}

// const EXEC_NO_COPYOUT: u64 = u64::MAX;
const EXEC_MAX_COMMANDS: u64 = 1000;
const MAX_BUFFER_SIZE: u64 = 512;
const MAX_ARRAY_LEN: u64 = 8;
#[derive(Debug)]
pub struct SyscallStub<'slice> {
    // syscall id
    sid: u64,
    // args value (int && ptr only)
    args_val: Vec<u64>,
    // args type
    args_typ: Vec<u64>, // int, intptr, ...
    // args data buffer (array && simple struct)
    // args_buf: &'slice mut [u8], //, MAX_BUFFER_SIZE],
    args_buf: &'slice mut BytesMut,
    // return value
    pub ret: u64,
}

#[derive(Debug)]
pub enum SerializeError {
    // #[error("buffer two small to serialize the prog, provided size: {provided} bytes")]
    BufferTooSmall { provided: usize },
}

/// Serialize a prog into packed binary format.
pub fn serialize(target: &Target,
                 p: &Prog,
                 buf: &mut [u8],
                 call_idx: usize,) -> Result<(usize, usize), SerializeError> {
    // let ptr = buf.as_ptr();
    let mut ctx = ExecCtx {target,
                                            buf,
                                            // res_args: HashMap::default(),
                                            copyout_seq: 0,
                                            eof: false};

    let (call_buf, err_code_addr) = ctx
                                                  .serialize_call(p.
                                                                  calls().
                                                                  get(call_idx).
                                                                  unwrap())
                                                  .unwrap();
    ctx.write_slice(&call_buf);
    ctx.write_u64(EXEC_INSTR_EOF);

    if !ctx.buf.has_remaining_mut() || 
                            ctx.eof || 
                            ctx.copyout_seq > EXEC_MAX_COMMANDS {
        Err(SerializeError::BufferTooSmall {
            provided: buf.len(),
        })
    } else {
        Ok((ctx.buf.remaining_mut(), err_code_addr as usize))
    }
}

struct ExecCtx<'a, 'b> {
    target: &'a Target,
    buf: &'b mut [u8],
    // res_args: HashMap<ResValueId, ArgInfo>,
    copyout_seq: u64,
    eof: bool,
}

impl ExecCtx<'_, '_> {
    fn serialize_call(&mut self, c: &Call) -> Result<(Vec<u8>, u64), failure::Error> {
        // get current call
        let current_call = self.target.syscall_of(c.sid());
       
        // use syscallstub to store the syscall to be executed
        let mut call_stub = SyscallStub {
            sid: 0,
            args_val: Default::default(),
            args_typ: Default::default(),
            // args_buf: &mut [0; MAX_BUFFER_SIZE as usize],
            args_buf: &mut BytesMut::with_capacity(MAX_BUFFER_SIZE as usize),
            ret: 0xffffffff,
        };
        let mut data_index: u64 = 0;
        let mut err_code_addr: u64 = 0;
        
        call_stub.sid = c.sid() as u64;
        
        let name = current_call.call_name();
        if name == "Str_ParseNbr_Int32U" ||
           name == "Str_ParseNbr_Int32S" {
            println!("{:?}", c.args());
            println!("{}", name);
            println!("test...");
        }

        for (idx, val) in c.args().iter().enumerate() {
            if idx == c.args().len() - 1 {
                err_code_addr = data_index;
            }
            // here we stub each argument within the system call 
            self.stub_args(val, &mut call_stub, current_call, &mut data_index);
        }
        let ret = c.ret.clone();
        if ret.is_some() {
            // call_stub.ret = ret.unwrap().kind()
            match &ret.as_ref().unwrap().kind() {
                ValueKind::Res => {
                    call_stub.ret = ret.unwrap().as_res().unwrap().res_val_id().unwrap();
                }
                _ => {
                    call_stub.ret = 0xffffffff;
                }
            }
        }
        // dbg!(&call_stub);
        let call_buf = self.struct_to_u8(&mut call_stub).unwrap();
        Ok((call_buf, err_code_addr))
    }

    // convert syscallstub into buffer
    fn struct_to_u8(&self, call_stub: &mut SyscallStub) -> Result<Vec<u8>, failure::Error> {
        let mut call_buf = BytesMut::with_capacity(MAX_BUFFER_SIZE as usize);

        call_buf.put_u64_le(call_stub.sid);

        call_buf.put_u64_le(call_stub.args_val.len() as u64);
        for val in &call_stub.args_val {
            call_buf.put_u64_le(*val);
        }

        call_buf.put_u64_le(call_stub.args_typ.len() as u64);
        for val in &call_stub.args_typ {
            call_buf.put_u64_le(*val);
        }

        call_buf.put_u64_le(call_stub.ret);

        call_buf.put_u64_le(call_stub.args_buf.len() as u64); // MAX_BUFFER_SIZE
                                                              // println!("raw string = {:?}", &call_stub.args_buf);

        let buf_size = call_stub.args_buf.len();
        let short_bits = 8 - buf_size % 8;
        let mut reduent = vec![0; short_bits];
        let mut reversed: Vec<u8> = vec![];

        call_stub.args_buf.put_slice(&mut reduent.as_mut_slice());
        for cur_data in call_stub.args_buf.chunks(8) {
            reversed.truncate(0);
            reversed.extend(cur_data.iter().rev());
            call_buf.put_slice(&reversed);
        }
        // dbg!(call_buf.to_vec().len().unwrap());

        Ok(call_buf.to_vec())
    }

    fn checked_as_structs(&self, current_call: &Syscall) -> bool {
        if current_call.struct_pos.len() != 0 {
            true
        } else {
            false
        }
    }

    fn stub_int(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        // TODO: current call here need fix, thought I forget why
        _current_call: &Syscall,
        flags: u64,
    ) {
        let val = val.checked_as_int();
        let int_val = val.value(self.target).0;
        call_stub.args_typ.push(flags);
        call_stub.args_val.push(int_val);
    }

    fn stub_res(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {
        // println!("res val = {:?}", &val);
        let val = val.checked_as_res();
        if flags == RES_TYPE_IN ^ RES_TYPE_OUT {
            let mut kid = 0;
            match val.res_val_id() {
                Some(id) => kid = id,
                None => {}
            }
            match &val.kind {
                ResValueKind::Own(_) | ResValueKind::Null => {
                    call_stub.args_typ.push(flags ^ RES_TYPE_IN);
                    // because of bug of syscall impl, remove temply
                    call_stub.args_val.push(kid);
                }
                ResValueKind::Ref(_idx) => {
                    call_stub.args_typ.push(flags ^ RES_TYPE_OUT);
                    // because of bug of syscall impl, remove temply
                    call_stub.args_val.push(kid);
                }
            }
        } else if flags == STRC_TYPE ^ PTR_TYPE {
            // if it is a struct type
            call_stub.args_typ.push(STRC_TYPE | PTR_TYPE);
            let struct_name = current_call.struct_pos[0].0.to_owned();
            let struct_map = match self.target.os() {
                "ucos" => UCOSHASHMAP.get(&struct_name).unwrap(),
                "freertos" => FREERTOSHASHMAP.get(&struct_name).unwrap(),
                "rtthread" => RTTHREADHASHMAP.get(&struct_name).unwrap(),
                "zephyr" => ZEPHYRHASHMAP.get(&struct_name).unwrap(),
                _ => {
                    panic!("unsupported os type");
                }
            };
            let idx = ((*struct_map) as u64) * MAX_ARRAY_LEN;
            match val.res_val_id() {
                Some(id) => call_stub
                    .args_val
                    .push(idx.saturating_add(id % MAX_ARRAY_LEN)),
                None => {
                    // println!("failed to identify the resource type!");
                    // println!("current call = {:?}\n, val = {:?}", &current_call, &val);
                    call_stub.args_val.push(idx.saturating_add(0));
                }
            }
        } else if flags == INT_TYPE ^ PTR_TYPE {
            let res_val = val.val;
            call_stub
                .args_buf
                // .as_mut()
                .put_slice(&res_val.to_be_bytes());
            call_stub.args_val.push(*data_index);
            *data_index += mem::size_of::<u64>() as u64;
        }
    }

    fn stub_data(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        _current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {
        let val = val.checked_as_data();
        let bs = &val.data;
        
        call_stub.args_typ.push(flags);
        call_stub.args_val.push(*data_index);
        call_stub.args_val.push(bs.len() as u64);

        *data_index += if bs.len() == 0 { 1 } else { bs.len() } as u64;
        if bs.is_empty() {
            return;
        }
        let pad = 8 - bs.len() % 8;
        call_stub.args_buf.put_slice(bs.as_ref());
        if pad != 8 {
            static PAD: [u8; 8] = [0; 8];
            call_stub.args_buf.put_slice(&PAD[0..pad]);
        }

    }

    fn stub_group(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        _current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {
        // dbg!("Error...");
        let val = val.checked_as_group();
        call_stub.args_typ.push(flags);
        call_stub.args_val.push(*data_index);

        let data = &val.get_val();
        let size = if data.len() % 64 == 0 {
            data.len() / 2
        } else {
            data.len() / 2 + 1
        };
        let byte_size = size * mem::size_of::<u64>();
        *data_index += byte_size as u64;
        call_stub.args_buf.put_slice(&vec![0u8; byte_size]);
        call_stub.args_val.push(byte_size as u64);
    }

    // deal with int stub
    fn try_int_stub(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall, 
        data_index: &mut u64,
        flags: u64,
    ) {
        // dbg!(flags);
        if flags == INT_TYPE {
            self.stub_int(val, call_stub, current_call, flags);
        } else if flags == PTR_TYPE {
            let int_val = val.checked_as_int().value(self.target).0;
            let byte_size = 1 * mem::size_of::<u64>();
            call_stub.args_typ.push(flags);
            call_stub.args_buf.put_u64(int_val);
            call_stub.args_val.push(*data_index);
            call_stub.args_val.push(byte_size as u64);
            *data_index += byte_size as u64;
        } else {
            println!("can't resolve type {:?}", val.kind());
            unreachable!()
        }
    }

    // deal with res stub
    fn try_res_stub(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {
        self.stub_res(val, call_stub, current_call, data_index, flags);
    }

    // deal with data stub
    fn try_data_stub(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) { 
        self.stub_data(val, call_stub, current_call, data_index, flags);        
    }

    // deal with ptr stub
    fn try_ptr_stub(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {
        let val = val.checked_as_ptr();
        let a = val.pointee.to_owned();
        //  get the start point and the offset of the args_buffer
        if a.is_none() { 
            call_stub.args_typ.push(flags);
            call_stub.args_val.push(0);
            return;
        }
        let a = a.unwrap();
        if a.kind() == ValueKind::Ptr {
            self.try_ptr_stub(&a, call_stub, current_call, data_index, flags);
        }
        self.ptr_physical_addr(&val, call_stub, current_call, data_index, flags);
    }

    // deal with group stub
    // note group stub may contain many recursive struct, eg a group contains many int
    fn try_group_stub(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) { 
        let val = val.checked_as_group(); 

        for item in val.inner.iter() { 
            let a = item.to_owned();
            let val: Value = item.to_owned();
            match a.kind() {
                ValueKind::Group => {
                    self.try_group_stub(
                        &val,
                        call_stub,
                        current_call,
                        data_index,
                        flags,
                    );  
                }
                ValueKind::Ptr => {
                    self.try_ptr_stub(
                        &val,
                        call_stub,
                        current_call,
                        data_index,
                        flags,
                    );
                }
                _ => {
                    self.stub_group(&val, 
                                    call_stub, 
                                    current_call, 
                                    data_index, 
                                    flags);
                }
            }
        }
        // self.stub_group(val, call_stub, current_call, data_index, flags);
    }

    // stub each argument within system call a string buffer
    fn stub_args(
        &mut self,
        val: &Value,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
    ) {
        // dbg!(&val);
        match val.kind() {
            ValueKind::Integer => {
                // dbg!("is int");
                // dbg!(&val);
                self.try_int_stub(&val, call_stub, current_call, data_index, INT_TYPE);
            }
            ValueKind::Res => {
                // dbg!("is res");
                // dbg!(&val);
                self.try_res_stub(
                    &val,
                    call_stub,
                    current_call,
                    data_index,
                    RES_TYPE_IN ^ RES_TYPE_OUT,
                );
            }
            // TODO: here we still have some problem, especially deal with recursive data structure
            ValueKind::Ptr => {
                // dbg!("is ptr");
                // dbg!(&val);
                self.try_ptr_stub(&val, 
                                  call_stub, 
                                  current_call, 
                                  data_index, 
                                  PTR_TYPE);
            }
            ValueKind::Data => {
                // dbg!("is Data");
                // dbg!(&val);
                self.try_data_stub(&val, call_stub, current_call, data_index, DATA_TYPE);
            }
            ValueKind::Group => {
                // dbg!("is Group");
                // dbg!(&val);
                self.try_group_stub(&val, call_stub, current_call, data_index, GROUP_TYPE);
            }
            _ => {
                println!("can't resolve type {:?}", val.kind());
                unreachable!()
            }
        }
    }

    // write a signle u64 into buffer
    fn write_u64(&mut self, val: u64) {
        self.buf.put_u64_le(val);
    }

    // write a entire array into buffer
    fn write_slice<T: AsRef<[u8]>>(&mut self, slice: T) {
        if self.buf.len() >= slice.as_ref().len() {
            self.buf.put_slice(slice.as_ref());
        } else {
            self.eof = true;
        }
    }
    
    // convert pointer type address from host address to guest address
    fn ptr_physical_addr(
        &mut self,
        val: &PtrValue,
        call_stub: &mut SyscallStub,
        current_call: &Syscall,
        data_index: &mut u64,
        flags: u64,
    ) {

        let a = val.pointee.to_owned().unwrap();

        match a.kind() {
            ValueKind::Integer => {
                let val = val.pointee.as_ref().unwrap();
                self.try_int_stub(
                    &val,
                    call_stub,
                    current_call,
                    data_index,
                    flags,
                );
            }
            ValueKind::Data => {
                let val = val.pointee.as_ref().unwrap();
                self.try_data_stub(
                    &val,
                    call_stub,
                    current_call,
                    data_index,
                    flags,
                );
            }
            ValueKind::Group => {
                let val = val.pointee.as_ref().unwrap();
                self.try_group_stub(
                    &val,
                    call_stub,
                    current_call,
                    data_index,
                    flags,
                );
            }
            ValueKind::Res => {
                let val = val.pointee.as_ref().unwrap();
                if self.checked_as_structs(current_call) {
                    self.try_res_stub(
                        &val,
                        call_stub,
                        current_call,
                        data_index,
                        flags,
                    );
                } else {
                    self.try_res_stub(
                        &val,
                        call_stub,
                        current_call,
                        data_index,
                        flags,
                    );
                }
            } 
            _ => {
                println!("{}", current_call.call_name());
                println!("can't resolve type {:?}", a.kind());
                unreachable!()
            }
        }
    }

    /*
       fn write_copyin(&mut self, c: &Call) {
           foreach_call_args(self.target, c, |val, ctx| {
               if ctx.base.is_none() {
                   return;
               }
               let base = ctx.base.unwrap();
               let mut val_addr = base + ctx.offset;
               let ty = val.ty(ctx.target);
               val_addr -= ty.bitfield_unit_off();

               if let Some(res_val) = val.as_res() {
                   if let ResValueKind::Own(id) = &res_val.kind {
                       self.res_args.insert(*id, ArgInfo::with_addr(val_addr));
                   }
               }

               if let ValueKind::Group | ValueKind::Union = &val.kind() {
                   return;
               }

               let ty = val.ty(ctx.target);
               if val.dir() == Dir::Out
                   || (ty.as_const().is_some() && ty.checked_as_const().pad())
                   || (val.size(ctx.target) == 0 && !ty.is_bitfield())
               {
                   return;
               }

               self.write_u64(EXEC_INSTR_COPY_IN);
               self.write_u64(val_addr);
               self.write_arg(val);
           })
       }

       fn write_copyout(&mut self, c: &Call) {
           foreach_call_args(self.target, c, |val, _| {
               if val.as_res().is_none() {
                   return;
               }
               let val = val.checked_as_res();
               if val.own_res() {
                   let id = val.res_val_id().unwrap();
                   let info = self.res_args.get_mut(&id).unwrap();
                   if info.ret {
                       return;
                   }
                   let new_idx = self.copyout_seq;
                   self.copyout_seq += 1;
                   info.idx = new_idx;
                   let addr = info.addr;

                   self.write_u64(EXEC_INSTR_COPY_OUT);
                   self.write_u64(new_idx);
                   self.write_u64(addr);
                   self.write_u64(val.size(self.target));
               }
           })
       }

       fn write_arg(&mut self, val: &Value) {
           match val.kind() {
               ValueKind::Integer => {
                   let val = val.checked_as_int();
                   let (int_val, pid_stride) = val.value(self.target);
                   let ty = val.ty(self.target);
                   self.write_const_arg(
                       ty.bitfield_unit(),
                       int_val,
                       ty.bitfield_off(),
                       ty.bitfield_len(),
                       pid_stride,
                       ty.format(),
                   );
               }
               ValueKind::Res => {
                   let val = val.checked_as_res();
                   match &val.kind {
                       ResValueKind::Own(_) | ResValueKind::Null => {
                           let ty = val.ty(self.target);
                           self.write_const_arg(val.size(self.target), val.val, 0, 0, 0, ty.format());
                       }
                       ResValueKind::Ref(idx) => {
                           if let Some(info) = self.res_args.get(idx).cloned() {
                               self.write_u64(EXEC_ARG_RESULT);
                               let idx = info.idx;
                               let ty = val.ty(self.target);
                               let meta = val.size(self.target) | (ty.format() as u64) << 8;
                               self.write_u64(meta);
                               self.write_u64(idx);
                               self.write_u64(val.op_div);
                               self.write_u64(val.op_add);
                               self.write_u64(val.val);
                           } else {
                               let ty = val.ty(self.target);
                               self.write_const_arg(
                                   val.size(self.target),
                                   val.val,
                                   0,
                                   0,
                                   0,
                                   ty.format(),
                               );
                           }
                       }
                   }
               }
               ValueKind::Ptr => {
                   let val = val.checked_as_ptr();
                   let ty = val.ty(self.target);
                   self.write_const_arg(
                       val.size(self.target),
                       ptr_physical_addr(self.target, val),
                       0,
                       0,
                       0,
                       ty.format(),
                   );
               }
               ValueKind::Vma => {
                   let val = val.checked_as_vma();
                   let ty = val.ty(self.target);
                   self.write_const_arg(
                       val.size(self.target),
                       vma_physical_addr(self.target, val),
                       0,
                       0,
                       0,
                       ty.format(),
                   );
               }
               ValueKind::Data => {
                   let val = val.checked_as_data();
                   let ty = val.ty(self.target);
                   let bs = &val.data;

                   if bs.is_empty() {
                       return;
                   }
                   self.write_u64(EXEC_ARG_DATA);
                   let mut flags = bs.len() as u64;
                   if matches!(ty.kind(), TypeKind::BufferFilename | TypeKind::BufferString) {
                       flags |= EXEC_ARG_DATA_READABLE;
                   }
                   self.write_u64(flags);
                   let pad = 8 - bs.len() % 8;
                   self.write_slice(bs);
                   if pad != 8 {
                       static PAD: [u8; 8] = [0; 8];
                       self.write_slice(&PAD[0..pad]);
                   }
               }
               ValueKind::Union => {
                   let val = val.checked_as_union();
                   self.write_arg(&val.option)
               }
               _ => unreachable!(),
           }
       }

       fn write_const_arg(
           &mut self,
           sz: u64,
           val: u64,
           bf_offset: u64,
           bf_len: u64,
           pid_stride: u64,
           bf: BinaryFormat,
       ) {
           self.write_u64(EXEC_ARG_CONST);
           let meta = sz | (bf as u64) << 8 | bf_offset << 16 | bf_len << 24 | pid_stride << 32;
           self.write_u64(meta);
           self.write_u64(val);
       }
    */
}
/*
#[derive(Clone)]
struct ArgCtx<'a> {
    target: &'a Target,
    base: Option<u64>,
    offset: u64,
}

fn foreach_call_args(target: &Target, call: &Call, mut f: impl FnMut(&Value, &ArgCtx)) {
    let mut ctx = ArgCtx {
        target,
        base: None,
        offset: 0,
    };
    if let Some(ret) = call.ret.as_ref() {
        foreach_arg(ret, &mut ctx, &mut f)
    }
    for val in call.args() {
        foreach_arg(val, &mut ctx, &mut f)
    }
}

fn foreach_arg(val: &Value, ctx: &mut ArgCtx, f: &mut dyn FnMut(&Value, &ArgCtx)) {
    let ctx_backup = ctx.clone();

    f(val, ctx);

    match &val.kind() {
        ValueKind::Group => {
            let val = val.checked_as_group();
            for inner in &val.inner {
                foreach_arg(inner, ctx, f);
                ctx.offset += inner.size(ctx.target);
            }
        }
        ValueKind::Union => {
            let val = val.checked_as_union();
            foreach_arg(&val.option, ctx, f);
        }
        ValueKind::Ptr => {
            let val = val.checked_as_ptr();
            if let Some(pointee) = val.pointee.as_ref() {
                ctx.base = Some(ptr_physical_addr(ctx.target, val));
                ctx.offset = 0;
                foreach_arg(pointee, ctx, f);
            }
        }
        _ => {}
    }

    *ctx = ctx_backup;
}
#[inline]
fn ptr_physical_addr(target: &Target, ptr: &PtrValue) -> u64 {
    if ptr.is_special() {
        let idx = (-(ptr.addr as i64)) as usize;
        target.special_ptrs()[idx]
    } else {
        target.data_offset() + ptr.addr
    }
}

#[inline]
fn vma_physical_addr(target: &Target, vma: &VmaValue) -> u64 {
    if vma.is_special() {
        let idx = (-(vma.addr as i64)) as usize;
        target.special_ptrs()[idx]
    } else {
        target.data_offset() + vma.addr
    }
}

*/
