# trace2syz

[![Build Status](https://travis-ci.org/shankarapailoor/trace2syz.svg?branch=master)](https://travis-ci.org/shankarapailoor/trace2syz) [![Go Report Card](https://goreportcard.com/badge/github.com/shankarapailoor/trace2syz)](https://goreportcard.com/report/github.com/shankarapailoor/trace2syz)

Hi! This is tool converts strace output to Syzkaller programs. It is adapted from MoonShine which can be found [here](https://github.com/shankarapailoor/moonshine). 

# Getting Started
The following setup instructions have been tested on Ubuntu 16.04. Let us know if there are issues on other versions or distributions.
## Requirements

### Go
Trace2Syz is written in Go so the first step is to setup Go. You can either follow the below instructions or follow the [Official Go Installation Guide](https://golang.org/doc/install) . 

```bash
$ wget https://dl.google.com/go/go1.10.3.linux-amd64.tar.gz
$ sudo tar -C /usr/local -xzf go1.10.3.linux-amd64.tar.gz
$ export PATH=$PATH:/usr/local/go/bin
$ go version
go version go1.10.3 linux/amd64
```
After installing Go, setup your Go workspace. Your Go workspace is where all Go project binary and source code is stored. By default, Go expects the workspace to be under ```$HOME/go``` so either create the directory ```$HOME/go``` or install to a custom location and set ```$GOPATH```(**Note**: If you have already setup Syzkaller then this step can be skipped since Syzkaller is a Go project)

### Ragel
Trace2Syz uses [ragel](http://www.colm.net/open-source/ragel/) (variation of lex) to scan/parse traces.
```bash
sudo apt-get update
sudo apt-get install ragel
```

### Goyacc
Trace2Syz uses [goyacc](https://godoc.org/golang.org/x/tools/cmd/goyacc) (variation of yacc) to scan/parse traces.
```bash
go get golang.org/x/tools/cmd/goyacc
```
goyacc gets installed in ```$HOME/go/bin``` (or ```$GOPATH/bin``` if workspace is not in the home directory). Make sure this directory is on your $PATH.

```bash
$ export PATH=$PATH:$HOME/go/bin
```

## Build and Run Trace2Syz

### Build
```bash
go get -u -d github.com/shankarapailoor/trace2syz/...
cd $GOPATH/src/github.com/shankarapailoor/trace2syz/
make
```

### Run
Once trace2syz has been successfully built, we can generate seeds for Syzkaller as follows:

```bash
$ ./bin/trace2syz -dir [tracedir] -distill [distillConfig.json]
```
The arguments are explained below:
* ```-dir``` is a directory for traces to be parsed. We have provided a tarball of sample traces on [Google Drive](https://drive.google.com/file/d/1eKLK9Kvj5tsJVYbjB2PlFXUsMQGASjmW/view?usp=sharing) to get started. To run the [example](#example) below, download the tarball, move it to the ```getting-started/``` directory, and unpack. 
* ```-distill``` is a config file that specifies the distillation strategy (e.g. implicit, explicit only). If the traces don't have call coverage information or you simply don't want to distill, then this parameter should be ommitted and trace2syz will generate traces "as is". We have provided an example config under ```getting-started/distill.json```
