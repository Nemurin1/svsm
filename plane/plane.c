#include <plane.h>
#include <console.h>

/********************************************
 * Variables
 ********************************************/
static struct realm_config config;

static struct plane_run run;
static struct aux_plane_context aux_planes[PLANE_MAX_AUX_PLANES_NUM + 1];

/********************************************
 * GIC
 ********************************************/

static void aux_plane_gic_init(struct gic_state *gic)
{
	gic->gicv3_hcr = (1UL << 0) | /* GIC_ENABLE */
			 (1UL << 8) | /* GIC_vSIGEOIcount */
			 (1UL << 15);  /* GIC_DVIM */
	gic->gicv3_misr = 0;
	gic->gicv3_vmcr = 0;

	for (int i = 0; i < PLANE_GIC_LRS_NUM; ++i) {
		gic->gicv3_lrs[i] = 0;
	}
}

/********************************************
 * Timer
 ********************************************/

static void aux_plane_timer_init(struct timer_state *timer) {
	timer->cntp_ctl = 0;
	timer->cntp_cval = 0;
	timer->cntv_ctl = 0;
	timer->cntv_cval = 0;
}

static bool check_aux_plane_timer_pending(struct timer_state *timer)
{
	return TIMER_ASSERTED(timer->cntp_ctl) || TIMER_ASSERTED(timer->cntv_ctl);
}

static void inject_virt_timer_irq(struct aux_plane_context *aux_plane)
{
	for (int i = 0; i < 4; ++i) {
		if (aux_plane->gic.gicv3_lrs[i] == 0x50c002000000001bUL) {
			return;
		}
	}

	for (int i = 0; i < 4; ++i) {
		unsigned long gic_lr_state = aux_plane->gic.gicv3_lrs[i] & ICH_LR_STATE;
		if (!gic_lr_state) {
			aux_plane->gic.gicv3_lrs[i] = 0x50c002000000001bUL;
			return;
		}
	}
}

/********************************************
 * Exception
 ********************************************/

static unsigned long *page_table_walk(struct aux_plane_context *aux_plane, unsigned long va)
{
	unsigned long *pgd, *p4d, *pud, *pmd, *pte;
	unsigned int pgd_off, p4d_off, pud_off, pmd_off, pte_off;

	pgd_off = (va >> 48) & 0xf;
	p4d_off = (va >> 39) & 0x1ff;
	pud_off = (va >> 30) & 0x1ff;
	pmd_off = (va >> 21) & 0x1ff;
	pte_off = (va >> 12) & 0x1ff;

	unsigned long ttbr_el1;
	unsigned long *tb_base;
	rsi_plane_sysreg_read(aux_plane->index, RSI_SYSREG(SYS_TTBR1_EL1), &ttbr_el1, NULL);
	tb_base = (unsigned long *)(ttbr_el1 & 0xfffffffffffcUL);

	pgd = tb_base + pgd_off;

	p4d = (unsigned long *)(*pgd & 0xfffffffff000UL) + p4d_off;
	if ((*p4d & 0x3) == 1) {
		return p4d;
	}

	pud = (unsigned long *)(*p4d & 0xfffffffff000UL) + pud_off;
	if ((*pud & 0x3) == 1) {
		return pud;
	}

	pmd = (unsigned long *)(*pud & 0xfffffffff000UL) + pmd_off;
	if ((*pmd & 0x3) == 1) {
		return pmd;
	}

	pte = (unsigned long *)(*pmd & 0xfffffffff000UL) + pte_off;

	return pte;
}

static bool handle_data_abort(struct aux_plane_context *aux_plane)
{
	bool ret = false;

	unsigned long far = run.exit.far_el2;
	unsigned long hpfar = run.exit.hpfar_el2;
	unsigned long fipa = hpfar << 8L;
	unsigned long fipapn = fipa >> 12L;

	unsigned long *pte;
	unsigned long ipa_width;
	unsigned long prot_ns_shared;

	ipa_width = config.ipa_bits;
	prot_ns_shared = 1UL << (ipa_width - 1);

	switch (fipapn) {
		case 0x8000 ... 0x800f:
		case 0x8080 ... 0x808f:
		case 0x80a0 ... 0x8fff:
		case 0x9000:
		case 0x9010:
		case 0x9040:
		case 0xa000 ... 0xa003:
		case 0x10000 ... 0x10001:
		case 0x40030 ... 0x4007f:
		case 0x409da ... 0x40a22:
		case 0x41270 ... 0x4127f:
		case 0xb1600 ... 0xb55ff:
		case 0xbac00 ... 0xbacc1:
		case 0xbf4e0:
		case 0x4010000 ... 0x401ffff:
		case 0x8000000 ... 0x8000007:
			pte = page_table_walk(aux_plane, far & ~0xfffUL);
			*pte = *pte | prot_ns_shared;
			ret = true;
			break;
		default:
			// efi_print("[P0]\tFIPA 0x%lx not handled\n", fipa);
			break;
	}

	return ret;
}

static bool handle_smc_exception(struct aux_plane_context *aux_plane)
{
	bool ret = false;

	unsigned long smc_func = aux_plane->gprs[0];
	unsigned long args[5] = {0};
	args[4] = aux_plane->gprs[4];

	switch (smc_func) {
		case 0x80000000: /* SMC Version */
			args[0] = (1UL << 16) | 2UL;
			ret = true;
			break;
		case 0x80000001: /* SMC Feature */
			args[0] = (unsigned long)(-1);
			ret = true;
			break;
		case 0x84000000: /* PSCI Version */
			args[0] = (1UL << 16) | 1UL;
			ret = true;
			break;
		case 0x84000006: /* PSCI_MIGRATE_INFO_TYPE */
			args[0] = (unsigned long)(-1);
			ret = true;
			break;
		case 0x8400000a: /* PSCI_FEATURES */
			switch (aux_plane->gprs[1]) {
				case 0x80000000:
				case 0xc4000001: /* PSCI CPU_SUSPEND */
					args[0] = 0;
					ret = true;
					break;
				default:
					args[0] = (unsigned long)(-1);
					ret = true;
					break;
			}
			break;
		case 0x84000050: /* PSCI_TRNG */
			args[0] = (unsigned long)(-1);
			ret = true;
			break;
		case 0xbeefdead:
			rsi_plane_enter(0xdeadbeef, (unsigned long)0);
			break;
		case SMC_RSI_ABI_VERSION:
			args[0] = (unsigned long)(-1);
			ret = true;
			break;
		default:
			ret = false;
			break;
	}

	if (ret) {
		// Successfully handled smc function
		aux_plane->pc += 4UL;
		for (int i = 0; i < 5; i++) {
			aux_plane->gprs[i] = args[i];
		}

	} else {
		// Failed to handle smc function
		// efi_print("[P0]\tSMC function 0x%lx not handled\n", smc_func);
	}

	return ret;
}

static bool handle_sys_exception(struct aux_plane_context *aux_plane)
{
	bool ret = false;

	unsigned long iss = ESR_ELx_ISS(run.exit.esr_el2);

	switch (iss) {
		case 0x240024:
			ret = true;
			break;
		default:
			break;
	}

	if (ret) {
		// Successfully handled sys function
		aux_plane->pc += 4UL;
	} else {
		// Failed to handle sys function
		// efi_print("[P0]\tSYS function 0x%lx not handled\n", iss);
	}

	return ret;
}


static bool handle_sync_exception(struct aux_plane_context *aux_plane)
{
	struct plane_exit *plane_exit = &run.exit;

	unsigned long esr = plane_exit->esr_el2;

	bool ret = false;

	switch (ESR_ELx_EC(esr)) {
		case ESR_ELx_EC_WFx:
			break;
		case ESR_ELx_EC_DABT_LOW:
			ret = handle_data_abort(aux_plane);
			break;
		case ESR_ELx_EC_IABT_LOW:
			break;
		case ESR_ELx_EC_HVC64:
			break;
		case ESR_ELx_EC_SMC64:
			ret = handle_smc_exception(aux_plane);
			break;
		case ESR_ELx_EC_SYS64:
			ret = handle_sys_exception(aux_plane);
			break;
		default:
			break;
	}

	return ret;
}

static bool handle_irq_exception(struct aux_plane_context *aux_plane)
{
	struct plane_exit *plane_exit = &run.exit;

	unsigned long misr = plane_exit->gicv3_misr;
	if (misr == 0x1) {
		for (int i = 0; i < 4; ++i) {
			unsigned long gic_lr_state = aux_plane->gic.gicv3_lrs[i] & ICH_LR_STATE;
			if (!gic_lr_state) {
				aux_plane->gic.gicv3_lrs[i] = 0;
			}
		}
	}

	return true;
}

static bool handle_host_exception(struct aux_plane_context *aux_plane)
{
	return false;
}

static bool handle_aux_plane_exception(struct aux_plane_context *aux_plane)
{
	struct plane_exit *plane_exit = &run.exit;

	bool ret = false;

	int reason = plane_exit->reason;
	switch (reason) {
	case RSI_EXIT_SYNC:
		ret = handle_sync_exception(aux_plane);
		break;
	case RSI_EXIT_IRQ:
		ret = handle_irq_exception(aux_plane);
		break;
	case RSI_EXIT_HOST:
		ret = handle_host_exception(aux_plane);
		break;
	default:
		break;
	}

	return ret;
}

/********************************************
 * Context
 ********************************************/

static void aux_plane_context_init(int plane_index,
				   unsigned long plane_entry,
				   unsigned long plane_fdt_addr)
{
	struct aux_plane_context *aux_plane;
	aux_plane = &aux_planes[plane_index];

	aux_plane->state = PLANE_STATE_PENDING;
	aux_plane->index = plane_index;

	aux_plane->pc = plane_entry;
	aux_plane->gprs[0] = plane_fdt_addr;
	aux_plane->pstate = PSR_MODE_EL1h | PSR_I_BIT | PSR_F_BIT | PSR_A_BIT | PSR_D_BIT;
	aux_plane->flags = PLANE_ENTER_FLAG_GIC_OWNER;

	rsi_set_memory_range_shared(0x40030000UL, 0x40080000UL);
	rsi_set_memory_range_shared(0x409da000UL, 0x40a23000UL);
	rsi_set_memory_range_shared(0x41270000UL, 0x41280000UL);
	rsi_set_memory_range_shared(0xb1600000UL, 0xb5600000UL);
	rsi_set_memory_range_shared(0xbac00000UL, 0xbacc2000UL);

	aux_plane_gic_init(&aux_plane->gic);
	
	aux_plane_timer_init(&aux_plane->timer);

}

static void restore_aux_plane_context(struct aux_plane_context *aux_plane)
{
	/* Restore the aux plane context */
	run.enter.pc = aux_plane->pc;
	run.enter.flags = aux_plane->flags;
	run.enter.spsr_el2 = aux_plane->pstate;
	for (int i = 0; i < PLANE_GPRS_NUM; i++)
		run.enter.gprs[i] = aux_plane->gprs[i];

	aux_plane->state = PLANE_STATE_ACTIVE;

	run.enter.gicv3_hcr = aux_plane->gic.gicv3_hcr;
	for (int i = 0; i < PLANE_GIC_LRS_NUM; ++i) {
		run.enter.gicv3_lrs[i] = aux_plane->gic.gicv3_lrs[i];
	}
}

static void save_aux_plane_context(struct aux_plane_context *aux_plane)
{
	/* Save the aux plane context */
	aux_plane->pc = run.exit.elr_el2;
	aux_plane->pstate = run.exit.spsr_el2;
	for (int i = 0; i < PLANE_GPRS_NUM; i++)
		aux_plane->gprs[i] = run.exit.gprs[i];

	aux_plane->state = PLANE_STATE_STOPPED;

	aux_plane->gic.gicv3_hcr = run.exit.gicv3_hcr;
	aux_plane->gic.gicv3_misr = run.exit.gicv3_misr;
	aux_plane->gic.gicv3_vmcr = run.exit.gicv3_vmcr;
	for (int i = 0; i < PLANE_GIC_LRS_NUM; ++i) {
		aux_plane->gic.gicv3_lrs[i] = run.exit.gicv3_lrs[i];
	}

	aux_plane->timer.cntp_ctl = run.exit.cntp_ctl;
	aux_plane->timer.cntp_cval = run.exit.cntp_cval;
	aux_plane->timer.cntv_ctl = run.exit.cntv_ctl;
	aux_plane->timer.cntv_cval = run.exit.cntv_cval;
}

static void context_main_loop(void)
{
	while (true) {
		static int i = 0;
		i = (i + 1) % config.num_aux_planes;
		int current_plane = i + 1;

		struct aux_plane_context *aux_plane;
		aux_plane = &aux_planes[current_plane];
		if (aux_plane->state != PLANE_STATE_PENDING)
			continue;

		if (check_aux_plane_timer_pending(&aux_plane->timer)) {
			inject_virt_timer_irq(aux_plane);
		}
		restore_aux_plane_context(aux_plane);
		if (rsi_plane_enter(current_plane, (unsigned long)&run) != 0) {
			// efi_print("[P%d]\tEnter aux plane failed\n", current_plane);
		}
		save_aux_plane_context(aux_plane);

		if (handle_aux_plane_exception(aux_plane)) {
			aux_plane->state = PLANE_STATE_PENDING;
		} else {
			aux_plane->state = PLANE_STATE_ABORT;
			// efi_print("[P0]\tUnhandled P%d exception\n", current_plane);
			for (;;);
		}
	}
}

/********************************************
 * Core
 ********************************************/

static inline unsigned long get_realm_config(void)
{
	struct arm_smccc_res res;
	arm_smccc_smc(SMC_RSI_REALM_CONFIG, (unsigned long)&config,
		      0, 0, 0, 0, 0, 0, &res);
	return res.a0;
}

void plane_main(unsigned long kernel_entry,
			    unsigned long kernel_fdt_addr)
{
	int ret;

	/* Get realm configure */
	ret = get_realm_config();
	if (ret != 0) {
		// efi_print("[P0]\tGet realm config failed\n");
		for (;;);
	}

	if (config.num_aux_planes == 0) {
		// efi_print("[P0]\tNo aux plane, enter kernel directly\n");
		/* No aux plane, enter kernel directly */
		void (*entry)(u64, u64, u64, u64);
		entry = (void *)kernel_entry;
		entry(kernel_fdt_addr, 0, 0, 0);
	}

	/* Initialize the aux plane context */
	for (int i = 1; i <= config.num_aux_planes; i++) {
		aux_plane_context_init(i, kernel_entry,
				       kernel_fdt_addr);
	}

	/* Schedule planes */
	context_main_loop();
}

typedef void (*request_fn_t)(void);

void plane_main_svsm(unsigned long kernel_entry,
			    unsigned long kernel_fdt_addr)
{
	int ret;

	uart_puts("Hello UART\n");

	/* Get realm configure */

	ret = get_realm_config();
	if (ret != 0) {
		// efi_print("[P0]\tGet realm config failed\n");
		uart_puts("[P0]\tGet realm config failed\n");
		for (;;);
	}

	uart_puts("[P0]\tGet realm config\n");

	if (config.num_aux_planes == 0) {
		// efi_print("[P0]\tNo aux plane, enter kernel directly\n");
		/* No aux plane, enter kernel directly */
		void (*entry)(u64, u64, u64, u64);
		entry = (void *)kernel_entry;
		entry(kernel_fdt_addr, 0, 0, 0);
	}
	uart_puts("[P0]\tCurrent realm have pn\n");

	/* Initialize the aux plane context */
	for (int i = 1; i <= 1; i++) {
		aux_plane_context_init(i, kernel_entry,
				       kernel_fdt_addr);
	}
	uart_puts("[P0]\tExit Plane code\n");
}

static void context_main_once(request_fn_t request_callback)
{
	while (true) {
		static int i = 0;
		i = (i + 1) % config.num_aux_planes;
		int current_plane = i + 1;

		struct aux_plane_context *aux_plane;
		aux_plane = &aux_planes[current_plane];
		if (aux_plane->state != PLANE_STATE_PENDING)
			continue;

		if (check_aux_plane_timer_pending(&aux_plane->timer)) {
			inject_virt_timer_irq(aux_plane);
		}
		restore_aux_plane_context(aux_plane);
		if (rsi_plane_enter(current_plane, (unsigned long)&run) != 0) {
			// efi_print("[P%d]\tEnter aux plane failed\n", current_plane);
		}
		save_aux_plane_context(aux_plane);

		if (handle_aux_plane_exception(aux_plane)) {
			aux_plane->state = PLANE_STATE_PENDING;
		} else {
			aux_plane->state = PLANE_STATE_ABORT;
			// efi_print("[P0]\tUnhandled P%d exception\n", current_plane);
			for (;;);
		}
	}
}