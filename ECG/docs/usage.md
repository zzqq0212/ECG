## 1. Input payload generate

### Install LLVM Compile Chain 
We refer to the [LLVM installation guidances](https://llvm.org/docs/GettingStarted.html).
For example, here we list the automatic installation for debain/ubuntu.
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
```bash
First, we need to download the kernel source code, for instance, we use the linux kernel version 6.7 to demonstrate the call graph extraction workflow.

# download linux kernel 6.7 
cd ECG
git clone https://github.com/torvalds/linux
cd linux
export Kernel=$pwd
git checkout -f 0dd3ee3

# after we have the Linux Kernel source code, we need to compile it.
``` bash
make defconfig  
make kvmconfig
make olddefconfig
make LLVM=1 -j32
```
<!-- ``` vim
# vim .config

CONFIG_PREEMPT=y
CONFIG_PREEMPT_RT_BASE=y
CONFIG_HAVE_PREEMPT_LAZY=y
CONFIG_PREEMPT_LAZY=y
CONFIG_PREEMPT_RT_FULL=y
CONFIG_PREEMPT_COUNT=y
CONFIG_KCOV=y 
CONFIG_DEBUG_INFO=y 
CONFIG_KASAN=y
CONFIG_KASAN_INLINE=y 
CONFIG_CONFIGFS_FS=y
CONFIG_SECURITYFS=y
```

``` vim
# vim Makefile to add the below line code after line:

KBUILD_CFLAGS += -Wframe-larger-than=4096
``` -->
Due to size of linux kernel source code is big. For demonstrating the workflow, we just create a `linux` folder in `ECG` for replacing the real linux kernel folder `linux`. 

```bash
cd ECG/linux
go run KernelBitCode.go

# run build.sh to generate the .bc file, make sure acquire `root` privilege.
chmod +x build.sh
./build.sh

# Use the opt command convert .bc file to .dot file. This is a sample to show the opt operation. you can replace yourt actual .dot file path.
opt -dot-callgraph-enable-new-pm=0 built-in.bc.callgraph.bc
```
Now, we can acquire the kernel call graph file(built-in.bc.callgraph.dot), then we can extract the targeted module's call graph chain that need contain related system call function.

### Prompt assemble
```bash
# After run convert_prompt.py script to obtain call graph chain of targetd kernel moudule. Then we can obtain assembled prompts file, such as: convert_prompt_fs_processed.txt
chmod +x convert_prompt.py
python3 convert_prompt.py
```

### LLM Code Generation 
We obtain the targetd module call graph chains prompts, we will use the LLM to help generate the corresponding C code. we will use the [ollama](https://ollama.com/) to get up and running with large language models, such as: Mixtral8x7b, LLama3-8b and so on. 

```bash
# install ollama (Linux platform), other platform installation can find in (www.ollama.com/download) page.
curl -fsSL https://ollama.com/install.sh | sh

# run ollama
ollama serve

# download Mixtral8x7b or LLama3:8b model
ollama pull mixtral:8x7b
ollama pull llama3:8b

# To execute the automatic script to interact with LLM in ECG folder, it's recommended to run both python code in two tmux session. 

cd ECG
python3 app.py
python3 generate_html.py

# Then, we can get the related LLM's response contains C program from responsed html page.
gcc -o test test.c

# use strace to obtain the system call trace info, finally copy it to `tracedir folders.
strace -o tracefile -s 65500 -v -xx -f -k /path/to/executable arg1 arg2 .. argN

# use `moonshine` to get the syzlang trace files.
./moonshine/bin/moonshine -dir [tracedir] -distill [distillCOnfig.json]

# convert `tracedir` to corpus.db using syz-db. (go run syz-db.go)
./tools/syz-db/syz-db pack [tracedir]

```
Finally, we finish the `corpus.db`. Relevant `corpus.db` of experiment can bu used in the ECG.

## 2. Tool build

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

### Prepare Kernel(similar with "Extract Call Graph" step to compile Linux kernel)
In here we use Linux Kernel(Enable Real time Config) v6.7 as an example.
First we need to have have a compilable Linux
```bash
# download linux kernel 
git clone https://github.com/torvalds/linux
cd linux
export Kernel=$pwd
git checkout -f 0dd3ee3

After we have the Linux Kernel, we need to compile it.
``` bash
# modified configuration
make defconfig  
make kvmconfig
vim .config
```

``` vim
# modified configuration
CONFIG_PREEMPT=y
CONFIG_PREEMPT_RT_BASE=y
CONFIG_HAVE_PREEMPT_LAZY=y
CONFIG_PREEMPT_LAZY=y
CONFIG_PREEMPT_RT_FULL=y
CONFIG_PREEMPT_COUNT=y
CONFIG_KCOV=y 
CONFIG_DEBUG_INFO=y 
CONFIG_KASAN=y
CONFIG_KASAN_INLINE=y 
CONFIG_CONFIGFS_FS=y
CONFIG_SECURITYFS=y
```

make it!
``` bash
make olddefconfig
make -j32
```

Now we should have vmlinux (kernel binary) and bzImage (packed kernel image):
```bash
$ ls $KERNEL/vmlinux
$KERNEL/vmlinux
$ ls $KERNEL/arch/x86/boot/bzImage
$KERNEL/arch/x86/boot/bzImage
```

### Prepare Image
```bash 
sudo apt-get install debootstrap 
export IMAGE=$pwd
cd $IMAGE/
wget https://raw.githubusercontent.com/google/syzkaller/master/tools/create-image.sh -O create-image.sh
chmod +x create-image.sh
./create-image.sh
```
Now we have a image stretch.img and a public key.

### Build ECG
```bash
cd ECG
make
```
As the result compiled binaries should appear in the bin/ dir.

### Ready QEMU
Install QEMU:
``` bash
sudo apt-get install qemu-system-x86
```
Make sure the kernel boots and sshd starts:
``` bash 
qemu-system-x86_64 \
	-m 2G \
	-smp 2 \
	-kernel $KERNEL/arch/x86/boot/bzImage \
	-append "console=ttyS0 root=/dev/sda earlyprintk=serial net.ifnames=0" \
	-drive file=$IMAGE/stretch.img,format=raw \
	-net user,host=10.0.2.10,hostfwd=tcp:127.0.0.1:10021-:22 \
	-net nic,model=e1000 \
	-enable-kvm \
	-nographic \
	-pidfile vm.pid \
	2>&1 | tee vm.log
```
see if ssh works
``` bash 
ssh -i $IMAGE/stretch.id_rsa -p 10021 ``-o "StrictHostKeyChecking no" 
```

To kill the running QEMU instance press Ctrl+A and then X or run:
``` bash
kill $(cat vm.pid)
```
If QEMU works, the kernel boots and ssh succeeds, we can shutdown QEMU and try to run ECG.

## 3. Usage

Now we can start to prepare a __config.json__ file. Move this __config.json__ to `ECG` directory.

``` json 
{
        "target": "linux/amd64",
        "http": "127.0.0.1:56295",
        "workdir": "./workdir",
        "cover": false,
        "kernel_obj": "$(Kernel)/vmlinux",
        "image": "$(image)/stretch.img",
        "sshkey": "$(image)/stretch.id_rsa",
        "syzkaller": "$pwd",
        "procs": 2,
        "type": "qemu",
        "vm": {
                "count": 2,
                "kernel": "$(Kernel)/bzImage",
                "cpu": 2,
                "mem": 4096
        }


```
Finally, to copy the `corpus/corpus.db` in `ECG`. 

```bash
cd ECG
cp ../corpus/corpus.db .
```

```
Now we can run it.

``` bash
./bin/syz-manager -config config.json
```