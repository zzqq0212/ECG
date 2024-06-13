
const call_t syscalls[] = {
    {"syz_builtin0", 0, {1, }},
    {"syz_builtin1", 0, {1, }},
    {"syz_builtin2", 0, {1, }},
    {"syz_builtin3", 0, {1, }},
    {"syz_builtin4", 0, {1, }},
    {"syz_builtin5", 0, {1, }},
    {"syz_mycall", 0, {}, (syscall_t)syz_mycall},
};