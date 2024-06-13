use crate::vm::VmInstance;
use crate::{convert, vm::QemuConfig};
use chrono::Utc;
use feryr_prog::corpus_handle::{
    corpus::CorpusWrapper,
    prog::Prog,
    serialization::serialize,
    syscall::Syscall,
    target::{Target, TargetBuilder},
    ty::Type,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use std::{ 
    error::Error,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, Once},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("exec asan error")]
    ExecAsan,
    #[error("killed(maybe cause by timeout)")]
    TimeOut,
    #[error("unexpected executor exit status: {0}")]
    UnexpectedExitStatus(i32),
}

const UCOS_ARM: &str = include_str!(concat!(env!("OUT_DIR"), "/ucos.json"));
const FREERTOS_RISV32: &str = include_str!(concat!(env!("OUT_DIR"), "/freertos.json"));
const FREERTOS_AARCH64: &str = include_str!(concat!(env!("OUT_DIR"), "/freertos.json"));
const RTTHREAD_ARM: &str = include_str!(concat!(env!("OUT_DIR"), "/rtthread.json"));
const ZEPHYR_X86: &str = include_str!(concat!(env!("OUT_DIR"), "/zephyr.json"));

pub const TARGETS: [(&str, &str); 5] = [
    ("ucos/arm", UCOS_ARM),
    ("freertos/riscv", FREERTOS_RISV32),
    ("freertos/aarch64", FREERTOS_AARCH64),
    ("rtthread/arm", RTTHREAD_ARM),
    ("zephyr/i386", ZEPHYR_X86),
];

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysTarget {
    UCOS = 0,
    FreertosRisv32 = 1,
    Rtthread = 2,
    Zephyr = 3,
    FreertosAarch64 = 4,
}

impl FromStr for SysTarget {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match &s.to_ascii_lowercase()[..] {
            "ucos/arm" => Self::UCOS,
            "freertos/riscv32" => Self::FreertosRisv32,
            "freertos/aarch64" => Self::FreertosAarch64,
            "rtthread/arm" => Self::Rtthread,
            "zephyr/i386" => Self::Zephyr,
            _ => return Err("unsupported target".to_string()),
        };
        Ok(t)
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    TargetNotSupported,
    Parse(String),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::TargetNotSupported => write!(f, "target not supported"),
            LoadError::Parse(e) => write!(f, "parse: {}", e),
        }
    }
}

impl Error for LoadError {}

pub fn load_target<T: AsRef<str>>(
    target: T, 
    config: &QemuConfig, 
) -> Result<Target, LoadError> { 
    // load syscall description json file path
    let fuzzer_path = config.fuzzer_path.clone();
    let target_os = config.target_os.clone();
    let arch = config.arch.clone();
    let struct_json_path = PathBuf::from(fuzzer_path)
    .join("sys")
    .join(&target_os)
    .join(format!("{}-{}.json", &target_os, &arch));

    // loading into target struct
    let sys = target
    .as_ref()
    .parse::<SysTarget>()
    .map_err(|_| LoadError::TargetNotSupported)?;
    load_sys_target(sys, struct_json_path)
}

pub fn load_sys_target(
    sys: SysTarget, 
    json_path: PathBuf,
) -> Result<Target, LoadError> {
    static ONCE: Once = Once::new();
    static mut TARGETS_CACHE: Option<Mutex<Vec<Option<Target>>>> = None;
    ONCE.call_once(|| {
        let targets = vec![None; 18];
        unsafe { TARGETS_CACHE = Some(Mutex::new(targets)) };
    });

    let idx = sys as usize;
    let targets_cache = unsafe { TARGETS_CACHE.as_ref().unwrap() };
    let mut targets = targets_cache.lock().unwrap();
    if let Some(target) = &targets[idx] {
        return Ok(target.clone());
    }

    // cache missing, do load
    let description_json = load_description_json(json_path)?;
    dbg!(&description_json);  
    
    let (syscalls, tys, res_kinds) = convert::description_json_to_ast(&description_json)?;
    let target = build_target(&description_json, syscalls, tys, res_kinds)?;
    // save to cache
    targets[idx] = Some(target.clone());
    Ok(target)
}

fn build_target(
    descrption_json: &JsonValue,
    syscalls: Vec<Syscall>,
    tys: Vec<Type>,
    res_kinds: Vec<String>,
) -> Result<Target, LoadError> {
    let mut builder = TargetBuilder::new();
    let target_json = get(descrption_json, "Target")?;
    let mut ptrs = vec![0x0000000000000000, 0xffffffffffffffff, 0x9999999999999999];
    let os = get(target_json, "OS")?.as_str().unwrap();
    dbg!(&os);
    let arch = get(target_json, "Arch")?.as_str().unwrap();
    if os == "linux" {
        if arch == "amd64" {
            ptrs.push(0xffffffff81000000);
            ptrs.push(0xffffffffff600000);
        } else if arch == "riscv64" {
            ptrs.push(0xffffffe000000000);
            ptrs.push(0xffffff0000000000);
        }
    }
    builder
        .os(os)
        .arch(arch)
        .revision(get(descrption_json, "Revision")?.as_str().unwrap())
        .ptr_sz(get(target_json, "PtrSize")?.as_u64().unwrap())
        .page_sz(get(target_json, "PageSize")?.as_u64().unwrap())
        .page_num(get(target_json, "NumPages")?.as_u64().unwrap())
        .le_endian(get(target_json, "LittleEndian")?.as_bool().unwrap())
        .special_ptrs(ptrs)
        .data_offset(get(target_json, "DataOffset")?.as_u64().unwrap())
        .syscalls(syscalls)
        .tys(tys)
        .res_kinds(res_kinds);
    Ok(builder.build())
}

pub type JsonValue = simd_json::OwnedValue;
use simd_json::prelude::*;
use simd_json::OwnedValue;

pub fn get<'a>(val: &'a JsonValue, key: &str) -> Result<&'a JsonValue, LoadError> {
    val.get(key)
        .ok_or_else(|| LoadError::Parse(format!("missing '{}', json:\n{:#}", key, val)))
}
pub fn load_description_json(json_path: PathBuf) -> Result<OwnedValue, LoadError> {
    // Read the JSON file from the provided path
 
    let mut file_content = std::fs::read(&json_path)
        .map_err(|e| LoadError::Parse(format!("{}", e)))?;
    let val: OwnedValue = simd_json::to_owned_value(&mut file_content)
        .map_err(|e| LoadError::Parse(format!("Error parsing JSON: {}", e)))?;

    Ok(val)
}
// pub fn load_description_json(sys: SysTarget) -> Result<JsonValue, LoadError> {
//     static ONCE: Once = Once::new();
//     static mut JSONS_CACHE: Option<Mutex<Vec<Option<simd_json::OwnedValue>>>> = None;
//     ONCE.call_once(|| {
//         let jsons = vec![None; 18];
//         unsafe { JSONS_CACHE = Some(Mutex::new(jsons)) };
//     });
//     let idx = sys as usize;
//     let jsons_cache = unsafe { JSONS_CACHE.as_ref().unwrap() };
//     let mut jsons = jsons_cache.lock().unwrap();
//     if let Some(js) = &jsons[idx] {
//         return Ok(js.clone());
//     }

//     let mut d = TARGETS[idx].1.as_bytes().to_vec();
//     let val: simd_json::OwnedValue =
//         simd_json::to_owned_value(&mut d).map_err(|e| LoadError::Parse(format!("{}", e)))?;
//     jsons[idx] = Some(val.clone());
//     Ok(val)
// }
 
pub fn get_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn dump_to_file<T: Serialize, P: AsRef<Path>>(
    obj: T,
    file_path: P,
) -> Result<(), failure::Error> {
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;
    serde_json::to_writer(&mut f, &obj)?;
    f.write_all(b"\n")?;
    Ok(())
}

pub fn load_crash<P: AsRef<Path>>(path: P) -> Result<Prog, failure::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let p = serde_json::from_reader(reader)?;
    Ok(p)
}

pub fn load_corpus_from_file<P: AsRef<Path>>(path: P) -> Result<CorpusWrapper, failure::Error> {
    let mut corpus = CorpusWrapper::default();
    let p = &path.as_ref();
    dbg!(&p);
    if Path::new(p).is_file() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let deserializer = serde_json::Deserializer::from_reader(reader);
        let iterator = deserializer.into_iter::<Prog>();

        for item in iterator {
            corpus.add_prog(item?, 1);
        }
    }
    Ok(corpus)
}

pub fn try_repro(
    target: &Target,
    repro_prog: &Prog,
    thread_id: &usize,
    boot_config: &QemuConfig,
) -> bool {
    // reset
    let thread_id = thread_id ^ (Utc::now().timestamp_millis() as usize);
    let vm_instance = VmInstance::new(&boot_config, thread_id).unwrap();

    // repro program
    if let Err(e) = Prog::exec_input_prog(
        &repro_prog,
        &target,
        &vm_instance.qemu_shared_mem,
        &thread_id,
    ) {
        println!(
            "[{}]: {:?}: Repro Succeed: {}",
            chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
            thread_id,
            &e
        );
        return true;
    }

    println!(
        "[{}]: Repro Won't Succeed: {}",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
        thread_id
    );
    return false;
}

pub fn save_crash(
    work_dir: &String,
    current_prog: &Prog,
    tmp_target: &Target,
    tid: &usize,
) -> Result<(), failure::Error> {
    // format crash repro path
    let file_name = format!("{}/crashes/{}", work_dir, tid.to_string());
    let mut prog_buf = Vec::new();

    // format crash prog
    for call_idx in 0..current_prog.calls().len() {
        let s1 = current_prog.calls().get(call_idx).unwrap();
        let s2 = tmp_target.syscall_of(s1.sid());

        if s2.call_name() == "OSTaskCreate" ||
           s2.call_name() == "Str_ParseNbr_Int32U" ||
           s2.call_name() == "Str_ParseNbr_Int32S" {
            
            break;
        }

        let mut call_buf = [0; 1 << 16];
        let _ = serialize(tmp_target, current_prog, &mut call_buf, call_idx);
        prog_buf.extend_from_slice(&call_buf);
        prog_buf.push(0xfe);
    }

    // write to file
    let mut temp_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_name)
        .unwrap();
    let _ = temp_file.write(&prog_buf);
    let mut tmp_file = File::create(file_name + "-0").unwrap();

    // check if write to file have any error
    let _ = match write!(tmp_file, "{}", current_prog.display(tmp_target)) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    };
    Ok(())
}

// Check qemu pipe file, if asan find any asan bug
pub fn check_crash(work_dir: &String, tid: &usize) -> ExecError {
    // load log file name
    let file_name: String = format!("{}/{}/{}-{}", work_dir, "output", "vm", tid.to_string());

    // check weather the file contents contains string like 'qEmbAsan'
    let context = std::fs::read_to_string(file_name).unwrap();
    for line in context.lines().rev() {
        if line.contains("qEmbAsan") {
            return ExecError::ExecAsan;
        }
    }

    return ExecError::TimeOut;
}
