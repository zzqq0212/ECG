#ifndef _KCOV_H_
#define _KCOV_H_

#ifndef u8
typedef unsigned char u8;
#endif

#ifndef u16
typedef unsigned short u16;
#endif
#ifndef u32
typedef unsigned int u32;
#endif

#ifndef u64
typedef unsigned long long u64;
#endif

#ifndef kcov_u64
typedef unsigned long long kcov_u64;
#endif

#define _RET_IP_		(u32)__builtin_return_address(0)

/* Number of 32-bit words written per one comparison: */
#define KCOV_WORDS_PER_CMP 4

#define KCOV_CMP_CONST          (1 << 0)
#define KCOV_CMP_SIZE(n)        ((n) << 1)

#define _kcov_size               4096 + 1        // 存储的comparisons个数（4个字节） + 地址的个数（每个地址4个字节），不是整块内存的长度
u32 _kcov_start_addr[_kcov_size]   = {0};       


#define _kcmp_size               4096 + 1        // 存储的comparisons个数（4个字节） + 每个comparisons（每个4个字节），不是整块内存的长度
u32 _kcmp_start_addr[_kcmp_size]   = {0};

u32 _kcov_once_total             = 0;
u32 _kcmp_once_total             = 0;

void __sanitizer_cov_trace_pc(void);
void __sanitizer_cov_trace_cmp1(u8 arg1, u8 arg2);
void __sanitizer_cov_trace_cmp2(u16 arg1, u16 arg2);
void __sanitizer_cov_trace_cmp4(u32 arg1, u32 arg2);
void __sanitizer_cov_trace_cmp8(kcov_u64 arg1, kcov_u64 arg2);
void __sanitizer_cov_trace_const_cmp1(u8 arg1, u8 arg2);
void __sanitizer_cov_trace_const_cmp2(u16 arg1, u16 arg2);
void __sanitizer_cov_trace_const_cmp4(u32 arg1, u32 arg2);
void __sanitizer_cov_trace_const_cmp8(kcov_u64 arg1, kcov_u64 arg2);
void __sanitizer_cov_trace_switch(u32 val, void *arg);

#endif