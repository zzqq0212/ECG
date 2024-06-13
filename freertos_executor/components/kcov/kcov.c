#include <stdio.h>
#include "kcov.h"

/*
 * Entry point from instrumented code.
 * This is called once per basic-block/edge.
 */
void __sanitizer_cov_trace_pc(void)
{
	u32 *area;
	u32 ip = _RET_IP_;
	u32 pos;

	area = _kcov_start_addr;
	/* The first 32-bit word is the number of subsequent PCs. */
	pos = area[0] + 1;
	if (pos < _kcov_size)
	 {
		/* Previously we write pc before updating pos. However, some
		 * early interrupt code could bypass check and invoke
		 * __sanitizer_cov_trace_pc(). If such interrupt is
		 * raised between writing pc and updating pos, the pc could be
		 * overitten by the recursive __sanitizer_cov_trace_pc().
		 * Update pos before writing pc to avoid such interleaving.
		 */
		area[0] = pos;
		area[pos] = ip;
	}
	_kcov_once_total++;
}

void __sanitizer_cov_trace_cmp1(u8 arg1, u8 arg2)
{
}

void __sanitizer_cov_trace_cmp2(u16 arg1, u16 arg2)
{
}

void __sanitizer_cov_trace_cmp4(u32 arg1, u32 arg2)
{
	u32 *area;
	u32 count, start_index, end_pos, max_pos;

	u32 type	= KCOV_CMP_SIZE(2);
	u32 ip 		= _RET_IP_;

	/*
	 * We write all comparison arguments and types as u32.
	 * The buffer in _kcmp_start_addr for u32.
	 */
	area = (u32 *)_kcmp_start_addr;
	max_pos = _kcmp_size * sizeof(u32);

	count = area[0];

	/* Every record is KCOV_WORDS_PER_CMP 32-bit words. */
	start_index = 1 + count * KCOV_WORDS_PER_CMP;
	end_pos = (start_index + KCOV_WORDS_PER_CMP) * sizeof(u32);
	if (end_pos <= max_pos) 
	{
		area[0] = count + 1;
		area[start_index] = type;
		area[start_index + 1] = arg1;
		area[start_index + 2] = arg2;
		area[start_index + 3] = ip;
	}
	_kcmp_once_total++;
}

void __sanitizer_cov_trace_cmp8(kcov_u64 arg1, kcov_u64 arg2)
{
}

void __sanitizer_cov_trace_const_cmp1(u8 arg1, u8 arg2)
{
}

void __sanitizer_cov_trace_const_cmp2(u16 arg1, u16 arg2)
{
}
void __sanitizer_cov_trace_const_cmp4(u32 arg1, u32 arg2)
{
	u32 *area;
	u32 count, start_index, end_pos, max_pos;

	u32 type	= KCOV_CMP_SIZE(2) | KCOV_CMP_CONST;
	u32 ip 		= _RET_IP_; 

	/*
	 * We write all comparison arguments and types as u32.
	 * The buffer in _kcmp_start_addr for u32.
	 */
	area = (u32 *)_kcmp_start_addr;
	max_pos = _kcmp_size * sizeof(u32);

	count = area[0];

	/* Every record is KCOV_WORDS_PER_CMP 32-bit words. */
	start_index = 1 + count * KCOV_WORDS_PER_CMP;
	end_pos = (start_index + KCOV_WORDS_PER_CMP) * sizeof(u32);
	if (end_pos <= max_pos) 
	{
		area[0] = count + 1;
		area[start_index] = type;
		area[start_index + 1] = arg1;
		area[start_index + 2] = arg2;
		area[start_index + 3] = ip;
	}
}

void __sanitizer_cov_trace_const_cmp8(kcov_u64 arg1, kcov_u64 arg2)
{
}

void __sanitizer_cov_trace_switch(u32 val, void *arg)
{
	u32 i;
	u32 *cases = arg;
	u32 count = cases[0];
	u32 size = cases[1];
	u32 type = KCOV_CMP_CONST;

	switch (size) {
	case 8:
		type |= KCOV_CMP_SIZE(0);
		break;
	case 16:
		type |= KCOV_CMP_SIZE(1);
		break;
	case 32:
		type |= KCOV_CMP_SIZE(2);
		break;
	case 64:
		type |= KCOV_CMP_SIZE(3);
		break;
	default:
		return;
	}
	for (i = 0; i < count; i++)
		//write_comp_data(type, cases[i + 2], val, _RET_IP_);
	{
		u32 *area;
		u32 count, start_index, end_pos, max_pos;

		u32 ip 		= _RET_IP_; 
		u32 arg1	= cases[i + 2];
		u32 arg2	= val;
		/*
		* We write all comparison arguments and types as u32.
		* The buffer in _kcmp_start_addr for u32.
		*/
		area = (u32 *)_kcmp_start_addr;
		max_pos = _kcmp_size * sizeof(u32);

		count = area[0];

		/* Every record is KCOV_WORDS_PER_CMP 32-bit words. */
		start_index = 1 + count * KCOV_WORDS_PER_CMP;
		end_pos = (start_index + KCOV_WORDS_PER_CMP) * sizeof(u32);
		if (end_pos <= max_pos) 
		{
			area[0] = count + 1;
			area[start_index] = type;
			area[start_index + 1] = arg1;
			area[start_index + 2] = arg2;
			area[start_index + 3] = ip;
		}
	}
}
