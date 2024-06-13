typedef unsigned long long uint64;
typedef unsigned int uint32;
typedef unsigned short uint16;
typedef unsigned char uint8;


static uint8 _fuzz_prog_buffer[4096] = {0};

const int kMaxArgs = 9;
const int kMaxInput = 4 << 20; // keep in sync with prog.ExecBufferSize
int kMaxCommands = 1000; // prog package knows about this constant (prog.execMaxCommands)

static unsigned long long procid;

const uint64 instr_eof = -1;
const uint64 instr_copyin = -2;
const uint64 instr_copyout = -3;
const uint64 instr_setprops = -4;

const uint64 arg_const = 0;
const uint64 arg_addr32 = 1;
const uint64 arg_addr64 = 2;
const uint64 arg_result = 3;
const uint64 arg_data = 4;
const uint64 arg_csum = 5;

const uint64 binary_format_native = 0;
const uint64 binary_format_bigendian = 1;
const uint64 binary_format_strdec = 2;
const uint64 binary_format_strhex = 3;
const uint64 binary_format_stroct = 4;


#define SYZ_PAGE_SIZE 2048
#define SYZ_NUM_PAGES 8192
#define SYZ_DATA_OFFSET 536870912

typedef struct call_attrs_t { 
	uint64_t disabled;
	uint64_t timeout;
	uint64_t prog_timeout;
	uint64_t ignore_return;
	uint64_t breaks_returns;
	uint64_t no_generate;
	uint64_t no_minimize;
} call_attrs_t;

typedef intptr_t(*syscall_t)(intptr_t, intptr_t, intptr_t, intptr_t, intptr_t, intptr_t, intptr_t, intptr_t, intptr_t);


typedef struct call_props_t { 
	int fail_nth;
	bool async;
	int rerun;
} call_props_t;

typedef struct call_t {
	const char* name;
	int sys_nr;
	call_attrs_t attrs;
	syscall_t call;
} call_t;

#define read_call_props_t(var, reader) { \
	(var).fail_nth = (int)(reader); \
	(var).async = (bool)(reader); \
	(var).rerun = (int)(reader); \
}

#include "common_freertos.h"
#include "syscalls.h"



const uint64 no_copyout = -1;

static struct res_t {
	bool executed;
	uint64 val;
} results[1000];

inline uint64 BITMASK(uint64 shift, uint64 len) {
    return ((1ULL << len) - 1) << shift;
}

uint64 read_input(uint8** input_posp, bool peek)
{
	uint64 v = 0;
	unsigned shift = 0;
	uint8* input_pos = *input_posp;
	for (int i = 0;; i++, shift += 7) {
		const int maxLen = 10;
		if (i == maxLen) {}
		if (input_pos >= _fuzz_prog_buffer + kMaxInput) {}
		uint8 b = *input_pos++;
		v |= (uint64)(b & 0x7f) << shift;
		if (b < 0x80) {
			if (i == maxLen - 1 && b > 1) {}
			break;
		}
	}
	if (v & 1)
		v = ~(v >> 1);
	else
		v = v >> 1;
	if (!peek)
		*input_posp = input_pos;
	return v;
}



uint64 swap(uint64 v, uint64 size, uint64 bf)
{
	if (bf == binary_format_native)
		return v;
	if (bf != binary_format_bigendian) {
		return v;
	}
	switch (size) {
	case 2:
		return htobe16(v);
	case 4:
		return htobe32(v);
	case 8:
		return htobe64(v);
	default:
		return 0;
	}
}

uint8_t swap_uint8(uint8_t v) {
    return v;  // Return the value unchanged because it's a single byte
}

uint16_t swap_uint16(uint16_t v) {
    return (v >> 8) | (v << 8);
}

uint32_t swap_uint32(uint32_t v) {
    return ((v >> 24) & 0xff) | ((v << 8) & 0xff0000) | ((v >> 8) & 0xff00) | ((v << 24) & 0xff000000);
}

uint64_t swap_uint64(uint64_t v) {
    v = (v >> 32) | (v << 32);
    v = ((v & 0xffff0000ffff0000ULL) >> 16) | ((v & 0x0000ffff0000ffffULL) << 16);
    return ((v & 0xff00ff00ff00ff00ULL) >> 8) | ((v & 0x00ff00ff00ff00ffULL) << 8);
}

void copyin_int16(char* addr, uint16_t val, uint64_t bf, uint64_t bf_off, uint64_t bf_len) {
    uint16_t x = *(uint16_t*)addr;
    if (bf_off == 0 && bf_len == 0) {
        *(uint16_t*)addr = swap_uint16(val);
        return;
    }
    uint64_t shift = bf_off;
    x = (x & ~BITMASK(shift, bf_len)) | ((val << shift) & BITMASK(shift, bf_len));
    *(uint16_t*)addr = swap_uint16(x);
}

void copyin_int32(char* addr, uint32_t val, uint64_t bf, uint64_t bf_off, uint64_t bf_len) {
    uint32_t x = *(uint32_t*)addr;
    if (bf_off == 0 && bf_len == 0) {
        *(uint32_t*)addr = swap_uint32(val);
        return;
    }
    uint64_t shift = bf_off;
    x = (x & ~BITMASK(shift, bf_len)) | ((val << shift) & BITMASK(shift, bf_len));
    *(uint32_t*)addr = swap_uint32(x);
}

void copyin_int64(char* addr, uint64_t val, uint64_t bf, uint64_t bf_off, uint64_t bf_len) {
    uint64_t x = *(uint64_t*)addr;
    if (bf_off == 0 && bf_len == 0) {
        *(uint64_t*)addr = swap_uint64(val);
        return;
    }
    uint64_t shift = bf_off;
    x = (x & ~BITMASK(shift, bf_len)) | ((val << shift) & BITMASK(shift, bf_len));
    *(uint64_t*)addr = swap_uint64(x);
}


void copyin_int8(char* addr, uint64 val, uint64 bf, uint64 bf_off, uint64 bf_len) {
    uint8 x = *(uint8*)addr;
    if (bf_off == 0 && bf_len == 0) {
        *(uint8*)addr = (uint8)swap_uint8(val);
        return;
    }
    // Define BITMASK macro or function if not already defined
    const uint64 shift = bf_off;  // Assuming little-endian as an example
    x = (x & ~((1U << bf_len) - 1) << shift) | ((val << shift) & ((1U << bf_len) - 1) << shift);
    *(uint8*)addr = (uint8)swap_uint8(x);
}

void copyin(char* addr, uint64 val, uint64 size, uint64 bf, uint64 bf_off, uint64 bf_len) {
    switch (size) {
        case 1:
            copyin_int8(addr, val, bf, bf_off, bf_len);
            break;
        case 2:
            copyin_int16(addr, val, bf, bf_off, bf_len);
            break;
        case 4:
            copyin_int32(addr, val, bf, bf_off, bf_len);
            break;
        case 8:
            copyin_int64(addr, val, bf, bf_off, bf_len);
            break;
        default:
            // Handle unsupported size
            break;
    }
}

uint64 read_const_arg(uint8** input_posp, uint64* size_p, uint64* bf_p, uint64* bf_off_p, uint64* bf_len_p)
{
	uint64 meta = read_input(input_posp, true);
	uint64 val = read_input(input_posp, true);
	*size_p = meta & 0xff;
	uint64 bf = (meta >> 8) & 0xff;
	*bf_off_p = (meta >> 16) & 0xff;
	*bf_len_p = (meta >> 24) & 0xff;
	uint64 pid_stride = meta >> 32;
	val += pid_stride * procid;
	*bf_p = bf;
	return val;
}


uint64 read_result(uint8** input_posp)
{
	uint64 idx = read_input(input_posp, true);
	uint64 op_div = read_input(input_posp, true);
	uint64 op_add = read_input(input_posp, true);
	uint64 arg = read_input(input_posp, true);
	if (idx >= kMaxCommands) {}
	if (results[idx].executed) {
		arg = results[idx].val;
		if (op_div != 0)
			arg = arg / op_div;
		arg += op_add;
	}
	return arg;
}

uint64 read_arg(uint8** input_posp)
{
	uint64 typ = read_input(input_posp, true);
	switch (typ) {
	case arg_const: {
		uint64 size, bf, bf_off, bf_len;
		uint64 val = read_const_arg(input_posp, &size, &bf, &bf_off, &bf_len);
		if (bf != binary_format_native && bf != binary_format_bigendian) {}
		if (bf_off != 0 || bf_len != 0) {}
		return swap(val, size, bf);
	}
	case arg_addr32:
	case arg_addr64: {
		return read_input(input_posp, true) + SYZ_DATA_OFFSET;
	}
	case arg_result: {
		uint64 meta = read_input(input_posp, true);
		uint64 bf = meta >> 8;
		if (bf != binary_format_native) {}
		return read_result(input_posp);
	}
	default:
        return 0;
	}
}

static intptr_t execute_syscall(const call_t* c, intptr_t a[kMaxArgs])
{
	return c->call(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8]);
}

void _kcov_buf_full()
{
    for(;;) {
        if (_kcov_start_addr[4096] == 0) {
            break;
        }
    }
}

void _kcmp_buf_full()
{
    for(;;) {
        if (_kcmp_start_addr[4096] == 0) {
            break;
        }
    }
}

void handle_call()
{

}


void fuzz_start_one(uint8* input_pos) 
{

    read_input(&input_pos, true); // total number of calls
	for (;;) {
		uint64 call_num = read_input(&input_pos, true);
		if (call_num == instr_eof)
			break;
		if (call_num == instr_copyin) {
			char* addr = (char*)(read_input(&input_pos, true) + SYZ_DATA_OFFSET);
			uint64 typ = read_input(&input_pos, true);
			switch (typ) {
			case arg_const: {
				uint64 size, bf, bf_off, bf_len;
				uint64 arg = read_const_arg(&input_pos, &size, &bf, &bf_off, &bf_len);
				copyin(addr, arg, size, bf, bf_off, bf_len);
				break;
			}
			case arg_addr32:
			case arg_addr64: {
				uint64 val = read_input(&input_pos, true) + SYZ_DATA_OFFSET;
				if (typ == arg_addr32) {}
				else {}
				break;
			}
			case arg_result: {
				uint64 meta = read_input(&input_pos, true);
				uint64 size = meta & 0xff;
				uint64 bf = meta >> 8;
				uint64 val = read_result(&input_pos);
				copyin(addr, val, size, bf, 0, 0);
				break;
			}
			case arg_data: {
				uint64 size = read_input(&input_pos, true);
				size &= ~(1ull << 63); // readable flag
				if (input_pos + size > _fuzz_prog_buffer + kMaxInput) {}
				memcpy(addr, input_pos, size);
				input_pos += size;
				break;
			}
			default:
                break;
			}
			continue;
		}
		if (call_num == instr_copyout) {
			read_input(&input_pos, true); // index
			read_input(&input_pos, true); // addr
			read_input(&input_pos, true); // size
			// The copyout will happen when/if the call completes.
			continue;
		}
        const call_t* call = &syscalls[call_num];
        uint64 copyout_index = read_input(&input_pos, true);
		uint64 num_args = read_input(&input_pos, true);
        uint64 args[kMaxArgs] = {};
		for (uint64 i = 0; i < num_args; i++)
			args[i] = read_arg(&input_pos);
		for (uint64 i = num_args; i < kMaxArgs; i++)
			args[i] = 0;
        
        execute_syscall(call, args);

		handle_call();
	

    }
}

void wait_prog_buffer()
{

}

void clean_prog_buffer()
{
	for (int i = 0; i < 4096; i++) {
		_fuzz_prog_buffer[i] = 0;
	}
	
}

void executor_main()
{
    for (;;) {

		// wait for input
		printf("wait for prog\n");
		wait_prog_buffer();

		// execute program
		printf("execute one\n");
		fuzz_start_one(_fuzz_prog_buffer);

		printf("clean prog buffer");
		clean_prog_buffer();

	}
}
