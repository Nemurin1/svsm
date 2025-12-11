/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#include <rsi_smc.h>
#include <symbol.h>

#define RSI_GRANULE_SHIFT		12
#define RSI_GRANULE_SIZE		(_AC(1, UL) << RSI_GRANULE_SHIFT)

enum ripas {
	RSI_RIPAS_EMPTY = 0,
	RSI_RIPAS_RAM = 1,
	RSI_RIPAS_DESTROYED = 2,
	RSI_RIPAS_DEV = 3,
};

static inline unsigned long rsi_request_version(unsigned long req,
						unsigned long *out_lower,
						unsigned long *out_higher)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_ABI_VERSION, req, 0, 0, 0, 0, 0, 0, &res);

	if (out_lower)
		*out_lower = res.a1;
	if (out_higher)
		*out_higher = res.a2;

	return res.a0;
}

static inline unsigned long rsi_plane_enter(unsigned long plane_idx,
					    phys_addr_t run_ptr)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_PLANE_ENTER, plane_idx, run_ptr,
		      0, 0, 0, 0, 0, &res);

	return res.a0;

}

static inline unsigned long rsi_plane_sysreg_read(unsigned long plane_idx,
						  unsigned long sysreg_addr,
						  unsigned long *value_lower,
						  unsigned long *value_upper)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_PLANE_SYSREG_READ, plane_idx, sysreg_addr,
			0, 0, 0, 0, 0, &res);

	if (res.a0 == RSI_SUCCESS) {
		if (value_lower) {
			*value_lower = res.a1;
		}
		if (value_upper) {
			/*
			 * TODO: should check whether it is a 128 bits sysreg.
			 */
			*value_upper = res.a0;
		}
	}

	return res.a0;
}

static inline unsigned long rsi_plane_sysreg_write(unsigned long plane_idx,
						   unsigned long sysreg_addr,
						   unsigned long value_lower,
						   unsigned long value_upper)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_PLANE_SYSREG_WRITE, plane_idx, sysreg_addr,
			value_lower, value_upper, 0, 0, 0, &res);

	return res.a0;
}

static inline long rsi_set_addr_range_state(phys_addr_t start,
					    phys_addr_t end,
					    enum ripas state,
					    unsigned long flags,
					    phys_addr_t *top)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_IPA_STATE_SET, start, end, state,
		      flags, 0, 0, 0, &res);

	if (top)
		*top = res.a1;

	if (res.a2 != RSI_ACCEPT)
		return -1;

	return res.a0;
}

static inline int rsi_set_memory_range(phys_addr_t start, phys_addr_t end,
				       enum ripas state, unsigned long flags)
{
	unsigned long ret;
	phys_addr_t top;

	while (start != end) {
		ret = rsi_set_addr_range_state(start, end, state, flags, &top);
		if (ret || top < start || top > end)
			return -1;
		start = top;
	}

	return 0;
}

static inline int rsi_set_memory_range_shared(phys_addr_t start,
					      phys_addr_t end)
{
	return rsi_set_memory_range(start, end, RSI_RIPAS_EMPTY,
				    RSI_CHANGE_DESTROYED);
}