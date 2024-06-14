pub mod defs;

use chrono::{DateTime, Utc};
use clap::{App, Arg};
use feryr_prog::{
    corpus_handle::corpus::CorpusWrapper, corpus_handle::relation::RelationWrapper,
    corpus_handle::target::Target, cover_handle::cover::*,
};
use std::process::Command;
use util::vm::*;
use Default;

#[derive(Debug)]
pub struct FuzzManager {
    pub uptime: DateTime<Utc>,
    pub target: Target,
    pub corpus: CorpusWrapper,
    pub total_exec: usize,
    pub qemu_config: QemuConfig,
    pub last_exec: usize,
    pub relation: RelationWrapper,
    pub coverage: Cover,
}

impl FuzzManager {
    pub fn new() -> Self {
        FuzzManager {
            // presist data
            uptime: chrono::offset::Utc::now(),
            target: Target::default(),
            corpus: CorpusWrapper::default(),
            total_exec: 0,
            qemu_config: QemuConfig::default(),
            last_exec: 0,
            relation: RelationWrapper::default(),
            coverage: Cover::new(),
        }
    }
}

pub fn usage_help() {
    println!("Usage: ./start -config /path");
}

pub fn quit_fuzzer() {
    println!("{}", "quit fuzzer!!!");
    Command::new("kill")
        .arg("9")
        .arg("$(pgrep -f qemu-system")
        .output()
        .expect("failed to kill qemu");
}

pub fn parse_args() {
    let _matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("Real-time Kernel Fuzzer")
        .arg(Arg::with_name(""))
        .arg(
            Arg::with_name("configuration file directory")
                .short("c")
                .long("config")
                .required(true)
                .takes_value(true)
                .value_name("CONFIG_FILE"),
        )
        .arg(Arg::with_name("debug config").short("d").long("debug"))
        .get_matches();
}
