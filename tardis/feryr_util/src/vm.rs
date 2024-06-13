use chrono::*;
use failure;
use feryr_prog::{corpus_handle::prog::Prog, cover_handle::cover::*, crash_handle::crash::*};
use rand::prelude::*;
use rand::rngs::SmallRng;
use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_value::Value;
use std::{ 
    collections::VecDeque,
    env,
    fs::{create_dir_all, File},
    io::BufReader,
    path::Path,
    process::{Child, Command, Stdio},
    thread, time,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter)]
pub enum ShmOps {
    CorpuShm,
    DataShm,
    MiscShm,
    RunStatus,
}

#[derive(Default, Clone, Debug)]
pub struct QemuAddr {
    pub basic_addr: String,
    pub addr_offset: String,
}

#[derive(Debug)]
pub struct VmInstance {
    pub current_prog: Prog,
    _crash_info: Crash,
    pub qemu_shared_mem: Vec<(String, String)>,
    pub qemu_instance: Child,
}
impl VmInstance {
    // init a vm instance
    pub fn new(qemu_config: &QemuConfig, tid: usize) -> Result<VmInstance, failure::Error> {
        let mut vm_instance = VmInstance {
            current_prog: Prog::default(),
            // TODO: here should add crash info not simply ingore
            _crash_info: Crash::default(),
            qemu_shared_mem: Vec::default(),
            qemu_instance: parse_qemu_boot(&qemu_config, tid)?,
        };

        for op in ShmOps::iter() {
            let shm_address = fetch_address(&qemu_config, op.clone())?;
            vm_instance.qemu_shared_mem.push(shm_address);
        }

        Ok(vm_instance)
    }
}

pub struct VmExecutor {
    pub thread_id: usize,
    pub boot_config: QemuConfig,
    pub prog_buffer: VecDeque<Prog>,
    pub crashes: Vec<Crash>,
    pub vm_instance: VmInstance,
    pub rng: SmallRng,
}
impl VmExecutor {
    // init vm executor
    pub fn new(qemu_config: &QemuConfig, tid: usize) -> Result<VmExecutor, failure::Error> {
        let vm_executor = VmExecutor {
            thread_id: tid,
            boot_config: qemu_config.clone(),
            prog_buffer: VecDeque::new(),
            crashes: Vec::default(),
            vm_instance: VmInstance::new(&qemu_config.clone(), tid)?,
            rng: SmallRng::from_entropy(),
        };
        Ok(vm_executor)
    }

    // check if there is new coverage
    pub fn vm_cover_handle(&mut self, cover: &mut Cover) -> Result<u8, failure::Error> {
        let trace_bits = cover.handle_bits(&self.thread_id).unwrap();
        let cov_res = cover.has_new_bits(trace_bits).unwrap();
        Ok(cov_res)
    }

    pub fn stop(&mut self) {
        // kill old qemu
        self.vm_instance.qemu_instance.kill().unwrap();
        self.vm_instance
            .qemu_instance
            .wait()
            .expect("failed to wait on child");
        let pid = self.vm_instance.qemu_instance.id() as i32;
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
        // self.vm_instance.qemu_instance.kill().unwrap();
        // self.vm_instance
        //     .qemu_instance
        //     .wait()
        //     .expect("failed to wait on child");
    }

    pub fn restart(&mut self) {
        // reset qemu
        self.thread_id = self.thread_id ^ (Utc::now().timestamp_millis() as usize);
        self.vm_instance = VmInstance::new(&self.boot_config, self.thread_id).unwrap();
    }
}

#[derive(Clone, Default, Deserialize, Serialize, Debug)]
pub struct QemuConfig {
    pub fuzzer_path: String,
    pub target_os: String,
    pub machine_type: String,
    pub kernel_obj: String,
    pub qmp_path: String,
    pub workdir: String,
    pub arch: String,
    pub corpus_path: String,
    pub port: u32,
    pub cpu_num: u32,
    pub mem_size: u32,
    pub unique_id: String,
    pub func_addr: String,
}

impl QemuConfig {
    pub fn get_field_by_name<T, R>(data: T, field: &str) -> R
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let mut map = match serde_value::to_value(data) {
            Ok(Value::Map(map)) => map,
            _ => panic!("expected a struct"),
        };

        let key = Value::String(field.to_owned());
        let value = match map.remove(&key) {
            Some(value) => value,
            None => panic!("no such field"),
        };

        match R::deserialize(value) {
            Ok(r) => r,
            Err(_) => panic!("wrong type?"),
        }
    }

    pub fn parse_boot_json(config_file_path: String) -> Result<QemuConfig, failure::Error> {
        let qemu_config_file = File::open(config_file_path)?;
        let reader = BufReader::new(qemu_config_file);
        let qemu_config: QemuConfig = serde_json::from_reader(reader)?;

        Ok(qemu_config)
    }
}

pub fn parse_qemu_start_cmd(
    qemu_config: &QemuConfig,
    tid: usize,
) -> Result<Command, failure::Error> {
    // read args from config file
    let vm_arch: String = QemuConfig::get_field_by_name(&qemu_config, "arch");
    let vm_qmp_path: String = QemuConfig::get_field_by_name(&qemu_config, "qmp_path");
    let vm_machine_type: String = QemuConfig::get_field_by_name(&qemu_config, "machine_type");
    let vm_kernel_obj: String = QemuConfig::get_field_by_name(&qemu_config, "kernel_obj");
    let vm_func_addr: String = QemuConfig::get_field_by_name(&qemu_config, "func_addr");
    let qemu_dir = vm_qmp_path.clone()
        + &"/../../build/".to_owned()
        + &vm_arch
        + &"-softmmu/qemu-system-".to_owned()
        + &vm_arch;

    // load plugin path
    let plugin_path = vm_qmp_path.to_owned()
        + &"/../../build/contrib/plugins/libdrcov.so,coverfile=cover-".to_owned()
        + &tid.to_string()
        + &",funcaddr=".to_owned()
        + &vm_func_addr.to_string();

    // create output dir
    create_dir_all(format!(
        "{}/fuzz-loop-{}/output",
        &qemu_config.workdir, &qemu_config.unique_id
    ))?;

    // set mapped address file
    let qemu_shm_path_key = "QEMU_RAM_SHM_PATH".to_string();
    let qemu_shm_path = format!("{}{}", "feryr-".to_owned(), tid.to_string());

    // parse qemu start command according to os
    let mut cmd = Command::new(qemu_dir);
    cmd.env(qemu_shm_path_key, qemu_shm_path);
    match qemu_config.target_os.as_str() {
        "ucos" => {
            cmd.args([
                "-machine",
                &vm_machine_type,
                "-display",
                "none",
                "-net",
                "none",
                "-chardev",
                "stdio,id=con,mux=on",
                "-serial",
                "chardev:con",
                "-mon",
                "chardev=con,mode=readline",
                "-kernel",
                &vm_kernel_obj,
                "-plugin",
                &plugin_path,
                "-d",
                "plugin",
                "-nographic",
                "-snapshot",
            ]);
        }
        "freertos" => {
            cmd.args([
                "-machine",
                &vm_machine_type,
                "-smp",
                "4",
                "-bios",
                "none",
                "-chardev",
                "stdio,id=con,mux=on",
                "-serial",
                "chardev:con",
                "-mon",
                "chardev=con,mode=readline",
                "-kernel",
                &vm_kernel_obj,
                "-plugin",
                &plugin_path,
                "-d",
                "plugin",
                "-nographic",
                "-snapshot",
            ]);
        }
        "rtthread" => {
            let sd_path = Path::new(&vm_kernel_obj)
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                + "/sd.bin";

            cmd.args([
                "-machine",
                &vm_machine_type,
                "-display",
                "none",
                "-sd",
                &sd_path,
                "-net",
                "none",
                "-chardev",
                "stdio,id=con,mux=on",
                "-serial",
                "chardev:con",
                "-mon",
                "chardev=con,mode=readline",
                "-kernel",
                &vm_kernel_obj,
                "-plugin",
                &plugin_path,
                "-d",
                "plugin",
                "-nographic",
                "-snapshot",
            ]);
        }
        "zephyr" => {
            cmd.args([
                "-machine",
                &vm_machine_type,
                "-display",
                "none",
                "-net",
                "none",
                "-chardev",
                "stdio,id=con,mux=on",
                "-serial",
                "chardev:con",
                "-mon",
                "chardev=con,mode=readline",
                "-kernel",
                &vm_kernel_obj,
                "-plugin",
                &plugin_path,
                "-d",
                "plugin",
                "-nographic",
                "-snapshot",
            ]);
        }
        _ => {
            panic!("unsupported os");
        }
    };

    Ok(cmd)
}

pub fn parse_qemu_boot<'a>(qemu_config: &QemuConfig, tid: usize) -> Result<Child, failure::Error> {
    // parse qemu config file

    let start_cmd: Child;
    let mut qemu_cmd = parse_qemu_start_cmd(qemu_config, tid).unwrap();
    let qemu_output_file = Stdio::from(
        File::create(format!(
            "{}/fuzz-loop-{}/output/vm-{}",
            &qemu_config.workdir,
            &qemu_config.unique_id.to_owned(),
            tid.to_string()
        ))
        .unwrap(),
    );

    let args: Vec<String> = env::args().collect();
    if args.len() == 4 {
        start_cmd = qemu_cmd
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to boot vm");
    } else {
        start_cmd = qemu_cmd
            .stdout(qemu_output_file)
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to boot vm");
    }

    let vm_wait_boot_time = time::Duration::from_millis(20);
    thread::sleep(vm_wait_boot_time);

    Ok(start_cmd)
}

pub fn fetch_address(
    qemu_config: &QemuConfig,
    shm_name: ShmOps,
) -> Result<(String, String), failure::Error> {
    // define regex for shm's guest virtual addressa
    let virtual_regex = Regex::new(r"([0-9abcdefABCDEF]){8}").unwrap();
    let kernel_path: String = QemuConfig::get_field_by_name(&qemu_config, "kernel_obj");

    // orignize && execute objdump cmd
    let shm_var = match shm_name {
        ShmOps::CorpuShm => "corpus_buffer",
        ShmOps::DataShm => "data_buffer",
        ShmOps::MiscShm => "misc",
        ShmOps::RunStatus => "executor_status",
    };

    // println!(
    //     "[{}]: checking kernel symbol: {}",
    //     chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
    //     shm_var
    // );
    let readelf_cmd = " readelf -a ".to_owned() + &kernel_path + " | grep -wn  " + &shm_var;
    let readelf_bash_cmd = Command::new("bash")
        .arg("-c")
        .arg(readelf_cmd)
        .output()
        .expect("failed to obtain kernel symbol message!");
    // get virtual address and match it in regex
    let readelf_address = String::from_utf8(readelf_bash_cmd.stdout).unwrap();
    let virtual_address = virtual_regex.captures(&readelf_address);
    let virtual_address = match virtual_address {
        Some(v) => v.get(0).unwrap().as_str(),
        None => {
            println!(
                "[{}]: failed to obtain kernel symbol {}",
                chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
                shm_var
            );
            return Err(failure::err_msg("failed to obtain kernel symbol message!"));
        }
    };

    let num_length = virtual_address.len();
    let mut addr_offset = virtual_address.to_owned();
    unsafe {
        addr_offset.as_bytes_mut()[0] = '0' as u8;
    };
    let mut vir_add = virtual_address.to_owned();
    unsafe {
        vir_add.as_bytes_mut()[num_length - 1] = '0' as u8;
    }

    Ok(((&virtual_address).to_string(), addr_offset.to_string()))
}
