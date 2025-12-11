#include <esr.h>
#include <ptrace.h>
#include <rsi_cmds.h>

#define PLANE_STATE_IDLE      0
#define PLANE_STATE_PENDING   1
#define PLANE_STATE_ACTIVE    2
#define PLANE_STATE_ABORT     3
#define PLANE_STATE_STOPPED   4

#define PLANE_GPRS_NUM 31
#define PLANE_GIC_LRS_NUM 16
#define PLANE_MAX_AUX_PLANES_NUM 3

#define RSI_SYSREG(sysreg)		((sysreg) >> 5)

typedef uint64_t u64;

/* Planes GIC 状态 */
struct gic_state {
    u64 gicv3_hcr;
    u64 gicv3_lrs[PLANE_GIC_LRS_NUM];
    u64 gicv3_misr;
    u64 gicv3_vmcr;
};

/* Planes Timer 状态 */
#define CNTx_CTL_ENABLE    (1 << 0)
#define CNTx_CTL_IMASK     (1 << 1)
#define CNTx_CTL_ISTATUS   (1 << 2)

#define TIMER_ASSERTED(ctl) \
    (((ctl) & CNTx_CTL_ENABLE) && \
     !((ctl) & CNTx_CTL_IMASK) && \
     ((ctl) & CNTx_CTL_ISTATUS))

struct timer_state {
    u64 cntp_ctl;
    u64 cntp_cval;
    u64 cntv_ctl;
    u64 cntv_cval;
};

/* Planes Context */
struct aux_plane_context {
    u64 state;
    u64 index;

    u64 pc;
    u64 gprs[PLANE_GPRS_NUM];
    u64 pstate;
    u64 flags;

    struct gic_state gic;
    struct timer_state timer;
} ;

/* plane 主函数，裸机实现，__noreturn 表示不会返回 */
__attribute__((noreturn))
void plane_main(unsigned long kernel_entry,
			   unsigned long kernel_fdt_addr);
