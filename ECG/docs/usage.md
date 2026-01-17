## Install Prerequisites

Basic dependencies install (take for example on debain or ubuntu):
``` bash
sudo apt update
sudo apt install make gcc flex bison libncurses-dev libelf-dev libssl-dev
```

### GCC

If your distro's GCC is older, it's preferable to get the latest GCC from [this](https://gcc.gnu.org/) list. Download and unpack into `$GCC`, and you should have GCC binaries in `$GCC/bin/`

>**Ubuntu 20.04 LTS**: You can ignore this section. GCC is up-to-date.

``` bash
ls $GCC/bin/
# Sample output:
# cpp     gcc-ranlib  x86_64-pc-linux-gnu-gcc        x86_64-pc-linux-gnu-gcc-ranlib
# gcc     gcov        x86_64-pc-linux-gnu-gcc-9.0.0
# gcc-ar  gcov-dump   x86_64-pc-linux-gnu-gcc-ar
# gcc-nm  gcov-tool   x86_64-pc-linux-gnu-gcc-nm
```

## Input payload generation

### Install LLVM Compile Chain 
We refer to the [LLVM installation guidances](https://llvm.org/docs/GettingStarted.html).
For example, here we list the automatic installation shell for debain/ubuntu.
```bash
# For convenience there is an automatic installation script available that installs LLVM for you.
# To install the latest stable version:
bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"

# To install a specific version of LLVM:
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh <version number>
# To install all apt.llvm.org packages at once:
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh <version number> all
# or
sudo ./llvm.sh all
```

### Extract Call Graph 
In this step, we need to extract the kernel call graph from kernel source code and api by static analysis.

First, we need to download the kernel source code, for instance, we use the linux kernel version 6.7 to demonstrate the call graph extraction workflow.

``` bash
# Download linux kernel 6.7 
cd ECG
git clone https://github.com/torvalds/linux
cd linux
export Kernel=$pwd
git checkout -f 0dd3ee3
```

``` bash
# After we have the Linux Kernel source code, we need to compile it.
make defconfig  
make kvmconfig
make LLVM=1 -j32
```

Due to size of linux kernel source code is big. For demonstrating the workflow, we just create a `ecg-kernel-control-graph` folder in `ECG` for replacing the real linux kernel folder `linux`. 

```bash
cd ECG/ecg-kernel-control-graph
go run CallGraphBitCode.go -cmd module 

# Run build.sh to generate the .bc file, make sure acquire executable privilege.
chmod +x build.sh
./build.sh

# Use the opt command convert .bc file to .dot file. This is a sample to show the opt operation, and you can replace yourt actual .dot file path.
opt -dot-callgraph-enable-new-pm=0 built-in.bc.callgraph.bc
```
Now, we can acquire the kernel call graph file(built-in.bc.callgraph.dot), then we can extract the targeted module's call graph chain that need contain related system call function.

### Prompt assemble
```bash
# After run convert_prompt.py script to obtain call graph chain of targetd kernel moudule. Then we can obtain assembled prompts file, such as: convert_prompt_fs_processed.txt
chmod +x convert_prompt.py
python3 convert_prompt.py

# We can obtain relevant html file, such as: convert_prompt_fs.processed.txt. Then, we will extract the code from the html file.
chmod +x extract_code.py
python3 extract_code.py /path/to/your/convert_prompt_fs_processed.html
```

### LLM Code Generation 
We obtain the targetd module call graph chains prompts, we will use the LLM to help generate the corresponding C code. we will use the [ollama](https://ollama.com/) to get up and running with large language models, such as: Mixtral8x7b, LLama3-8b and so on. 

```bash
# Install ollama (Linux platform), other platform installation can find in (www.ollama.com/download) page.
curl -fsSL https://ollama.com/install.sh | sh

# Run ollama
ollama serve

# Download Mixtral8x7b or LLama3:8b model
ollama pull mixtral:8x7b
ollama pull llama3:8b

# To execute the automatic script to interact with LLM in ECG folder, it's recommended to run both python code in two tmux session. 

cd ECG/ecg-script
python3 app.py
python3 generate_code.py

# Then, we can get the related LLM's response contains C program from responsed html page.
gcc -o test test.c

# To use strace to obtain the system call trace info, finally copy it to `tracedir folders.
strace -o tracefile -s 65500 -v -xx -f -k /path/to/executable arg1 arg2 .. argN

# To use `moonshine` to get the syzlang trace file. This moonshine/bin is built on Linux kernel x86-64 platform.
cd benchmarks/moonshine 
./moonshine/bin/moonshine -dir [tracedir] -distill [distillCOnfig.json]

# To convert `tracedir` to corpus.db using syz-db. (go run syz-db.go)
cd ECG/tools/syz-db
go build syz-db.go
./tools/syz-db/syz-db pack [tracedir]
```
Finally, we finish the `corpus.db`. Relevant `corpus.db` of experiment can bu used in the ECG.

## Build

### Install golang
We use golang in ECG, so make sure golang is installed before build ECG.

``` bash
wget https://dl.google.com/go/go1.22.4.linux-amd64.tar.gz
tar -xf go1.22.4.linux-amd64.tar.gz
mv go goroot
mkdir gopath
export GOPATH=`pwd`/gopath
export GOROOT=`pwd`/goroot
export PATH=$GOPATH/bin:$PATH
export PATH=$GOROOT/bin:$PATH
```

### Compile Kernel
In here we compile Linux Kernel v6.7 ARM as an example.
First we need to have have a compilable Linux.
```bash
# Download linux kernel 
git clone https://github.com/torvalds/linux
cd linux
export Kernel=$pwd
git checkout -f 0dd3ee3
```

After we have the Linux Kernel ARM, we need to compile it. You will require an ARM64 kernel with gcc plugin support. If not, obtain the ARM64 toolchain from Linaro.Get `gcc-linaro-6.1.1-2016.08-x86_64_aarch64-linux-gnu.tar.xz` from [here](https://releases.linaro.org/components/toolchain/binaries/6.1-2016.08/aarch64-linux-gnu/).
Extract and add its `bin/` to your `PATH`.
If you have another ARM64 toolchain on your machine, ensure that this newly downloaded toolchain takes precedence.

```bash
# Modified configuration
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make defconfig
vim .config
```

``` vim
# modified configuration
CONFIG_KCOV=y
CONFIG_KASAN=y
CONFIG_DEBUG_INFO=y
CONFIG_CMDLINE="console=ttyAMA0"
CONFIG_KCOV_INSTRUMENT_ALL=y
CONFIG_DEBUG_FS=y
CONFIG_NET_9P=y
CONFIG_NET_9P_VIRTIO=y
CONFIG_CROSS_COMPILE="aarch64-linux-gnu-"
```

make it!
``` bash
ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- make -j40
```

Now we should have vmlinux (kernel binary) and Image (packed kernel image):
```bash
ls $KERNEL/vmlinux
$KERNEL/vmlinux

ls $KERNEL/arch/arm/boot/Image
$KERNEL/arch/x86/boot/Image
```

### Prepare Image
We will use buildroot to create the disk image.
You can obtain buildroot from [here](https://buildroot.uclibc.org/download.html).
Extract the tarball and perform a `make menuconfig` inside it.
Choose the following options.

    Target options
	    Target Architecture - Aarch64 (little endian)
    Toolchain type
	    External toolchain - Linaro AArch64
    System Configuration
    [*] Enable root login with password
            ( ) Root password = set your password using this option
    [*] Run a getty (login prompt) after boot  --->
	    TTY port - ttyAMA0
    Target packages
	    [*]   Show packages that are also provided by busybox
	    Networking applications
	        [*] dhcpcd
	        [*] iproute2
	        [*] openssh
    Filesystem images
	    [*] ext2/3/4 root filesystem
	        ext2/3/4 variant - ext3
	        exact size in blocks - 6000000
	    [*] tar the root filesystem

Run `make`. After the build, confirm that `output/images/rootfs.ext3` exists.

If you're expreriencing a very slow sshd start up time with arm64 qemu running on x86, the reason is probably low entropy and it be "fixed" with installing `haveged`. It can be found in the buildroot `menuconfig`:

```
    Target packages
	    Miscellaneous
	        [*] haveged
```

### Compile ECG
```bash
cd ECG
CC=gcc-linaro-6.3.1-2017.05-x86_64_aarch64-linux-gnu/bin/aarch64-linux-gnu-g++
make TARGETARCH=arm64
```
As the result compiled binaries should appear in the bin/ dir.

### Build qemu for ARM64
If the build was successful, you should have a `aarch64-softmmu/qemu-system-aarch64` binary.
Obtain the QEMU source from git or from the latest source release.
``` bash
git clone https://github.com/qemu/qemu.git
cd qemu
./configure
make -j32
```

Make sure the kernel boots and sshd starts:
``` bash 
/path/to/aarch64-softmmu/qemu-system-aarch64 \
      -machine virt \
      -cpu cortex-a57 \
      -nographic -smp 1 \
      -hda /path/to/rootfs.ext3 \
      -kernel /path/to/arch/arm64/boot/Image \
      -append "console=ttyAMA0 root=/dev/vda oops=panic panic_on_warn=1 panic=-1 ftrace_dump_on_oops=orig_cpu debug earlyprintk=serial slub_debug=UZ" \
      -m 2048 \
      -net user,hostfwd=tcp::10023-:22 -net nic
```
At this point, you should be able to see a login prompt.

## Set up the QEMU disk

Now that we have a shell, let us add a few lines to existing init scripts so that they are executed each time ecg brings up the VM.

At the top of /etc/init.d/S50sshd add the following lines:

``` vim
ifconfig eth0 up
dhcpcd
mount -t debugfs none /sys/kernel/debug
chmod 777 /sys/kernel/debug/kcov
```

Comment out the line

``` vim
/usr/bin/ssh-keygen -A
```

Next we set up ssh. Create an ssh keypair locally and copy the public key to `/authorized_keys` in `/`. Ensure that you do not set a passphrase when creating this key.

Open `/etc/ssh/sshd_config` and modify the following lines as shown below.

``` vim
PermitRootLogin yes
PubkeyAuthentication yes
AuthorizedKeysFile      /authorized_keys
PasswordAuthentication yes
```
Reboot the machine, and ensure that you can ssh from host to guest as.
``` bash
ssh -i /path/to/id_rsa root@localhost -p 10023
```

To kill the running QEMU instance press Ctrl+A and then X or run:
``` bash
kill $(cat vm.pid)
```

If QEMU works, the kernel boots and ssh succeeds, we can shutdown QEMU and try to run `ECG`.

## Usage

Now we can start to prepare a __config.json__ file. Move this __config.json__ to `ECG` directory.

``` json 
{
    "name": "QEMU-aarch64",
    "target": "linux/arm64",
    "http": ":56700",
    "workdir": "/path/to/a/dir/to/store/syzkaller/corpus",
    "kernel_obj": "/path/to/linux/build/dir",
    "syzkaller": "/path/to/syzkaller/arm64/",
    "image": "/path/to/rootfs.ext3",
    "sshkey": "/path/to/id_rsa",
    "procs": 8,
    "type": "qemu",
    "vm": {
        "count": 1,
        "qemu": "/path/to/qemu-system-aarch64",
        "cmdline": "console=ttyAMA0 root=/dev/vda",
        "kernel": "/path/to/Image",
        "cpu": 2,
        "mem": 2048
    }
}

```
Finally, to copy the `corpus/corpus.db` in `ECG`. 

```bash
cd ECG
cp ../corpus/corpus.db .
```

Now we can run it.

``` bash
./bin/syz-manager -config config.json
```