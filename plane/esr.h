#include <sysreg.h>

#define UL(x)   _UL(x)
#define _UL(x)  x##UL

#define ESR_ELx_EC_UNKNOWN	UL(0x00)
#define ESR_ELx_EC_WFx		UL(0x01)
/* Unallocated EC: 0x02 */
#define ESR_ELx_EC_CP15_32	UL(0x03)
#define ESR_ELx_EC_CP15_64	UL(0x04)
#define ESR_ELx_EC_CP14_MR	UL(0x05)
#define ESR_ELx_EC_CP14_LS	UL(0x06)
#define ESR_ELx_EC_FP_ASIMD	UL(0x07)
#define ESR_ELx_EC_CP10_ID	UL(0x08)	/* EL2 only */
#define ESR_ELx_EC_PAC		UL(0x09)	/* EL2 and above */
/* Unallocated EC: 0x0A - 0x0B */
#define ESR_ELx_EC_CP14_64	UL(0x0C)
#define ESR_ELx_EC_BTI		UL(0x0D)
#define ESR_ELx_EC_ILL		UL(0x0E)
/* Unallocated EC: 0x0F - 0x10 */
#define ESR_ELx_EC_SVC32	UL(0x11)
#define ESR_ELx_EC_HVC32	UL(0x12)	/* EL2 only */
#define ESR_ELx_EC_SMC32	UL(0x13)	/* EL2 and above */
/* Unallocated EC: 0x14 */
#define ESR_ELx_EC_SVC64	UL(0x15)
#define ESR_ELx_EC_HVC64	UL(0x16)	/* EL2 and above */
#define ESR_ELx_EC_SMC64	UL(0x17)	/* EL2 and above */
#define ESR_ELx_EC_SYS64	UL(0x18)
#define ESR_ELx_EC_SVE		UL(0x19)
#define ESR_ELx_EC_ERET		UL(0x1a)	/* EL2 only */
/* Unallocated EC: 0x1B */
#define ESR_ELx_EC_FPAC		UL(0x1C)	/* EL1 and above */
#define ESR_ELx_EC_SME		UL(0x1D)
/* Unallocated EC: 0x1E */
#define ESR_ELx_EC_IMP_DEF	UL(0x1f)	/* EL3 only */
#define ESR_ELx_EC_IABT_LOW	UL(0x20)
#define ESR_ELx_EC_IABT_CUR	UL(0x21)
#define ESR_ELx_EC_PC_ALIGN	UL(0x22)
/* Unallocated EC: 0x23 */
#define ESR_ELx_EC_DABT_LOW	UL(0x24)
#define ESR_ELx_EC_DABT_CUR	UL(0x25)
#define ESR_ELx_EC_SP_ALIGN	UL(0x26)
#define ESR_ELx_EC_MOPS		UL(0x27)
#define ESR_ELx_EC_FP_EXC32	UL(0x28)
/* Unallocated EC: 0x29 - 0x2B */
#define ESR_ELx_EC_FP_EXC64	UL(0x2C)
#define ESR_ELx_EC_GCS		UL(0x2D)
/* Unallocated EC:  0x2E */
#define ESR_ELx_EC_SERROR	UL(0x2F)
#define ESR_ELx_EC_BREAKPT_LOW	UL(0x30)
#define ESR_ELx_EC_BREAKPT_CUR	UL(0x31)
#define ESR_ELx_EC_SOFTSTP_LOW	UL(0x32)
#define ESR_ELx_EC_SOFTSTP_CUR	UL(0x33)
#define ESR_ELx_EC_WATCHPT_LOW	UL(0x34)
#define ESR_ELx_EC_WATCHPT_CUR	UL(0x35)
/* Unallocated EC: 0x36 - 0x37 */
#define ESR_ELx_EC_BKPT32	UL(0x38)
/* Unallocated EC: 0x39 */
#define ESR_ELx_EC_VECTOR32	UL(0x3A)	/* EL2 only */
/* Unallocated EC: 0x3B */
#define ESR_ELx_EC_BRK64	UL(0x3C)
/* Unallocated EC: 0x3D - 0x3F */
#define ESR_ELx_EC_MAX		UL(0x3F)

#define ESR_ELx_EC_SHIFT	(26)
#define ESR_ELx_EC_WIDTH	(6)
#define ESR_ELx_EC_MASK		(UL(0x3F) << ESR_ELx_EC_SHIFT)
#define ESR_ELx_EC(esr)		(((esr) & ESR_ELx_EC_MASK) >> ESR_ELx_EC_SHIFT)

#define ESR_ELx_ISS_MASK	0x01FFFFFFUL
#define ESR_ELx_ISS(esr)	((esr) & ESR_ELx_ISS_MASK)