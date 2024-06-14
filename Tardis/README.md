## Tardis -- Embedded OS Fuzzer 
Tardis is a coverage guided kernel fuzzer specially designed for Embedded Operating System fuzzing.

Like general kernel fuzzers, Tardis works on both the host and guest sides to effectively conduct the coverage-guided fuzzing.

On the host side, the primary responsibility is to manage the whole fuzzing process, e.g., test case generation and coverage analysis.
It uses the syscall information provided by the [Syzlang](https://github.com/google/syzkaller/blob/master/docs/syscall_descriptions.md) description to generate sequences of system calls during the test case generation.
Then, during the execution, it collects the runtime coverage and any crashes; based on this feedback information, it saves interesting test cases and mutates them in later execution.

To communicate between the host and the guest, Tardis modified [QEMU](./tardis_external/patches/qemu_shm.patch). Using the shared memory mechanism, Tardis can mmap certain QEMU memory, so that QEMU  can directly expose its internal space, where the target OS's memory locates. Hence, Tardis can directly access the OS's memory space from the host side. Any data I/O like writing test cases or reading coverage information would be transparent to the target OS.

The instrumentation mechanism is implemented upon Clang's [SanitizerCoverage](https://clang.llvm.org/docs/SanitizerCoverage.html).
In detail, Tardis reserves a data buffer to store the coverage information.
Then the independent coverage collection mechanism contains a coverage initialization function that can iterate every basic block of the target OS and assign them with an ID number. Also, it has a coverage collection function that can collect the coverage information during runtime.

By far, Tardis can support: [RT-Thread](https://github.com/RT-Thread/rt-thread), [FreeRTOS](https://github.com/FreeRTOS/FreeRTOS), [UC/OS](https://github.com/weston-embedded/uC-OS3), and [Zephyr](https://github.com/zephyrproject-rtos/zephyr) fuzz testing.


## Build Tardis
Tardis is written in pure [rust](https://www.rust-lang.org/), except for some patching code. Therefore, rust toolchain should be installed first.
```shell
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
> rustc --version # check install
```

To use the Syzlang descriptions, Tardis needs to install Syzkaller and add [patches](./tardis_external/patches/syz-gen.patch) to the source code, which may require the [Golang](https://go.dev/) environment and increase the build time. Therefore, the build tool required by Syzkaller needs to be installed, e.g., Golang compiler with GO111MODULE GCC 6.1.0 or later. Here, some Syzlang examples for OS fuzzing are provided to facilitate the preparation time: [RT-Thread](./sys/rtthread/)/[UCOS](./sys/ucos/)/[FreeRTOS](./sys/freertos/)/[Zephyr](./sys/zephyr/). 

Once all the necessary tools have been installed, Tardis can be easily built using the following command: 

```shell
> cargo build --release
```

Finally, Tardis can be found in the target/release directory.

## Use Tardis

Overall, fuzzing with Tardis requires two steps: (1) compile the OS and (2) start Tardis.

For the first step, Tardis uses Clang to compile the OS. Here, we use UCOS as an example.
First, we need to have the OS source code. Then we need to configure the OS compile file; we use this [patch](./tardis_external/patches/uCOS-Makefile.patch) as an example. 
Last we can compile the OS by ```make``` command, and have the OS image file.

After we have the OS image ready, we can use our QEMU to test whether it can boot successfully:

```shell
 $(path_to_qemu_build)/qemu-system-arm  -machine mcimx6ul-evk -kernel $(path_to_os_image)/OS.elf -nographic
```

Finally, we need to prepare a [config](./tardis_external/arm-qemu-gcc.json) file to provide Tardis with necessary boot parameters, and we can start the fuzzer with:

```shell
./target/release/fuzzer -config $(path_to_config)
```

If everything works ok, you'll see following log:
```

████████╗ █████╗ ██████╗ ██████╗ ██╗███████╗
╚══██╔══╝██╔══██╗██╔══██╗██╔══██╗██║██╔════╝
   ██║   ███████║██████╔╝██║  ██║██║███████╗
   ██║   ██╔══██║██╔══██╗██║  ██║██║╚════██║
   ██║   ██║  ██║██║  ██║██████╔╝██║███████║
   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚═╝╚══════╝
[2022-05-25][13:16:39]: init fuzz manager
[2022-05-25][13:16:39]: parse config file: config/arm-qemu-gcc.json
[2022-05-25][13:16:39]: load target ucos/arm
[2022-05-25][13:16:39]: init relation table
[2022-05-25][13:16:39]: initialize all vms
[2022-05-25][13:16:39]: load corpus from path , len is 0
[2022-05-25][13:16:39]: work dir is  fuzz/exp/RTOS/fuzz-loop-Brv2gNa584
[2022-05-25][13:16:39]: ThreadId(4) is running
...
```

Bug find by tardis can be find at this [link](./tardis_external/bugs.md)