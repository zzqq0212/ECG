#include <assert.h>

void enable_kcov();
void disable_kcov();
int kcov_enabled();


typedef unsigned long long uint64;
typedef unsigned int uint32;
typedef unsigned short uint16;
typedef unsigned char uint8;


#define MAX_PROG_LEN 8192
__attribute__((section(".kcov.metadata"))) static uint32 _fuzz_max_prog_len = MAX_PROG_LEN;
static uint8 _fuzz_prog_buffer[MAX_PROG_LEN] = {0};

// For handshake protocol between fuzzer and executor
#define ACK_VAR_DEFAULT -1
static uint32 _fuzz_ack_var = ACK_VAR_DEFAULT;
#define ACK_VAR_OK 1
__attribute__((section(".kcov.metadata"))) static uint32 _fuzz_ack_ok_var = ACK_VAR_OK;

static uint32 _fuzz_prog_index = 0;
static uint32 _fuzz_call_index = 0;

void __attribute__((noinline, used, optimize("O0"))) _fuzz_comm_ack() {
	_fuzz_ack_var = ACK_VAR_DEFAULT;
}

#define DO_HANDSHAKE(expected) { \
	assert((expected) == _fuzz_ack_var); \
	_fuzz_comm_ack(); \
}



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

const uint64 arg_csum_inet = 0;
const uint64 arg_csum_chunk_data = 0;
const uint64 arg_csum_chunk_const = 1;

const uint64 binary_format_native = 0;
const uint64 binary_format_bigendian = 1;
const uint64 binary_format_strdec = 2;
const uint64 binary_format_strhex = 3;
const uint64 binary_format_stroct = 4;

#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
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

struct csum_inet {
	uint32 acc;
};

static uint16 csum_inet_digest(struct csum_inet* csum)
{
	return ~csum->acc;
}

static void csum_inet_init(struct csum_inet* csum)
{
	csum->acc = 0;
}

static void csum_inet_update(struct csum_inet* csum, const uint8* data, size_t length)
{
	if (length == 0)
		return;

	size_t i = 0;
	for (; i < length - 1; i += 2)
		csum->acc += *(uint16*)&data[i];

	if (length & 1)
		csum->acc += le16toh((uint16)data[length - 1]);

	while (csum->acc > 0xffff)
		csum->acc = (csum->acc & 0xffff) + (csum->acc >> 16);
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
	uint64 meta = read_input(input_posp, false);
	uint64 val = read_input(input_posp, false);
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
	uint64 idx = read_input(input_posp, false);
	uint64 op_div = read_input(input_posp, false);
	uint64 op_add = read_input(input_posp, false);
	uint64 arg = read_input(input_posp, false);
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
	uint64 typ = read_input(input_posp, false);
	switch (typ) {
	case arg_const: {
		uint64 size, bf, bf_off, bf_len;
		uint64 val = read_const_arg(input_posp, &size, &bf, &bf_off, &bf_len);
		if (bf != binary_format_native && bf != binary_format_bigendian) {
			printf("#bad argument binary format format=%llu\n", bf);
		}
		if (bf_off != 0 || bf_len != 0) {
			printf("#bad argument bitfield off=%llu, len=%llu\n", bf_off, bf_len);
		}
		return swap(val, size, bf);
	}
	case arg_addr32:
	case arg_addr64: {
		return read_input(input_posp, false) + SYZ_DATA_OFFSET;
	}
	case arg_result: {
		uint64 meta = read_input(input_posp, false);
		uint64 bf = meta >> 8;
		if (bf != binary_format_native) {
			printf("#bad result argument format format=%llu\n", bf);
		}
		return read_result(input_posp);
	}
	default:
		printf("#bad argument type type=%llu\n", typ);
		return 0;
	}
}

intptr_t __attribute__((noinline, used, optimize("O0")))  execute_syscall(const call_t* c, intptr_t a[kMaxArgs])
{
	if (c == NULL)  {
		return -1;
	} else {
		return c->call(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8]);
		
	}
	return -1;
}

void handle_call()
{

}

int __attribute__((noinline, used, optimize("O0"))) fuzz_post_call() {
	DO_HANDSHAKE(_fuzz_ack_ok_var);
	printf("# in post call\n");
    return 0;
}

void print_call(const call_t* call, intptr_t args[kMaxArgs], int num_args) {
	printf("\n#executing %s (", call->name);
	for (int i = 0; i < num_args; i++) {
		if (i != 0){
			printf(", ");
			printf("0x%llx", (uint64)args[i]);
		} else {
			printf("0x%llx", (uint64)args[i]);
		}
	}
	printf(")\n");
}

void fuzz_start_one() 
{
	__sync_synchronize();
	DO_HANDSHAKE(_fuzz_ack_ok_var);

	_fuzz_call_index = 0;
	
	uint8* input_pos = _fuzz_prog_buffer;
    int a = read_input(&input_pos, false); // total number of calls
	printf("#executor: Prog has number of calls = %d\n", a);
	if (a == 0) {
		printf("#executor: Prog has no calls\n");
		return;
	}
	call_props_t call_props;
	memset(&call_props, 0, sizeof(call_props));

	// for (int i = 0; i < a; i++) {
	for(;;) {
		uint64 call_num = read_input(&input_pos, false);
		printf("#call_num = %llu\n", call_num);
		if (call_num == instr_eof){
			printf("#eof\n");
			break;
		}
			
		if (call_num == instr_copyin) {
			char* addr = (char*)(read_input(&input_pos, false) + SYZ_DATA_OFFSET);
			uint64 typ = read_input(&input_pos, false);
			switch (typ) {
			case arg_const: {
				uint64 size, bf, bf_off, bf_len;
				uint64 arg = read_const_arg(&input_pos, &size, &bf, &bf_off, &bf_len);
				copyin(addr, arg, size, bf, bf_off, bf_len);
				break;
			}
			case arg_addr32:
			case arg_addr64: {
				uint64 val = read_input(&input_pos, false) + SYZ_DATA_OFFSET;
				if (typ == arg_addr32) {
					*(uint32*)addr = val;
				}
				else {
					*(uint64*)addr = val;
				}
				break;
			}
			case arg_result: {
				uint64 meta = read_input(&input_pos, false);
				uint64 size = meta & 0xff;
				uint64 bf = meta >> 8;
				uint64 val = read_result(&input_pos);
				copyin(addr, val, size, bf, 0, 0);
				break;
			}
			case arg_data: {
				uint64 size = read_input(&input_pos, false);
				size &= ~(1ull << 63); // readable flag
				if (input_pos + size > _fuzz_prog_buffer + kMaxInput) {
					printf("#data arg overflow");
				}
				memcpy(addr, input_pos, size);
				input_pos += size;
				break;
			}
			case arg_csum: {
				printf("#checksum found at %p\n", addr);
				uint64 size = read_input(&input_pos, false);
				char* csum_addr = addr;
				uint64 csum_kind = read_input(&input_pos, false);
				switch (csum_kind) {
				case arg_csum_inet: {
					if (size != 2)
						printf("bag inet checksum size=%llu", size);
					printf("calculating checksum for %p\n", csum_addr);
					struct csum_inet csum;
					csum_inet_init(&csum);
					uint64 chunks_num = read_input(&input_pos, false);
					uint64 chunk;
					for (chunk = 0; chunk < chunks_num; chunk++) {
						uint64 chunk_kind = read_input(&input_pos, false);
						uint64 chunk_value = read_input(&input_pos, false);
						uint64 chunk_size = read_input(&input_pos, false);
						switch (chunk_kind) {
						case arg_csum_chunk_data:
							chunk_value += SYZ_DATA_OFFSET;
							printf("#%lld: data chunk, addr: %llx, size: %llu\n",
								      chunk, chunk_value, chunk_size);
							csum_inet_update(&csum, (const uint8*)chunk_value, chunk_size);
							break;
						case arg_csum_chunk_const:
							if (chunk_size != 2 && chunk_size != 4 && chunk_size != 8)
								printf("bad checksum const chunk size=%lld", chunk_size);
							// Here we assume that const values come to us big endian.
							printf("#%lld: const chunk, value: %llx, size: %llu\n",
								      chunk, chunk_value, chunk_size);
							csum_inet_update(&csum, (const uint8*)&chunk_value, chunk_size);
							break;
						default:
							printf("bad checksum chunk kind=%llu", chunk_kind);
						}
					}
					uint16 csum_value = csum_inet_digest(&csum);
					printf("writing inet checksum %hx to %p\n", csum_value, csum_addr);
					copyin(csum_addr, csum_value, 2, binary_format_native, 0, 0);
					break;
				}
				default:
					printf("bad checksum kind=%llu", csum_kind);
				}
				break;
			}
			default:
				printf("bad argument type=%llu", typ);
			}
			printf("#copyin done\n");
			continue;
		}
		if (call_num == instr_copyout) {
			read_input(&input_pos, false); // index
			read_input(&input_pos, false); // addr
			read_input(&input_pos, false); // size
			// The copyout will happen when/if the call completes.
			continue;
		}		
		if (call_num == instr_setprops) {

			printf("entered call_num == instr_setprops\n");

			read_call_props_t(call_props, read_input(&input_pos, false));
			continue;
		}
		if (call_num >= ARRAY_SIZE(syscalls)) {
			printf("#invalid syscall number call_num=%llu", call_num);
		}
		// if (call_num >= 0 && call_num <= 5) {
		// 	assert(false);
		// 	printf("#passing disabled syscall %llu\n", call_num);
			
		// 	continue;

		// }
        const call_t* call = &syscalls[call_num];
        uint64 copyout_index = read_input(&input_pos, false);
		uint64 num_args = read_input(&input_pos, false);
		if (num_args > kMaxArgs)
			printf("#command has bad number of arguments args=%llu\n", num_args);
        uint64 args[kMaxArgs] = {};
		for (uint64 i = 0; i < num_args; i++)
			args[i] = read_arg(&input_pos);
		for (uint64 i = num_args; i < kMaxArgs; i++)
			args[i] = 0;
        
		
		if (call == NULL)  {
			printf("#empty call\n");
			// continue;
		} else {
			print_call(call, args, num_args);
			enable_kcov(); assert(kcov_enabled());
        	execute_syscall(call, args);
			disable_kcov(); assert(!kcov_enabled());
			_fuzz_call_index += 1;
		}
		__sync_synchronize();
		fuzz_post_call();
		__sync_synchronize();
    }
	_fuzz_prog_index += 1;
}

/*
void __attribute__((noinline, used, optimize("O0")))   wait_prog_buffer()
{

}
*/

void clean_prog_buffer()
{
	printf("#clean prog buffer\n");
	for (int i = 0; i < MAX_PROG_LEN; i++) {
		_fuzz_prog_buffer[i] = 0;
	}
}

void executor_main()
{
    for (;;) {
		clean_prog_buffer();
		// wait for input
		printf("#wait for prog\n");

		// execute program
		// printf("#execute one\n");
        __sync_synchronize();
		fuzz_start_one();
	}
}

void executor_check_ints() {
	assert(sizeof(uint16_t) == 2);
    assert(sizeof(uint32_t) == 4);
    assert(sizeof(uint64_t) == 8);
    
    assert(sizeof(uint16_t) == 2);
    assert(sizeof(uint32_t) == 4);
    assert(sizeof(uint64_t) == 8);

    printf("#sizeof(uint8_t) = %d\n", sizeof(uint8_t));
    printf("#sizeof(uint16_t) = %d\n", sizeof(uint16_t));
    printf("#sizeof(uint32_t) = %d\n", sizeof(uint32_t));
    printf("#sizeof(uint64_t) = %d\n", sizeof(uint64_t));
	printf("#sizeof(uintptr_t) = %d\n", sizeof(uintptr_t));
}