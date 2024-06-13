// AUTOGENERATED FILE
// +build !codeanalysis
// +build !syz_target syz_target,syz_os_ucos,syz_arch_arm

package gen

import . "github.com/google/syzkaller/prog"
import . "github.com/google/syzkaller/sys/ucos"

func init() {
	RegisterTarget(&Target{OS: "ucos", Arch: "arm", Revision: revision_arm, PtrSize: 4, PageSize: 4096, NumPages: 4096, DataOffset: 536870912, LittleEndian: true, ExecutorUsesShmem: false, Syscalls: syscalls_arm, Resources: resources_arm, Consts: consts_arm}, types_arm, InitTarget)
}

var resources_arm = []*ResourceDesc{
	{Name: "DIR", Kind: []string{"DIR"}, Values: []uint64{0}},
	{Name: "dfs_fd_res", Kind: []string{"dfs_fd_res"}, Values: []uint64{0}},
	{Name: "dfs_fd_res1", Kind: []string{"dfs_fd_res1"}, Values: []uint64{0}},
	{Name: "fd", Kind: []string{"fd"}, Values: []uint64{0}},
	{Name: "fd1", Kind: []string{"fd1"}, Values: []uint64{0}},
	{Name: "rt_device_res", Kind: []string{"rt_device_res"}, Values: []uint64{0}},
	{Name: "rt_mempool_res", Kind: []string{"rt_mempool_res"}, Values: []uint64{0}},
	{Name: "stat_res", Kind: []string{"stat_res"}, Values: []uint64{0}},
	{Name: "statfs_res", Kind: []string{"statfs_res"}, Values: []uint64{0}},
	{Name: "str_ptr_res", Kind: []string{"str_ptr_res"}, Values: []uint64{0}},
	{Name: "timer_t_res", Kind: []string{"timer_t_res"}, Values: []uint64{0}},
}

var syscalls_arm = []*Syscall{
	{Name: "access", CallName: "access", Args: []Field{
		{Name: "path", Type: Ref(9)},
		{Name: "amode", Type: Ref(7)},
	}},
	{Name: "closedir", CallName: "closedir", Args: []Field{
		{Name: "d", Type: Ref(21)},
	}},
	{Name: "dfs_filesystem_get_mounted_path", CallName: "dfs_filesystem_get_mounted_path", Args: []Field{
		{Name: "device", Type: Ref(15)},
	}, Ret: Ref(28)},
	{Name: "dfs_init", CallName: "dfs_init"},
	{Name: "dfs_lock", CallName: "dfs_lock"},
	{Name: "dfs_mkfs", CallName: "dfs_mkfs", Args: []Field{
		{Name: "fs_name", Type: Ref(9)},
		{Name: "device_name", Type: Ref(9)},
	}},
	{Name: "dfs_mount", CallName: "dfs_mount", Args: []Field{
		{Name: "device_name", Type: Ref(9)},
		{Name: "path", Type: Ref(9)},
		{Name: "fs_type", Type: Ref(9)},
		{Name: "rwflag", Type: Ref(7)},
		{Name: "data", Type: Ref(16)},
	}},
	{Name: "dfs_unlock", CallName: "dfs_unlock"},
	{Name: "fd_get", CallName: "fd_get", Args: []Field{
		{Name: "fd", Type: Ref(7)},
	}, Ret: Ref(24)},
	{Name: "fd_get$res", CallName: "fd_get", Args: []Field{
		{Name: "fd1", Type: Ref(27)},
	}, Ret: Ref(25)},
	{Name: "fd_is_open", CallName: "fd_is_open", Args: []Field{
		{Name: "name", Type: Ref(9)},
	}},
	{Name: "fd_new", CallName: "fd_new", Ret: Ref(26)},
	{Name: "fd_new$open", CallName: "fd_new", Ret: Ref(26)},
	{Name: "fd_new$res", CallName: "fd_new", Ret: Ref(27)},
	{Name: "fd_put", CallName: "fd_put", Args: []Field{
		{Name: "dfs_fd_res", Type: Ref(17)},
	}},
	{Name: "fd_put$res", CallName: "fd_put", Args: []Field{
		{Name: "dfs_fd_res", Type: Ref(25)},
	}},
	{Name: "fstat", CallName: "fstat", Args: []Field{
		{Name: "fildes", Type: Ref(7)},
		{Name: "buf", Type: Ref(19)},
	}},
	{Name: "fsync", CallName: "fsync", Args: []Field{
		{Name: "fildes", Type: Ref(7)},
	}},
	{Name: "mkdir", CallName: "mkdir", Args: []Field{
		{Name: "path", Type: Ref(9)},
		{Name: "mode", Type: Ref(7)},
	}},
	{Name: "open", CallName: "open", Args: []Field{
		{Name: "name", Type: Ref(9)},
		{Name: "flag", Type: Ref(4)},
	}},
	{Name: "opendir", CallName: "opendir", Args: []Field{
		{Name: "path", Type: Ref(9)},
	}},
	{Name: "read", CallName: "read", Args: []Field{
		{Name: "fd", Type: Ref(26)},
		{Name: "buf", Type: Ref(22)},
		{Name: "len", Type: Ref(7)},
	}},
	{Name: "readdir", CallName: "readdir", Args: []Field{
		{Name: "d", Type: Ref(21)},
	}},
	{Name: "rename", CallName: "rename", Args: []Field{
		{Name: "src", Type: Ref(9)},
		{Name: "new", Type: Ref(9)},
	}},
	{Name: "rewinddir", CallName: "rewinddir", Args: []Field{
		{Name: "d", Type: Ref(21)},
	}},
	{Name: "rmdir", CallName: "rmdir", Args: []Field{
		{Name: "path", Type: Ref(9)},
	}},
	{Name: "rt_free", CallName: "rt_free", Args: []Field{
		{Name: "pmem", Type: Ref(10)},
	}},
	{Name: "rt_malloc", CallName: "rt_malloc", Args: []Field{
		{Name: "data_len", Type: Ref(6)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_memcmp", CallName: "rt_memcmp", Args: []Field{
		{Name: "p1_mem", Type: Ref(11)},
		{Name: "p2_mem", Type: Ref(20)},
		{Name: "size", Type: Ref(5)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_memcpy", CallName: "rt_memcpy", Args: []Field{
		{Name: "pdest", Type: Ref(11)},
		{Name: "psrc", Type: Ref(20)},
		{Name: "size", Type: Ref(5)},
	}},
	{Name: "rt_memmove", CallName: "rt_memmove", Args: []Field{
		{Name: "pdest", Type: Ref(11)},
		{Name: "psrc", Type: Ref(20)},
		{Name: "size", Type: Ref(5)},
	}},
	{Name: "rt_memset", CallName: "rt_memset", Args: []Field{
		{Name: "pmem", Type: Ref(10)},
		{Name: "data_val", Type: Ref(3)},
		{Name: "size", Type: Ref(6)},
	}},
	{Name: "rt_mp_alloc", CallName: "rt_mp_alloc", Args: []Field{
		{Name: "name", Type: Ref(9)},
		{Name: "time", Type: Ref(7)},
	}},
	{Name: "rt_mp_create", CallName: "rt_mp_create", Args: []Field{
		{Name: "name", Type: Ref(9)},
		{Name: "start", Type: Ref(16)},
		{Name: "size", Type: Ref(7)},
		{Name: "blocksize", Type: Ref(7)},
	}},
	{Name: "rt_mp_delete", CallName: "rt_mp_delete", Args: []Field{
		{Name: "name", Type: Ref(9)},
	}},
	{Name: "rt_mp_detach", CallName: "rt_mp_detach", Args: []Field{
		{Name: "rt_mp", Type: Ref(12)},
	}},
	{Name: "rt_mp_free", CallName: "rt_mp_free", Args: []Field{
		{Name: "block", Type: Ref(16)},
	}},
	{Name: "rt_mp_init", CallName: "rt_mp_init", Args: []Field{
		{Name: "rt_mp", Type: Ref(12)},
		{Name: "name", Type: Ref(9)},
		{Name: "start", Type: Ref(16)},
		{Name: "size", Type: Ref(7)},
		{Name: "blocksize", Type: Ref(7)},
	}, Ret: Ref(29)},
	{Name: "rt_realloc", CallName: "rt_realloc", Args: []Field{
		{Name: "pmem", Type: Ref(10)},
		{Name: "size", Type: Ref(5)},
	}},
	{Name: "rt_show_version", CallName: "rt_show_version"},
	{Name: "rt_snprintf", CallName: "rt_snprintf", Args: []Field{
		{Name: "pstr_dest", Type: Ref(9)},
		{Name: "pstr_cat", Type: Ref(32)},
	}, Ret: Ref(32)},
	{Name: "rt_strcasecmp", CallName: "rt_strcasecmp", Args: []Field{
		{Name: "p1_str", Type: Ref(9)},
		{Name: "p2_str", Type: Ref(9)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strcmp", CallName: "rt_strcmp", Args: []Field{
		{Name: "p1_str", Type: Ref(9)},
		{Name: "p2_str", Type: Ref(32)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strdup", CallName: "rt_strdup", Args: []Field{
		{Name: "pstr_dest", Type: Ref(18)},
	}, Ret: Ref(32)},
	{Name: "rt_strlen", CallName: "rt_strlen", Args: []Field{
		{Name: "pstr", Type: Ref(9)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strncmp", CallName: "rt_strncmp", Args: []Field{
		{Name: "p1_str", Type: Ref(9)},
		{Name: "p2_str", Type: Ref(32)},
		{Name: "len_max", Type: Ref(7)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strncpy", CallName: "rt_strncpy", Args: []Field{
		{Name: "p1_str", Type: Ref(9)},
		{Name: "p2_str", Type: Ref(9)},
		{Name: "len_max", Type: Ref(7)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strnlen", CallName: "rt_strnlen", Args: []Field{
		{Name: "pstr", Type: Ref(9)},
		{Name: "len_max", Type: Ref(7)},
	}, Attrs: SyscallAttrs{IgnoreReturn: true}},
	{Name: "rt_strstr$res1", CallName: "rt_strstr", Args: []Field{
		{Name: "pstr", Type: Ref(32)},
		{Name: "pstr_srch", Type: Ref(9)},
	}, Ret: Ref(32)},
	{Name: "rt_system_scheduler_init", CallName: "rt_system_scheduler_init"},
	{Name: "rt_system_scheduler_start", CallName: "rt_system_scheduler_start"},
	{Name: "rt_system_signal_init", CallName: "rt_system_signal_init"},
	{Name: "rt_system_timer_init", CallName: "rt_system_timer_init"},
	{Name: "rt_system_timer_thread_init", CallName: "rt_system_timer_thread_init"},
	{Name: "rt_thread_idle_init", CallName: "rt_thread_idle_init"},
	{Name: "rt_tick_from_millisecond", CallName: "rt_tick_from_millisecond", Args: []Field{
		{Name: "milli", Type: Ref(7)},
	}},
	{Name: "rt_tick_get", CallName: "rt_tick_get"},
	{Name: "rt_tick_increase", CallName: "rt_tick_increase"},
	{Name: "rt_tick_set", CallName: "rt_tick_set", Args: []Field{
		{Name: "tick", Type: Ref(7)},
	}},
	{Name: "rt_timer_control", CallName: "rt_timer_control", Args: []Field{
		{Name: "timer_t", Type: Ref(13)},
		{Name: "cmd", Type: Ref(7)},
		{Name: "arg", Type: Ref(16)},
	}},
	{Name: "rt_timer_create", CallName: "rt_timer_create", Args: []Field{
		{Name: "name", Type: Ref(9)},
		{Name: "timeout", Type: Ref(16)},
		{Name: "para", Type: Ref(16)},
		{Name: "time", Type: Ref(7)},
		{Name: "flag", Type: Ref(8)},
	}, Ret: Ref(33)},
	{Name: "rt_timer_delete", CallName: "rt_timer_delete", Args: []Field{
		{Name: "timer_t", Type: Ref(13)},
	}},
	{Name: "rt_timer_init", CallName: "rt_timer_init", Args: []Field{
		{Name: "timer_t", Type: Ref(13)},
		{Name: "name", Type: Ref(9)},
		{Name: "timeout", Type: Ref(16)},
		{Name: "para", Type: Ref(16)},
		{Name: "time", Type: Ref(7)},
		{Name: "flag", Type: Ref(8)},
	}},
	{Name: "rt_timer_start", CallName: "rt_timer_start", Args: []Field{
		{Name: "timer_t", Type: Ref(13)},
	}, Ret: Ref(33)},
	{Name: "rt_timer_stop", CallName: "rt_timer_stop", Args: []Field{
		{Name: "timer_t", Type: Ref(13)},
	}},
	{Name: "seekdir", CallName: "seekdir", Args: []Field{
		{Name: "d", Type: Ref(21)},
		{Name: "offset", Type: Ref(7)},
	}},
	{Name: "stat", CallName: "stat", Args: []Field{
		{Name: "file", Type: Ref(9)},
		{Name: "buf", Type: Ref(19)},
	}},
	{Name: "statfs", CallName: "statfs", Args: []Field{
		{Name: "file", Type: Ref(9)},
		{Name: "statfs", Type: Ref(14)},
	}, Ret: Ref(31)},
	{Name: "telldir", CallName: "telldir", Args: []Field{
		{Name: "d", Type: Ref(21)},
	}},
	{Name: "unlink", CallName: "unlink", Args: []Field{
		{Name: "pathname", Type: Ref(9)},
	}},
	{Name: "write", CallName: "write", Args: []Field{
		{Name: "fd", Type: Ref(26)},
		{Name: "buf", Type: Ref(16)},
		{Name: "len", Type: Ref(7)},
	}},
}

var types_arm = []Type{
	&BufferType{TypeCommon: TypeCommon{TypeName: "array", TypeAlign: 1, IsVarlen: true}, Kind: 1, RangeEnd: 128},
	&BufferType{TypeCommon: TypeCommon{TypeName: "array", TypeAlign: 1, IsVarlen: true}, Kind: 1, RangeEnd: 256},
	&BufferType{TypeCommon: TypeCommon{TypeName: "void", TypeAlign: 1}, Kind: 1},
	&ConstType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "const", TypeSize: 4, TypeAlign: 4}}},
	&IntType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "int32", TypeSize: 4, TypeAlign: 4}}, Kind: 1, RangeBegin: 1, RangeEnd: 6},
	&IntType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "int32", TypeSize: 4, TypeAlign: 4}}, Kind: 1, RangeEnd: 128},
	&IntType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "int32", TypeSize: 4, TypeAlign: 4}}, Kind: 1, RangeEnd: 256},
	&IntType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "int32", TypeSize: 4, TypeAlign: 4}}},
	&IntType{IntTypeCommon: IntTypeCommon{TypeCommon: TypeCommon{TypeName: "int8", TypeSize: 1, TypeAlign: 1}}},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(8)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(1)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(0)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(29)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(33)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(31)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(28)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(2)},
	&PtrType{TypeCommon: TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Elem: Ref(24)},
	&PtrType{TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Ref(8), 1},
	&PtrType{TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Ref(30), 2},
	&PtrType{TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Ref(0), 1},
	&PtrType{TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Ref(23), 2},
	&PtrType{TypeCommon{TypeName: "ptr", TypeSize: 4, TypeAlign: 4}, Ref(2), 1},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "DIR", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "dfs_fd_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "dfs_fd_res1", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "fd", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "fd1", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "rt_device_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "rt_mempool_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "stat_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "statfs_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "str_ptr_res", TypeSize: 4, TypeAlign: 4}},
	&ResourceType{TypeCommon: TypeCommon{TypeName: "timer_t_res", TypeSize: 4, TypeAlign: 4}},
}

var consts_arm = []ConstValue(nil)

const revision_arm = "4c91fa8d98a2172e44e252b965a5508e9a154892"
