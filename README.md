#  ECG - Augmenting Embedded Operating System Fuzzing via LLM-based Corpus Generation

`ECG` is a tool that augments embedded operating system fuzzing via LLM-based corpus generation.

<!-- 
Table of Contents
- [1. Linux Kernel Enriched Corpus Contructed by  leveraging exploits and PoCs for Fuzzers](#1-linux-kernel-enriched-corpus-for-fuzzers)
  - [1.1. Using Enriched corpus with Syzkaller](#11-using-enriched-corpus-with-syzkaller)
  - [1.2. Corpus Construction](#14-diy)
    - [1.2.1. Collecting exploits Manually](#141-fetching-corpus-manually)
    - [1.2.2. Generating corpus.db File](#142-generating-corpusdb-file)
  - [1.3. Results](#16-results)
    - [1.3.1. Coverage over time](#161-coverage-over-time)
    - [1.3.2. CVEs:](#164-cves)
    - [1.3.3. New Bugs Reported:](#165-new-bugs-reported) -->

# 1. Augmenting Embedded Operating System Fuzzing via LLM-based Corpus Generation

Documentation for using and generating the Enriched corpus provided here.


## 1.1. Using generated input payload by LLM with Syzkaller

The latest copy of the Corpus file [corpus.db](https://github.com/zzqq0212/Sunflower/releases/download/latest/corpus.db) is available in the releases for this repository. Meanwhile, The explotis-datas folder contains the exploits raw data. The file folder contains some trace examples that we obtained by executing the compiled expoits program with the strace tool.

We use syzkaller for kernel fuzzing. [Syzkaller](https://github.com/google/syzkaller) is an unsupervised coverage-guided kernel fuzzer.

### How to use syzkaller

### Download and Running

Download it to [syzkaller](https://github.com/google/syzkaller) workdir and start syzkaller.

```
cd syzkaller
mkdir workdir
cd workdir
wget https://github.com/zzqq0212/Sunflower/releases/download/latest/corpus.db
cd ..
./bin/syz-manager -config my.cfg
```

```
my.cfg sample:

{
	"target": "linux/amd64",
	"http": "myhost.com:56741",
	"workdir": "/syzkaller/workdir",
	"kernel_obj": "/linux/",
	"image": "./testdata/wheezy.img",
	"syzkaller": "./testdata/syzkaller",
	"disable_syscalls": ["keyctl", "add_key", "request_key"],
	"suppressions": ["some known bug"],
	"procs": 4,
	"type": "qemu",
	"vm": {
		"count": 16,
		"cpu": 2,
		"mem": 2048,
		"kernel": "/linux/arch/x86/boot/bzImage",
		"initrd": "linux/initrd"
	}
}
```



The `syz-manager` process will wind up VMs and start fuzzing in them.
Found crashes, statistics and other information is exposed on the HTTP address specified in the manager config.

## Crashes

Once syzkaller detected a kernel crash in one of the VMs, it will automatically start the process of reproducing this crash (unless you specified `"reproduce": false` in the config).
By default it will use 4 VMs to reproduce the crash and then minimize the program that caused it.
This may stop the fuzzing, since all of the VMs might be busy reproducing detected crashes.

If a reproducer is successfully found, it can be generated in one of the two forms: syzkaller program or C program.
Syzkaller always tries to generate a more user-friendly C reproducer, but sometimes fails for various reasons (for example slightly different timings).


## 1.2. Corpus Construction

### 1.2.1. Collecting expoloits tracefiles

you can show the files folder that contains the collected exploits's tracefile by converting the exploits's compiling and executing.

### 1.2.2. Generating corpus.db File

you can use syz-db.go pack from syzkaller when you have a collection of syz programs that need to be converted to a syzkaller comptaible corpus.db file.

## 1.3. Results

1.Host Machine System Configuration
   ```
  CPU: 128 core
  Memory: 32GB
  GPU: 2 Tesla V100S-PCIE-32GB
  Ubuntu 22.04.4 LTS jammy 
 ```
2.Virtal Machine Resource Configration
  ```
  2 core CPU + 2GB Memory
  ```
3.Test targeted Linux Version

We chose Linux kernel v5.15, v6.1, v6.3.4, and v6.5 as our test kernel targets. In detail, the Linux v6.5 is the latest release version when we were conducting experiments. Each version of the kernel uses the same compilation configuration, while KCOV and KASAN options are enabled in order to collect code coverage and detect memory errors. When setting up the KCSAN configuration, the same configuration is  used in the control test. 


### 1.3.1. Coverage over time 
10 VM (2vCPU and 2G RAM) average for 28 hours.

![image](https://github.com/zzqq0212/ECG/blob/main/files/emsoft_24_cov.png)  

### 1.3.2. New Bugs Reported Detailed Description:
| \#                    | Modules               | Versions           | Locations                                          | Bug Types         |
|-----------------------|-----------------------|--------------------|----------------------------------------------------|-------------------|
| 1                     | fs/buffer             | RT-Linux 6.7       | mark\_buffer\_dirty                                | logic error       |
| 2  | drivers/pci           | RT-Linux 6.7       | vga\_put                                           | logic error       |
| 3                     | kernel/sched          | RT-Linux 6.7       | select\_task\_rq\_fair                             | deadlock          |
| 4  | mm/filemap            | RT-Linux 6.7       | filemap\_fault / page\_add\_file\_rmap             | data race         |
| 5                     | drivers/net/          | RT-Linux 6.7       | e1000\_update\_stats                               | memory corruption |
| 6  | fs/inode              | RT-Linux 6.7       | inode\_update\_timestamps                          | data race         |
| 7                     | kernel/kprobes        | RT-Linux 6.7       | arch\_adjust\_kprobe\_addr                         | logic error       |
| 8  | fs/dcache             | RT-Linux 6.7       | d\_splice\_alias                                   | data race         |
| 9                     | fs/ext4               | RT-Linux 6.7       | \_\_ext4\_new\_inode /\_find\_next\_zero\_bit      | data race         |
| 10 | drivers/e1000         | RT-Linux 6.7       | e1000\_clean                                       | data race         |
| 11                    | lib/find\_bit         | RT-Linux 6.7       | \_find\_first\_bit                                 | data race         |
| 12 | kernel/sched          | RT-Linux 6.8       | \_\_wake\_up\_common                               | null-ptr defer    |
| 13                    | lib/kasprintf         | RT-Linux 6.8       | kvasprintf                                         | logic error       |
| 14 | kernel/events         | RT-Linux 6.8       | perf\_cgroup\_switch                               | logic error       |
| 15                    | fs/inode              | RT-Linux 6.8       | generic\_update\_time / inode\_needs\_update\_time | data race         |
| 16 | fs/ext4               | RT-Linux 6.8       | generic\_write\_end / mpage\_submit\_folio         | data race         |
| 17                    | fs/kernfs             | RT-Linux 6.8       | kernfs\_dop\_revalidate                            | memory corruption |
| 18 | fs/ext4               | RT-Linux 6.8       | ext4\_split\_extent\_at                            | memory corruption |
| 19                    | arch/x86/lib          | RT-Linux 6.8       | memcpy\_orig                                       | out-of-bounds     |
| 20 | kernel/events         | RT-Linux 6.8       | free\_event                                        | logic error       |
| 21                    | drivers/scsi          | RT-Linux 6.8       | \_\_bitmap\_weight /scsi\_device\_unbusy           | data race         |
| 22 | kernel/rcu            | RT-Linux 6.8       | \_\_call\_rcu\_common /mas\_walk                   | data race         |
| 23                    | mm/swap               | RT-Linux 6.8       | \_\_folio\_end\_writeback / lru\_add\_fn           | data race         |
| 24 | arch/x86/lib          | RT-Linux 6.8       | memmove                                            | memory corruption |
| 25                    | arch/x86/events/intel | RT-Linux 6.8       | intel\_pmu\_lbr\_counters\_reorder                 | logic error       |
| 26 | arch/x86/kernel       | RT-Linux 6.8       | deref\_stack\_reg                                  | logic error       |
| 27                    | arch/x86/kernel       | RT-Linux 6.8       | \_\_orc\_find                                      | memory leak       |
| 28 | net/9p                | RT-Linux 6.8       | p9pdu\_readf                                       | memory leak       |
| 29                    | arch/x86/lib          | OpenWrt 5.15       | memset\_erms                                       | logic error       |
| 30 | kernel/smp            | OpenWrt 5.15       | smp\_call\_function\_single                        | logic error       |
| 31                    | arch/arm64/kvm        | RaspberryPi OS 6.7 | kvm\_init\_stage2\_mmu                             | memory leak       |
| 32 | arch/arm64/kvm        | RaspberryPi OS 6.7 | kvm\_age\_gfn                                      | logic error       |