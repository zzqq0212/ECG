//! Syz-wrapper build script.
//!
//! This build script downloads, patches and builds Syzkaller.
//! The Syzlang description will be dump to json files in `OUT_DIR/sys/json`.
//! All the json format syscall descriptions will be included to source code
//! with `include_str!` macro (see `sys/mod.rs`), so that Healer can load them
//! without further manual efforts.
//! One can also skip this process and provide their own build via some extra env vars.
use std::{
    env::{self},
    fs::{copy, create_dir, read_dir, remove_file, File},
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{exit, Command},
};

/// Revision that the patches can be applied stably.
const STABLE_REVISION: &str = "15195ea3c66cc18f348576f4cfb94d03dd13c747";
const STABLE_CSUM: &str = "a7d44b1533ded0c4bd59217081fa247348e801b389f8e9ac11048ba1104b1134";
const OSES: [&str; 4] = ["ucos", "freertos", "zephyr", "rtthread"];

fn main() {
    let target_os = match env::var("TARGETOS") {
        Ok(v) => v,
        Err(_) => return,
    };
    let source_dir = env::var("SOURCEDIR").unwrap();
    let target_arch = env::var("TARGETARCH").unwrap();

    check_syz_exist();
    if env::var("SKIP_SYZ_BUILD").is_err() && check_syz_exist() == false {
        check_env();
        // Try to patch the latest revision first
        const LATEST_REVISION: &str = "master";
        let syz_dir = download(LATEST_REVISION, None);
        if let Some(sys_dir) = build_syz(syz_dir, &target_os, &source_dir, &target_arch) {
            copy_sys(sys_dir);
        } else {
            eprintln!("failed to patch and build latest revision, failback...");
            let syz_dir = download(STABLE_REVISION, Some(STABLE_CSUM));
            if let Some(sys_dir) = build_syz(syz_dir, &target_os, &source_dir, &target_arch) {
                copy_sys(sys_dir);
                return;
            }
            eprintln!(
                "failed to build and patch Syzkaller with stable revision ({})",
                STABLE_REVISION
            );
            exit(1)
        }
    } else if let Ok(sys_dir) = env::var("SYZ_SYS_DIR") {
        let sys_dir = PathBuf::from(sys_dir);
        copy_sys(sys_dir)
    } else {
        eprintln!("Directory that contains json format Syzlang description should be provided via `SYZ_SYS_DIR` env var, 
        when `SKIP_SYZ_BUILD` env var is set");
        exit(1)
    };
}

fn check_syz_exist() -> bool {
    let out_dir = env::var("OUT_DIR").unwrap();
    let syz_bin_path = PathBuf::from(&format!("{}{}", out_dir, "/../../../syz-bin"));
    return syz_bin_path.exists();
}

/// Check required tool to build Syzkaller
fn check_env() {
    const TOOLS: [(&str, &str); 6] = [
        ("wget", "download syzkaller"),
        ("sha384sum", "check download"),
        ("unzip", "unzip syzkaller.zip"),
        ("patch", "patch patches/*.diff"),
        ("make", "build the syzkaller description and executor"),
        ("go", "build the syzkaller"),
    ];
    let mut missing = false;
    for (tool, reason) in TOOLS.iter().copied() {
        let status = Command::new("which").arg(tool).status().unwrap();
        if !status.success() {
            eprintln!("missing tool {} to {}.", tool, reason);
            missing = true;
        }
    }
    if missing {
        eprintln!("missing tools, please install them first");
        exit(1)
    }
}

fn download(syz_revision: &str, csum: Option<&str>) -> PathBuf {
    let repo_url = format!(
        "https://github.com/google/syzkaller/archive/{}.zip",
        syz_revision
    );
    let target = env::var("OUT_DIR").unwrap();
    let syz_zip = PathBuf::from(&format!("{}/syzkaller-{}.zip", target, syz_revision));
    let syz_dir = format!("{}/syzkaller-{}", target, syz_revision);
    let syz_dir = PathBuf::from(syz_dir);
    let mut need_unzip = true;
    if syz_dir.exists() {
        return syz_dir;
    }

    if syz_zip.exists() {
        let mut need_remove = false;
        if let Some(expected_csum) = csum {
            need_remove = !check_download_csum(&syz_zip, expected_csum);
        } else if try_unzip(&target, &syz_zip) {
            need_unzip = false;
        } else {
            need_remove = true;
        };

        if need_remove {
            remove_file(&syz_zip).unwrap_or_else(|e| {
                eprintln!(
                    "failed to removed broken file({}): {}",
                    syz_zip.display(),
                    e
                );
                exit(1);
            })
        }
    }

    if !syz_zip.exists() {
        println!("downloading syzkaller...");
        let wget = Command::new("wget")
            .arg("-O")
            .arg(syz_zip.to_str().unwrap())
            .arg(&repo_url)
            .output()
            .unwrap_or_else(|e| {
                eprintln!("failed to spawn wget: {}", e);
                exit(1)
            });
        if !wget.status.success() {
            let stderr = String::from_utf8(wget.stderr).unwrap_or_default();
            eprintln!(
                "failed to download syzkaller from: {}, error: {}",
                repo_url, stderr
            );
            exit(1);
        }
        if let Some(csum) = csum {
            if !check_download_csum(&syz_zip, csum) {
                eprintln!("downloaded file {} was broken", syz_zip.display());
                exit(1);
            }
        }
        println!("cargo:rerun-if-changed={}", syz_zip.display());
    }

    if need_unzip && !try_unzip(&target, &syz_zip) {
        eprintln!("failed to unzip the downloaded file: {}", syz_zip.display());
        exit(1);
    }

    assert!(syz_dir.exists());
    println!("cargo:rerun-if-changed={}", syz_dir.display());
    syz_dir
}

fn try_unzip<P: AsRef<Path>>(current_dir: &str, syz_zip: P) -> bool {
    let unzip = Command::new("unzip")
        .current_dir(current_dir)
        .arg(syz_zip.as_ref())
        .output()
        .unwrap_or_else(|e| {
            eprintln!("failed to spawn unzip: {}", e);
            exit(1)
        });
    unzip.status.success()
}

fn check_download_csum<P: AsRef<Path>>(syz_zip: P, expected_csum: &str) -> bool {
    let output = Command::new("sha384sum")
        .arg(syz_zip.as_ref())
        .output()
        .unwrap();
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap_or_default();
        eprintln!("sha384sum failed: {}", stderr);
        exit(1)
    } else {
        let stdout = String::from_utf8(output.stdout).unwrap();
        let cksum = stdout.split(' ').next().unwrap();
        cksum.trim() == expected_csum
    }
}

fn build_syz(
    syz_dir: PathBuf,
    target_os: &String,
    source_dir: &String,
    target_arch: &String,
) -> Option<PathBuf> {
    if !syz_dir.join("bin").exists() {
        // patch syzkaller
        let patch_path = PathBuf::from("./patches/gen.patch");
        copy(&patch_path, &syz_dir.join("gen.patch")).unwrap_or_else(|e| {
            eprintln!(
                "failed to copy patch file '{}': {}",
                patch_path.display(),
                e
            );
            exit(1)
        });

        dbg!(&syz_dir.to_str().unwrap());
        let patch_file_name = syz_dir.join("gen.patch");
        let patch_cmd = Command::new("patch")
            .arg("-p1")
            .stdin(File::open(&patch_file_name).unwrap())
            .current_dir(syz_dir.to_str().unwrap())
            .output()
            .unwrap_or_else(|e| {
                eprintln!("failed to spawn git: {}", e);
                exit(1)
            });
        if !patch_cmd.status.success() {
            let stderr = String::from_utf8(patch_cmd.stderr).unwrap_or_default();
            eprintln!("failde to patch {}: {}", patch_path.display(), stderr);
            return None;
        }
        // copy spec
        for os in OSES {
            let spec_path = PathBuf::from("../sys/");
            let cp_cmd = Command::new("cp")
                .arg("-r")
                .arg(spec_path.join(&os).to_str().unwrap())
                .arg(syz_dir.join("sys"))
                .output()
                .unwrap_or_else(|e| {
                    eprintln!("failed to copy spec file '{}': {}", spec_path.display(), e);
                    exit(1)
                });
            if !cp_cmd.status.success() {
                let stderr = String::from_utf8(cp_cmd.stderr).unwrap_or_default();
                eprintln!(
                    "failed to copy spec file '{}': {}",
                    spec_path.display(),
                    stderr
                );
                return None;
            }
        }

        let targets = vec!["extract", "generate"];
        for target in targets {
            let make = Command::new("make")
                .current_dir(syz_dir.to_str().unwrap())
                .arg(target)
                .arg(format!("{}{}", "TARGETOS=", &target_os))
                .arg(format!("{}{}", "TARGETARCH=", &target_arch))
                .arg(format!("{}{}", "SOURCEDIR=", &source_dir))
                .output()
                .unwrap_or_else(|e| {
                    eprintln!("failed to spawn make: {}", e);
                    exit(1);
                });
            if !make.status.success() {
                let stderr = String::from_utf8(make.stderr).unwrap_or_default();
                eprintln!("failed to make {}: {}", target, stderr);
                return None;
            }
        }
        println!("cargo:rerun-if-changed={}", syz_dir.join("bin").display());
    }

    copy_maked_sys_file(&syz_dir);
    let sys_dir = syz_dir.join("sys").join("json");
    assert!(sys_dir.exists());
    println!("cargo:rerun-if-changed={}", sys_dir.display());
    Some(sys_dir)
}

fn copy_maked_sys_file(syz_dir: &Path) {
    use std::os::unix::fs::symlink;

    let bin_dir = syz_dir.join("bin");
    if !bin_dir.exists() {
        eprintln!("executable files not exist: {}", syz_dir.display());
        exit(1);
    }
    // target/[debug/release]/build/feryr-external/out/..
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_bin = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("syz-bin");
    println!(
        "copy bin from {} to {}...",
        bin_dir.display(),
        out_bin.display()
    );
    if let Err(e) = symlink(&bin_dir, &out_bin) {
        if e.kind() != ErrorKind::AlreadyExists {
            eprintln!(
                "failed to hardlink bin dir from {} to {}: {}",
                bin_dir.display(),
                out_bin.display(),
                e
            );
            exit(1);
        }
    }

    let syscall_header_dir = syz_dir.join("executor").join("syscalls.h");
    let syscall_spec_dir = syz_dir.join("sys");
    let out_spec = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("sys");

    // cp spec
    for os in OSES {
        let cp_cmd = Command::new("cp")
            .arg("-r")
            .arg(out_spec.join(os))
            .arg(&syscall_spec_dir)
            .output()
            .unwrap_or_else(|e| {
                eprintln!(
                    "failed to copy out spec file '{}': {}",
                    syscall_spec_dir.display(),
                    e
                );
                exit(1)
            });
        if !cp_cmd.status.success() {
            let stderr = String::from_utf8(cp_cmd.stderr).unwrap_or_default();
            eprintln!(
                "failed to copy out spec file '{}': {}",
                syscall_spec_dir.display(),
                stderr
            );
            exit(1)
        }
    }

    // cp json
    let json_cp_cmd = Command::new("cp")
        .arg("-r")
        .arg(out_spec.join("json"))
        .arg(&syscall_spec_dir)
        .output()
        .unwrap_or_else(|e| {
            eprintln!(
                "failed to copy out spec file '{}': {}",
                syscall_spec_dir.display(),
                e
            );
            exit(1)
        });
    if !json_cp_cmd.status.success() {
        let stderr = String::from_utf8(json_cp_cmd.stderr).unwrap_or_default();
        eprintln!(
            "failed to copy out spec file '{}': {}",
            syscall_spec_dir.display(),
            stderr
        );
        exit(1)
    }
    // copy(out_spec.join("json"), &syscall_spec_dir).unwrap_or_else(|e| {
    //     eprintln!(
    //         "failed to copy json '{}': {}",
    //         syscall_spec_dir.display(),
    //         e
    //     );
    //     exit(1)
    // });

    // cp syscall header
    let header_cp_cmd = Command::new("cp")
        .arg("-r")
        .arg(&syscall_header_dir)
        .arg(&out_spec)
        .output()
        .unwrap_or_else(|e| {
            eprintln!(
                "failed to copy out spec file '{} to {}': {}",
                syscall_header_dir.display(),
                out_spec.display(),
                e
            );
            exit(1)
        });
    if !header_cp_cmd.status.success() {
        let stderr = String::from_utf8(json_cp_cmd.stderr).unwrap_or_default();
        eprintln!(
            "failed to copy out header file '{} to {}': {}",
            syscall_header_dir.display(),
            out_spec.display(),
            stderr
        );
        exit(1)
    }
    // copy(out_spec.parent().unwrap().join("executor").join("syscalls.h"), &syscall_header_dir).unwrap_or_else(|e| {
    //     eprintln!(
    //         "failed to copy syscall header '{}': {}",
    //         syscall_spec_dir.display(),
    //         e
    //     );
    //     exit(1)
    // });
}

fn copy_sys(sys_dir: PathBuf) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_sys = out_dir.join("sys");
    if let Err(e) = create_dir(&out_sys) {
        if e.kind() != ErrorKind::AlreadyExists {
            eprintln!("failed to create out sys dir: {}", e);
            exit(1)
        }
    }

    for f in read_dir(sys_dir).unwrap().filter_map(|f| f.ok()) {
        let p = f.path();
        if p.is_dir() {
            let out = out_sys.join(p.file_name().unwrap());
            if let Err(e) = create_dir(&out) {
                if e.kind() == ErrorKind::AlreadyExists {
                    continue;
                } else {
                    eprintln!("failed to create out os sys dir: {}", e);
                    exit(1)
                }
            }
            copy_dir_json(&p, &out)
        }
    }
    println!("cargo:rerun-if-changed={}", out_sys.display());
}

fn copy_dir_json<P: AsRef<Path>>(from: P, to: P) {
    for f in read_dir(from.as_ref()).unwrap().filter_map(|f| f.ok()) {
        let p = f.path();
        if let Some(ext) = p.extension() {
            if ext.to_str().unwrap() == "json" {
                let fname = p.file_name().unwrap();
                let to_fname = to.as_ref().join(fname);
                copy(&p, &to_fname).unwrap_or_else(|e| {
                    eprintln!(
                        "failed to copy from {} to {}: {}",
                        p.display(),
                        to_fname.display(),
                        e
                    );
                    exit(1)
                });
            }
        }
    }
}
