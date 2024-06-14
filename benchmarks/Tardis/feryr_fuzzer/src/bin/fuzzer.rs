use chrono::*;

use defs::*;
use feryr::*;
use feryr_prog::{
    corpus_handle::gen, corpus_handle::mutation, corpus_handle::prog::Prog,
    corpus_handle::relation::Relation, corpus_handle::relation::RelationWrapper,
};
use std::{
    env,
    fs::create_dir_all,
    io::{Error, ErrorKind},
    sync::{
        atomic::{Ordering::*, *},
        Arc, RwLock,
    },
    thread, time,
};
use thread_id;
use util::{
    cpu_bind::*,
    sys::{
        check_crash, dump_to_file, get_random_string, load_corpus_from_file, save_crash, try_repro,
        ExecError,
    },
    vm::*,
};

fn main() -> Result<(), failure::Error> {
    ctrlc::set_handler(move || {
        // kill process
        RUNNING.store(false, Ordering::SeqCst);
        println!(
            "[{}]: Receive Ctrl C, wait...",
            chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
        );
        quit_fuzzer();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    parse_args();

    println!(
        "
            ███████╗███████╗██████╗ ██╗   ██╗██████╗ 
            ██╔════╝██╔════╝██╔══██╗╚██╗ ██╔╝██╔══██╗
            █████╗  █████╗  ██████╔╝ ╚████╔╝ ██████╔╝
            ██╔══╝  ██╔══╝  ██╔══██╗  ╚██╔╝  ██╔══██╗
            ██║     ███████╗██║  ██║   ██║   ██║  ██║
            ╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝"
    );

    // parse input args
    let config_file_path: String;
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            config_file_path = args[2].to_string();
        }
        4 => {
            config_file_path = args[2].to_string();
        }
        _ => {
            usage_help();
            return Err(Error::new(ErrorKind::Other, "wrong arguments").into());
        }
    }

    // init Manager to perserve runtime status
    println!(
        "[{}]: init fuzz manager",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
    );

    let mut fuzz_manager = feryr::FuzzManager::new();
    // read from fuzz config
    println!(
        "[{}]: parse config file: {}",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
        config_file_path
    );
    fuzz_manager.qemu_config = QemuConfig::parse_boot_json(config_file_path)?;

    // load corpus from local file
    println!(
        "[{}]: load target {}/{}",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
        &fuzz_manager.qemu_config.target_os,
        &fuzz_manager.qemu_config.arch,
    );

    // feryr::load_corpus(&mut fuzz_manager);
    let target_name: String = format!(
        "{}/{}",
        &fuzz_manager.qemu_config.target_os, &fuzz_manager.qemu_config.arch
    );

    // load target os information from struct.json file including system call spec and system architectures 
    fuzz_manager.target = util::sys::load_target(&target_name, &fuzz_manager.qemu_config)?;

    // init relation table
    println!(
        "[{}]: init relation table",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
    );
    let relation = Relation::new(&fuzz_manager.target);
    fuzz_manager.relation = RelationWrapper::new(relation);

    // bind fuzz thread to cpus
    println!(
        "[{}]: initialize all vms",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
    );
    let cpu_num = fuzz_manager.qemu_config.cpu_num;
    let mut handles = Vec::new();
    let free_cpus = util::cpu_bind::find_free_cpus(cpu_num as usize);
    let free_cpus_len = free_cpus.len();
    let bind_cpus = if free_cpus_len < 1 {
        println!("The number of free cpus is less than the number of jobs. Will not bind any thread to any cpu.");
        false
    } else {
        true
    };

    // read all system call description from *.txt
    fuzz_manager.corpus = load_corpus_from_file(&fuzz_manager.qemu_config.corpus_path).unwrap();
    println!(
        "[{}]: load corpus from path, len is {}",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
        // &fuzz_manager.qemu_config.corpus_path,
        &fuzz_manager.corpus.len(),
    );

    // set manager log to console or file
    fuzz_manager.qemu_config.unique_id = get_random_string(10);
    let crash_dir: String = format!(
        "{}/{}-{}/{}",
        &fuzz_manager.qemu_config.workdir,
        "fuzz-loop",
        &fuzz_manager.qemu_config.unique_id,
        "crashes"
    );
    create_dir_all(&crash_dir).unwrap();
    let cover_path: String = format!(
        "{}/{}-{}/{}",
        &fuzz_manager.qemu_config.workdir,
        "fuzz-loop",
        &fuzz_manager.qemu_config.unique_id,
        "cover"
    );
    let log_path: String = format!(
        "{}/{}-{}/{}",
        &fuzz_manager.qemu_config.workdir, "fuzz-loop", &fuzz_manager.qemu_config.unique_id, "logs"
    );
    println!(
        "[{}]: work dir is {}/fuzz-loop-{}",
        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
        &fuzz_manager.qemu_config.workdir,
        &fuzz_manager.qemu_config.unique_id,
    );

    let fuzz_manager = Arc::new(RwLock::new(fuzz_manager));
    let status_manager = Arc::clone(&fuzz_manager);

    thread::spawn(move || -> Result<(), failure::Error> {
        println!(
            "[{}]: Let's roll!",
            chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
        );
        thread::sleep(time::Duration::from_millis(3000));
        let mut total_branch = 0;
        // console output for every 3 seconds
        loop {
            // TODO: modified into 10s
            let ten_millis = time::Duration::from_millis(3000);
            thread::sleep(ten_millis);
            match status_manager.write() {
                Ok(mut handle) => {
                    // update coverage info
                    if total_branch == 0 {
                        handle.coverage.last_branch = handle.coverage.branch;
                    } else {
                        handle.coverage.last_branch = handle.coverage.branch - total_branch;
                    }
                    total_branch = handle.coverage.branch;

                    let now_time = chrono::Utc::now().format("[%Y-%m-%d][%H:%M:%S]");

                    let log_msg = format!(
                        "{}: total_exec: {}, total branches: {}, last_exec: {}, last branches: {}, crash: {}",
                        &now_time,
                        &handle.total_exec,
                        &handle.coverage.branch,
                        &handle.last_exec,
                        &handle.coverage.last_branch,
                        &handle.corpus.len_exceptions()
                    );
                    println!("{}", &log_msg);
                    dump_to_file(
                        now_time.to_string() + &handle.coverage.branch.to_string(),
                        &cover_path,
                    )?;
                    dump_to_file(&log_msg, &log_path)?;
                    handle.last_exec = 0;
                }
                Err(_e) => {
                    RUNNING.store(false, SeqCst);
                }
            }
        }
    });

    // bind one fuzzer instance to a specific cpu 
    for cpu_idx in 0..cpu_num {
        let thread_manager = Arc::clone(&fuzz_manager);

        // bind thread to a cpu
        let cpu_id = if bind_cpus {
            free_cpus[cpu_idx as usize]
        } else {
            0
        };
        bind_thread_to_cpu_core(cpu_id);

        // start vm
        let config = thread_manager.read().unwrap().qemu_config.clone();
        let cnt = 0;
        let handle = thread::spawn(move || {
            println!(
                "[{}]: {:?} is running",
                chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
                thread::current().id()
            );

            // let vm start by order, not at instant
            let time = Utc::now().timestamp_millis() as usize;
            let mut vm_executor = VmExecutor::new(&config, thread_id::get() ^ time).unwrap();

            // start fuzz loop
            fuzz_loop(&mut vm_executor, thread_manager, cnt);
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().unwrap();
    }

    Ok(())
}

// fuzzing loop start here
pub fn fuzz_loop(
    executor: &mut VmExecutor,
    fuzz_manager: Arc<RwLock<feryr::FuzzManager>>,
    cnt: u32,
) {
    // set generation rules
    const GENERATE_PERIOD: u32 = 50;

    // load fuzzer workdir and manager_id from config file
    let work_dir = format!(
        "{}/{}-{}",
        &fuzz_manager.read().unwrap().qemu_config.workdir,
        "fuzz-loop",
        &fuzz_manager.read().unwrap().qemu_config.unique_id
    );

    // start fuzzing loop
    while RUNNING.load(Ordering::SeqCst) {
        // init values
        let mut current_prog: Prog = Prog::default();
        let mut tmp_target = feryr_prog::corpus_handle::target::Target::default();

        // generate and mutate
        match fuzz_manager.read() {
            Ok(handle) => {
                tmp_target = handle.target.clone();
                if handle.corpus.is_empty() || cnt % GENERATE_PERIOD == 0 {
                    current_prog = gen::gen_prog(&handle.target, 
                                                 &handle.relation, 
                                                 &mut executor.rng);
                    executor.vm_instance.current_prog = current_prog.clone();
                } else {
                    let mut p = handle
                                      .corpus
                                      .select_one(&mut executor.rng)
                                      .unwrap();
                    mutation::mutate(
                        &handle.target,
                        &handle.relation,
                        &handle.corpus,
                        &mut executor.rng,
                        &mut p,
                    );
                    executor.vm_instance.current_prog = p;
                }
            }
            Err(_e) => {
                dbg!("failed to generate and mutate!");
            }
        }

        // write to manager
        match fuzz_manager.write() {
            Ok(mut handle) => {
                handle.last_exec = handle.last_exec + 1;
                handle.total_exec = handle.total_exec + 1;
            }
            Err(_e) => {
                dbg!("failed to write to manager!");
                RUNNING.store(false, SeqCst);
            }
        }

        // execute seeds
        if let Err(e) = Prog::exec_input_prog(
            &current_prog.clone(),
            &tmp_target,
            &executor.vm_instance.qemu_shared_mem,
            &executor.thread_id,
        ) {
            println!(
                "[{}]: VM-{} getting crash: {}",
                chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
                &executor.thread_id,
                &e
            );

            if format!("{}", e) == "create file err!" {
                continue;
            }

            // check crash type
            match check_crash(&work_dir, &executor.thread_id) {
                ExecError::ExecAsan => {
                    // asan error detected, direct save to corpus
                    println!(
                        "[{}]: Memory Crash Detected",
                        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
                    );
                    save_crash(&work_dir, &current_prog, &tmp_target, &executor.thread_id).unwrap();
                }
                ExecError::TimeOut => {
                    // timeout error detected, save to corpus if can repro
                    println!(
                        "[{}]: Timeout Crash Detected",
                        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
                    );
                    if try_repro(
                        &tmp_target,
                        &current_prog,
                        &executor.thread_id,
                        &executor.boot_config,
                    ) == true
                    {
                        save_crash(&work_dir,
                                   &current_prog, 
                                   &tmp_target, 
                                   &executor.thread_id).unwrap();
                    }
                }
                _ => {
                    // unknown error detected, save to corpus if can repro
                    println!(
                        "[{}]: Unknown Crash Detected",
                        chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S")
                    );
                    if try_repro(
                        &tmp_target,
                        &current_prog,
                        &executor.thread_id,
                        &executor.boot_config,
                    ) == true
                    {
                        save_crash(&work_dir,
                                   &current_prog,
                                   &tmp_target, 
                                   &executor.thread_id).unwrap();
                    }
                }
            }

            match fuzz_manager.write() {
                Ok(mut handle) => {
                    // reset executor
                    executor.stop();
                    let res = executor.vm_cover_handle(&mut handle.coverage).unwrap();
                    if res == 3 {
                        dbg!("stack failed!");
                    }
                    executor.restart();
                    handle.corpus.add_exception(current_prog.clone());
                }
                Err(_e) => {
                    dbg!("failed to write crash prog!");
                }
            }
        } else {
            // normal exec, handle coverage
            let file_name = format!("{}/{}", &work_dir, "corpus.db");
            dump_to_file(current_prog.clone(), &file_name).unwrap();
            match fuzz_manager.write() {
                Ok(mut handle) => {
                    let res = executor.vm_cover_handle(&mut handle.coverage).unwrap();
                    // add coverage info to instruct mutate (to switch to bind fuzz, make cover_cnt = 1)
                    if res == 2 {
                        let cover_cnt: u64 = handle.coverage.branch as u64;
                        handle.corpus.add_prog(current_prog.clone(), cover_cnt);

                        // TODO: update relations, which is relation to coverage info
                    } else if res == 3 {
                        dbg!("stack failed!");
                    }
                }
                Err(_e) => {
                    RUNNING.store(false, SeqCst);
                }
            }
        }
    }
    println!("{:?} is ended", thread::current());
}
