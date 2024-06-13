use std::path::PathBuf;
use std::{env, fs::copy, fs::read_dir};

fn main() -> std::io::Result<()> {
    // load target corpus path
    let mut cur_dir = env::current_dir().unwrap();
    cur_dir.pop();
    cur_dir.push("sys/json/");

    // load os target in dir: sys/json/* into build dir
    let files = read_dir(&cur_dir).unwrap();

    for file in files {
        // load file name
        let file = file.unwrap();
        let file_name = file.file_name().to_str().unwrap().to_owned();

        // load copy-from file path
        let mut target_path = PathBuf::new();
        target_path.push(&cur_dir);
        target_path.push(&file_name);

        // load copy-to file path
        let mut build_path = PathBuf::new();
        build_path.push(env::var("OUT_DIR").unwrap());
        build_path.push(&file_name);

        copy(target_path, build_path)?;
    }

    Ok(())
}
