// Copyright 2022 syzkaller project authors. All rights reserved.
// Use of this source code is governed by Apache 2 LICENSE that can be found in the LICENSE file.

// File autogenerated by genseccomp.py from Android U - edit at your peril!!

const struct sock_filter x86_64_system_filter[] = {
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 0, 0, 98),
    BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 202, 96, 0), // futex
    BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 16, 95, 0), // ioctl
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 186, 47, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 91, 23, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 38, 11, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 24, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 17, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 8, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 6, 89, 88), // read|write|open|close|stat|fstat
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 16, 88, 87), // lseek|mmap|mprotect|munmap|brk|rt_sigaction|rt_sigprocmask|rt_sigreturn
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 21, 87, 86), // pread64|pwrite64|readv|writev
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 35, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 32, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 29, 84, 83), // sched_yield|mremap|msync|mincore|madvise
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 33, 83, 82), // dup
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 37, 82, 81), // nanosleep|getitimer
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 72, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 58, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 44, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 43, 78, 77), // setitimer|getpid|sendfile|socket|connect
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 57, 77, 76), // sendto|recvfrom|sendmsg|recvmsg|shutdown|bind|listen|getsockname|getpeername|socketpair|setsockopt|getsockopt|clone
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 64, 76, 75), // vfork|execve|exit|wait4|kill|uname
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 89, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 79, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 78, 73, 72), // fcntl|flock|fsync|fdatasync|truncate|ftruncate
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 82, 72, 71), // getcwd|chdir|fchdir
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 90, 71, 70), // readlink
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 140, 11, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 112, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 95, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 93, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 92, 66, 65), // fchmod
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 94, 65, 64), // fchown
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 111, 64, 63), // umask|gettimeofday|getrlimit|getrusage|sysinfo|times|ptrace|getuid|syslog|getgid|setuid|setgid|geteuid|getegid|setpgid|getppid
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 137, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 135, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 132, 61, 60), // setsid|setreuid|setregid|getgroups|setgroups|setresuid|getresuid|setresgid|getresgid|getpgid|setfsuid|setfsgid|getsid|capget|capset|rt_sigpending|rt_sigtimedwait|rt_sigqueueinfo|rt_sigsuspend|sigaltstack
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 136, 60, 59), // personality
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 139, 59, 58), // statfs|fstatfs
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 169, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 157, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 155, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 153, 55, 54), // getpriority|setpriority|sched_setparam|sched_getparam|sched_setscheduler|sched_getscheduler|sched_get_priority_max|sched_get_priority_min|sched_rr_get_interval|mlock|munlock|mlockall|munlockall
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 156, 54, 53), // pivot_root
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 167, 53, 52), // prctl|arch_prctl|adjtimex|setrlimit|chroot|sync|acct|settimeofday|mount|umount2
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 179, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 175, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 172, 50, 49), // reboot|sethostname|setdomainname
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 177, 49, 48), // init_module|delete_module
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 180, 48, 47), // quotactl
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 280, 23, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 247, 11, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 217, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 206, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 203, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 201, 42, 41), // gettid|readahead|setxattr|lsetxattr|fsetxattr|getxattr|lgetxattr|fgetxattr|listxattr|llistxattr|flistxattr|removexattr|lremovexattr|fremovexattr|tkill
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 205, 41, 40), // sched_setaffinity|sched_getaffinity
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 211, 40, 39), // io_setup|io_destroy|io_getevents|io_submit|io_cancel
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 233, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 221, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 220, 37, 36), // getdents64|set_tid_address|restart_syscall
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 232, 36, 35), // fadvise64|timer_create|timer_settime|timer_gettime|timer_getoverrun|timer_delete|clock_settime|clock_gettime|clock_getres|clock_nanosleep|exit_group
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 235, 35, 34), // epoll_ctl|tgkill
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 257, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 254, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 251, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 248, 31, 30), // waitid
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 253, 30, 29), // ioprio_set|ioprio_get
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 256, 29, 28), // inotify_add_watch|inotify_rm_watch
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 265, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 262, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 261, 26, 25), // openat|mkdirat|mknodat|fchownat
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 264, 25, 24), // newfstatat|unlinkat
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 279, 24, 23), // linkat|symlinkat|readlinkat|fchmodat|faccessat|pselect6|ppoll|unshare|set_robust_list|get_robust_list|splice|tee|sync_file_range|vmsplice
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 321, 11, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 302, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 285, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 283, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 282, 19, 18), // utimensat|epoll_pwait
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 284, 18, 17), // timerfd_create
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 300, 17, 16), // fallocate|timerfd_settime|timerfd_gettime|accept4|signalfd4|eventfd2|epoll_create1|dup3|pipe2|inotify_init1|preadv|pwritev|rt_tgsigqueueinfo|perf_event_open|recvmmsg
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 314, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 305, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 303, 14, 13), // prlimit64
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 312, 13, 12), // clock_adjtime|syncfs|sendmmsg|setns|getcpu|process_vm_readv|process_vm_writev
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 320, 12, 11), // sched_setattr|sched_getattr|renameat2|seccomp|getrandom|memfd_create
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 434, 5, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 424, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 332, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 329, 8, 7), // bpf|execveat|userfaultfd|membarrier|mlock2|copy_file_range|preadv2|pwritev2
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 333, 7, 6), // statx
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 425, 6, 5), // pidfd_send_signal
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 440, 3, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 438, 1, 0),
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 437, 3, 2), // pidfd_open|clone3|close_range
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 439, 2, 1), // pidfd_getfd
    BPF_JUMP(BPF_JMP | BPF_JGE | BPF_K, 441, 1, 0), // process_madvise
    BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW),
};

#define x86_64_system_filter_size (sizeof(x86_64_system_filter) / sizeof(struct sock_filter))
