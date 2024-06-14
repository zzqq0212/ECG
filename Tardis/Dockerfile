FROM archlinux:latest

RUN echo 'Server = https://mirrors.tuna.tsinghua.edu.cn/archlinux/$repo/os/$arch' > /etc/pacman.d/mirrorlist
RUN pacman -Syu --noconfirm

RUN pacman -S --noconfirm base-devel ed binutils
RUN pacman -S --noconfirm make cmake ninja arm-none-eabi-gcc arm-none-eabi-newlib rustup wget curl git openssh python meson
RUN rustup toolchain install nightly
RUN mkdir -p /root/.cargo
RUN touch /root/.cargo/config
RUN echo -e "[source.crates-io]\nreplace-with = 'tuna'\n [source.tuna] \nregistry = \"https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git\" " >> /root/.cargo/config  
RUN pacman -S --noconfirm clang compiler-rt lld

RUN mkdir -p /workdir

COPY . /fuzzer

WORKDIR /workdir
CMD /bin/bash

