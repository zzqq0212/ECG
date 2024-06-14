use memmap2::MmapOptions;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_value::Value;
use std::{
    env, fs,
    fs::{create_dir_all, File, OpenOptions},
    io,
    io::{BufReader, Read},
    process::{Child, Command, Stdio},
    result::Result,
    thread, time,
};

use failure;

pub const IN_SHM_SZ: usize = 1 << 16;
pub fn get_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
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

    pub fn parse_json(config_file_path: String) -> Result<QemuConfig, failure::Error> {
        let qemu_config_file = File::open(config_file_path)?;
        let reader = BufReader::new(qemu_config_file);
        let qemu_config: QemuConfig = serde_json::from_reader(reader)?;

        Ok(qemu_config)
    }
}

pub fn parse_qemu_boot(qemu_config: &QemuConfig, tid: usize) -> Result<Child, failure::Error> {
    // qemu_start_cmd =
    //     " /root/fuzz/kernel/qemu/build/" + qemu_arch + "-softmmu/qemu-system-" + qemu_arch +
    //     " -machine " + qemu_machine + " -display none  -serial stdio " +
    //     " -kernel " + qemu_kernel_obj + " -nographic -S -gdb tcp::" + gdb_port +
    //     " -qmp unix:" + socket_pipeline + ",server,nowait" + " 2>&1 | tee vm.log &";
    let vm_arch: String = QemuConfig::get_field_by_name(&qemu_config, "arch");
    let vm_qmp_path: String = QemuConfig::get_field_by_name(&qemu_config, "qmp_path");
    let vm_machine_type: String = QemuConfig::get_field_by_name(&qemu_config, "machine_type");
    let vm_kernel_obj: String = QemuConfig::get_field_by_name(&qemu_config, "kernel_obj");
    let plugin_path = vm_qmp_path.clone()
        + &"/../../build/contrib/plugins/libdrcov.so,coverfile=cover-".to_owned()
        + &tid.to_string();

    let qemu_dir = vm_qmp_path
        + &"/../../build/".to_owned()
        + &vm_arch
        + &"-softmmu/qemu-system-".to_owned()
        + &vm_arch;

    let key = "QEMU_RAM_SHM_PATH";
    let path = format!("{}{}", "feryr-".to_owned(), tid.to_string());
    create_dir_all(format!(
        "{}/fuzz-loop-{}/output",
        &qemu_config.workdir, &qemu_config.unique_id
    ))?;
    let _std_path = format!(
        "{}/fuzz-loop-{}/output/vm-{}",
        &qemu_config.workdir,
        &qemu_config.unique_id,
        get_random_string(10)
    );
    let std_out = File::create("test1")?;
    let std_err = File::create("test2")?;

    let start_cmd: Child = Command::new(qemu_dir)
        .env(key, path)
        .args([
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
        ])
        .stdout(Stdio::from(std_out))
        .stderr(Stdio::from(std_err))
        .spawn()
        .expect("failed to execute process");

    // thread::sleep(time::Duration::from_secs(5));
    Ok(start_cmd)
}

fn main() {
    // set two mode
    // 1. single debug mode 2. whole call mode
    let args: Vec<String> = env::args().collect();
    if args.len() < 10 {
        println!("wrong args");
        return;
    }
    let mode = u64::from_str_radix(&args[1], 10).unwrap();
    let mut _qemu_instance: Child;
    let kernel_path: &String;
    let path: &String;
    let file_name: &String;
    let mut call_idx = 0;
    let mut qemu_path: &String = &String::new();
    let mut qemu_config = QemuConfig::default();

    if mode == 1 {
        // execute syscall
        if args.len() < 10 {
            println!("wrong args: ./feryr_debug -mod -k OS.elf -c testcase -s share_mem -i num");
            return;
        }
        kernel_path = &args[3];
        path = &args[5];
        file_name = &args[7];
        call_idx = u64::from_str_radix(&args[9], 10).unwrap();
        println!("qemu_path will not set {}", qemu_path);
    } else if mode == 2 {
        // execute calls
        if args.len() < 10 {
            println!(
                "wrong args: ./feryr_debug -mod -k OS.elf -c testcase -s share_mem -q qemu_path"
            );
            return;
        }

        kernel_path = &args[3];
        path = &args[5];
        file_name = &args[7];
        qemu_path = &args[9];
        qemu_config = QemuConfig::parse_json(qemu_path.to_owned()).unwrap();
    } else if mode == 3 {
        // execute calls
        if args.len() < 10 {
            println!(
                "wrong args: ./feryr_debug -mod -k OS.elf -c testcase -s share_mem -q qemu_path"
            );
            return;
        }

        kernel_path = &args[3];
        path = &args[5];
        file_name = &args[7];
        qemu_path = &args[9];
        qemu_config = QemuConfig::parse_json(qemu_path.to_owned()).unwrap();
    } else {
        println!("wrong arguments, mode can only be 1/2/3");
        return;
    }

    // read symbol table of corpus_buffer && executor_status
    let readelf_cmd =
        " readelf -a ".to_owned() + &kernel_path + &" | grep corpus_buffer".to_owned();
    let readelf_bash_cmd = Command::new("bash")
        .arg("-c")
        .arg(readelf_cmd)
        .output()
        .expect("failed to obtain kernel symbol message!");

    // get virtual address and match it in regex
    let virtual_regex = Regex::new(r"([0-9abcdefABCDEF]){8}").unwrap();
    let readelf_address = String::from_utf8(readelf_bash_cmd.stdout).unwrap();

    let virtual_address = virtual_regex
        .captures(&readelf_address)
        .unwrap()
        .get(0)
        .map_or("", |m| m.as_str());
    let num_length = virtual_address.len();
    let mut addr_offset = virtual_address.to_owned();
    unsafe {
        addr_offset.as_bytes_mut()[0] = '0' as u8;
    };
    let mut vir_add = virtual_address.to_owned();
    unsafe {
        vir_add.as_bytes_mut()[num_length - 1] = '0' as u8;
    }

    // get status virtual address
    let status_cmd =
        " readelf -a ".to_owned() + &kernel_path + &" | grep executor_status".to_owned();
    let status_bash_cmd = Command::new("bash")
        .arg("-c")
        .arg(status_cmd)
        .output()
        .expect("failed to obtain kernel symbol message!");
    // get virtual address and match it in regex
    let status_address = String::from_utf8(status_bash_cmd.stdout).unwrap();
    let status_virtual_address = virtual_regex
        .captures(&status_address)
        .unwrap()
        .get(0)
        .map_or("", |m| m.as_str());
    let status_num_length = status_virtual_address.len();
    let mut status_addr_offset = status_virtual_address.to_owned();
    unsafe {
        status_addr_offset.as_bytes_mut()[0] = '0' as u8;
    };
    let mut status_vir_add = status_virtual_address.to_owned();
    unsafe {
        status_vir_add.as_bytes_mut()[status_num_length - 1] = '0' as u8;
    }

    // read buffer from file
    let mut buf = vec![0; IN_SHM_SZ];
    let mut calls: Vec<Vec<u8>> = Vec::new();
    let mut tmp_buf: Vec<u8> = Vec::new();
    let fw: u8 = 0x66;
    let uw: u8 = 0x75;
    let cw: u8 = 0x63;
    let kw: u8 = 0x6b;

    let f = fs::File::open(path.clone()).unwrap();
    let mut r = io::BufReader::new(f);
    r.read(&mut buf).unwrap();

    for mut _i in 0..(buf.len() - 4) {
        if buf[_i] == fw && buf[_i + 1] == uw && buf[_i + 2] == cw && buf[_i + 3] == kw {
            calls.push(tmp_buf.to_owned());
            tmp_buf.clear();
            _i += 3;
        } else {
            tmp_buf.push(buf[_i]);
        }
    }

    // if execute single systemcall check if call_idx overflow
    if mode == 1 && call_idx >= calls.len() as u64 {
        println!("{}, {}", call_idx, calls.len());
        println!("wrong call_index!");
        return;
    } else if mode == 1 && call_idx < calls.len() as u64 {
        // mmap share memory
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_name)
            .unwrap();
        let mut corpus_shm = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

        // write to vm
        let start_index = usize::from_str_radix(&addr_offset, 16).unwrap();
        dbg!(&addr_offset);
        dbg!(&start_index);
        let buff = calls[call_idx as usize].to_owned();
        dbg!(&buff[0..16]);
        corpus_shm[start_index..start_index + buff.len()].copy_from_slice(&buff);
        // check result
        let mut check_buf = vec![0; IN_SHM_SZ];
        let lenn = check_buf.len();
        check_buf.copy_from_slice(&corpus_shm[start_index..start_index + lenn]);
        dbg!(&check_buf[0..16]);
    }

    // if execute calls
    if mode == 2 {
        let rng = rand::thread_rng().gen_range(0..100);
        let _qemu_instance = parse_qemu_boot(&qemu_config, rng).unwrap();

        let sleep_time = time::Duration::from_millis(200);
        thread::sleep(sleep_time);

        let shm_path = format!("{}{}", "/dev/shm/feryr-", rng.to_string().as_str());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(shm_path)
            .unwrap();
        let mut corpus_shm = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

        let pathdir = &path[..49];
        let paths = fs::read_dir(pathdir.to_owned()).unwrap();

        for crash in paths {
            let tem = &crash.unwrap();
            let mut buf = vec![0; IN_SHM_SZ];
            let mut calls: Vec<Vec<u8>> = Vec::new();
            let mut tmp_buf: Vec<u8> = Vec::new();
            let a: u8 = 0xfe;

            let f = fs::File::open(tem.path()).unwrap();
            let mut r = io::BufReader::new(f);
            r.read(&mut buf).unwrap();

            for i in &buf {
                if i.to_owned() != a {
                    tmp_buf.push(i.to_owned());
                } else {
                    calls.push(tmp_buf.to_owned());
                    tmp_buf.clear();
                }
            }

            // set corpus buffer
            let mut call_idxx = 0;
            let mut check_buf = vec![0; 2];
            let status_start_index = usize::from_str_radix(&status_addr_offset, 16).unwrap();
            let start_index = usize::from_str_radix(&addr_offset, 16).unwrap();
            loop {
                check_buf.copy_from_slice(&corpus_shm[status_start_index..status_start_index + 2]);

                if call_idxx >= calls.len() {
                    dbg!("end execute! {}", call_idxx);
                    break;
                }

                if call_idxx < calls.len() && check_buf[0] == 'w' as u8 && check_buf[1] == 'a' as u8
                {
                    let buff = calls[call_idxx as usize].to_owned();
                    call_idxx += 1;
                    corpus_shm[start_index..start_index + buff.len()].copy_from_slice(&buff);

                    check_buf[0] = 'r' as u8;
                    check_buf[1] = 'e' as u8;
                    corpus_shm[status_start_index..status_start_index + 2]
                        .copy_from_slice(&check_buf);
                    println!("call idx = {}", &call_idxx);
                }

                let one_sec = time::Duration::from_millis(300);
                thread::sleep(one_sec);
            }
        }
    }

    if mode == 3 {
        let rng = rand::thread_rng().gen_range(0..100);
        let _qemu_instance = parse_qemu_boot(&qemu_config, rng).unwrap();

        let sleep_time = time::Duration::from_millis(200);
        thread::sleep(sleep_time);

        let shm_path = format!("{}{}", "/dev/shm/feryr-", rng.to_string().as_str());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(shm_path)
            .unwrap();
        let mut corpus_shm = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

        let mut buf = vec![0; IN_SHM_SZ];
        let mut calls: Vec<Vec<u8>> = Vec::new();
        let mut tmp_buf: Vec<u8> = Vec::new();
        let a: u8 = 0xfe;

        let f = fs::File::open(path).unwrap();
        let mut r = io::BufReader::new(f);
        r.read(&mut buf).unwrap();

        for i in &buf {
            if i.to_owned() != a {
                tmp_buf.push(i.to_owned());
            } else {
                calls.push(tmp_buf.to_owned());
                tmp_buf.clear();
            }
        }

        // set corpus buffer
        let mut call_idxx = 0;
        let mut check_buf = vec![0; 2];
        let status_start_index = usize::from_str_radix(&status_addr_offset, 16).unwrap();
        let start_index = usize::from_str_radix(&addr_offset, 16).unwrap();
        loop {
            check_buf.copy_from_slice(&corpus_shm[status_start_index..status_start_index + 2]);

            if call_idxx >= calls.len() {
                dbg!("end execute! {}", call_idxx);
                break;
            }

            if call_idxx < calls.len() && check_buf[0] == 'w' as u8 && check_buf[1] == 'a' as u8 {
                let buff = calls[call_idxx as usize].to_owned();
                call_idxx += 1;
                corpus_shm[start_index..start_index + buff.len()].copy_from_slice(&buff);

                check_buf[0] = 'r' as u8;
                check_buf[1] = 'e' as u8;
                corpus_shm[status_start_index..status_start_index + 2].copy_from_slice(&check_buf);
                println!("call idx = {}", &call_idxx);
            }

            let one_sec = time::Duration::from_millis(300);
            thread::sleep(one_sec);
        }
    }

    return;
}
