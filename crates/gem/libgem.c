/*
 * Copyright 2021, Breakaway Consulting Pty. Ltd.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */
#include <stdint.h>
#include <stdbool.h>
#include <sel4cp.h>
#include "xemacps.h"
#include "xemacps_example.h"

bool gem_init(void);

#define ZYNQ_EMACPS_0_BASEADDR 0xE000B000
#define ZYNQ_EMACPS_1_BASEADDR 0xE000C000

#define ZYNQMP_EMACPS_0_BASEADDR 0xFF0B0000
#define ZYNQMP_EMACPS_1_BASEADDR 0xFF0C0000
#define ZYNQMP_EMACPS_2_BASEADDR 0xFF0D0000
#define ZYNQMP_EMACPS_3_BASEADDR 0xFF0E0000

#define VERSAL_EMACPS_0_BASEADDR 0xFF0C0000
#define VERSAL_EMACPS_1_BASEADDR 0xFF0D0000

#define RXBD_CNT       32	/* Number of RxBDs to use */
#define TXBD_CNT       32	/* Number of TxBDs to use */

/*
 * SLCR setting
 */
#define SLCR_LOCK_ADDR			(XPS_SYS_CTRL_BASEADDR + 0x4)
#define SLCR_UNLOCK_ADDR		(XPS_SYS_CTRL_BASEADDR + 0x8)
#define SLCR_GEM0_CLK_CTRL_ADDR		(XPS_SYS_CTRL_BASEADDR + 0x140)
#define SLCR_GEM1_CLK_CTRL_ADDR		(XPS_SYS_CTRL_BASEADDR + 0x144)


#define SLCR_LOCK_KEY_VALUE		0x767B
#define SLCR_UNLOCK_KEY_VALUE		0xDF0D
#define SLCR_ADDR_GEM_RST_CTRL		(XPS_SYS_CTRL_BASEADDR + 0x214)

/* CRL APB registers for GEM clock control */
#ifdef XPAR_PSU_CRL_APB_S_AXI_BASEADDR
#define CRL_GEM0_REF_CTRL	(XPAR_PSU_CRL_APB_S_AXI_BASEADDR + 0x50)
#define CRL_GEM1_REF_CTRL	(XPAR_PSU_CRL_APB_S_AXI_BASEADDR + 0x54)
#define CRL_GEM2_REF_CTRL	(XPAR_PSU_CRL_APB_S_AXI_BASEADDR + 0x58)
#define CRL_GEM3_REF_CTRL	(XPAR_PSU_CRL_APB_S_AXI_BASEADDR + 0x5C)
#endif

#define CRL_GEM_DIV_MASK	0x003F3F00
#define CRL_GEM_DIV0_SHIFT	8
#define CRL_GEM_DIV1_SHIFT	16

#ifdef XPAR_PSV_CRL_0_S_AXI_BASEADDR
#define CRL_GEM0_REF_CTRL	(XPAR_PSV_CRL_0_S_AXI_BASEADDR + 0x118)
#define CRL_GEM1_REF_CTRL	(XPAR_PSV_CRL_0_S_AXI_BASEADDR + 0x11C)
#endif

#define CRL_GEM_DIV_VERSAL_MASK		0x0003FF00
#define CRL_GEM_DIV_VERSAL_SHIFT	8

#define JUMBO_FRAME_SIZE	10240
#define FRAME_HDR_SIZE		18

#define GEMVERSION_ZYNQMP	0x7
#define GEMVERSION_VERSAL	0x107



u32 GemVersion;

bool gem_init(void) {
    LONG Status;
    XEmacPs_Config *Config;
    /*
	 *  Initialize instance. Should be configured for DMA
	 *  This example calls _CfgInitialize instead of _Initialize due to
	 *  retiring _Initialize. So in _CfgInitialize we use
	 *  XPAR_(IP)_BASEADDRESS to make sure it is not virtual address.
	 */
	Config = XEmacPs_LookupConfig(0);

    Status = XEmacPs_CfgInitialize(&EmacPsInstance, Config,
					Config->BaseAddress);

	if (Status != XST_SUCCESS) {
		EmacPsUtilErrorTrap("Error in initialize");
		return false;
	}

    GemVersion = ((Xil_In32(Config->BaseAddress + 0xFC)) >> 16) & 0xFFF;

	if (GemVersion == GEMVERSION_VERSAL) {
		Platform = Xil_In32(VERSAL_VERSION);
	} else if (GemVersion > 2) {
		Platform = Xil_In32(CSU_VERSION);
	}
	/* Enable jumbo frames for zynqmp */
	if (GemVersion > 2) {
		XEmacPs_SetOptions(&EmacPsInstance, XEMACPS_JUMBO_ENABLE_OPTION);
	}

    // INTR ID is not needed here
	XEmacPsClkSetup(&EmacPsInstance, 0);

    return true;
}

/****************************************************************************/
/**
*
* This function sets up the clock divisors for 1000Mbps.
*
* @param	EmacPsInstancePtr is a pointer to the instance of the EmacPs
*			driver.
* @param	EmacPsIntrId is the Interrupt ID and is typically
*			XPAR_<EMACPS_instance>_INTR value from xparameters.h.
* @return	None.
*
* @note		None.
*
*****************************************************************************/
void XEmacPsClkSetup(XEmacPs *EmacPsInstancePtr, u16 EmacPsIntrId)
{
	u32 ClkCntrl;
	u32 BaseAddress = EmacPsInstancePtr->Config.BaseAddress;

	if (GemVersion == 2)
	{
		/*************************************/
		/* Setup device for first-time usage */
		/*************************************/

	/* SLCR unlock */
	*(volatile unsigned int *)(SLCR_UNLOCK_ADDR) = SLCR_UNLOCK_KEY_VALUE;
	if (BaseAddress == ZYNQ_EMACPS_0_BASEADDR) {
#ifdef XPAR_PS7_ETHERNET_0_ENET_SLCR_1000MBPS_DIV0
		/* GEM0 1G clock configuration*/
		ClkCntrl =
		*(volatile unsigned int *)(SLCR_GEM0_CLK_CTRL_ADDR);
		ClkCntrl &= EMACPS_SLCR_DIV_MASK;
		ClkCntrl |= (EmacPsInstancePtr->Config.S1GDiv1 << 20);
		ClkCntrl |= (EmacPsInstancePtr->Config.S1GDiv0 << 8);
		*(volatile unsigned int *)(SLCR_GEM0_CLK_CTRL_ADDR) =
								ClkCntrl;
#endif
	} else if (BaseAddress == ZYNQ_EMACPS_1_BASEADDR) {
#ifdef XPAR_PS7_ETHERNET_1_ENET_SLCR_1000MBPS_DIV1
		/* GEM1 1G clock configuration*/
		ClkCntrl =
		*(volatile unsigned int *)(SLCR_GEM1_CLK_CTRL_ADDR);
		ClkCntrl &= EMACPS_SLCR_DIV_MASK;
		ClkCntrl |= (EmacPsInstancePtr->Config.S1GDiv1 << 20);
		ClkCntrl |= (EmacPsInstancePtr->Config.S1GDiv0 << 8);
		*(volatile unsigned int *)(SLCR_GEM1_CLK_CTRL_ADDR) =
								ClkCntrl;
#endif
	}
	/* SLCR lock */
	*(unsigned int *)(SLCR_LOCK_ADDR) = SLCR_LOCK_KEY_VALUE;
	#ifndef __MICROBLAZE__
	sleep(1);
	#else
	unsigned long count=0;
	while(count < 0xffff)
	{
		count++;
	}
	#endif
	}

	if ((GemVersion == GEMVERSION_ZYNQMP) && ((Platform & PLATFORM_MASK) == PLATFORM_SILICON)) {

#ifdef XPAR_PSU_CRL_APB_S_AXI_BASEADDR
		if (BaseAddress == ZYNQMP_EMACPS_0_BASEADDR) {
			/* GEM0 1G clock configuration*/
			ClkCntrl =
			*(volatile unsigned int *)(CRL_GEM0_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv1 << CRL_GEM_DIV1_SHIFT;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV0_SHIFT;
			*(volatile unsigned int *)(CRL_GEM0_REF_CTRL) =
									ClkCntrl;

		}
		if (BaseAddress == ZYNQMP_EMACPS_1_BASEADDR) {

			/* GEM1 1G clock configuration*/
			ClkCntrl =
			*(volatile unsigned int *)(CRL_GEM1_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv1 << CRL_GEM_DIV1_SHIFT;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV0_SHIFT;
			*(volatile unsigned int *)(CRL_GEM1_REF_CTRL) =
									ClkCntrl;
		}
		if (BaseAddress == ZYNQMP_EMACPS_2_BASEADDR) {

			/* GEM2 1G clock configuration*/
			ClkCntrl =
			*(volatile unsigned int *)(CRL_GEM2_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv1 << CRL_GEM_DIV1_SHIFT;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV0_SHIFT;
			*(volatile unsigned int *)(CRL_GEM2_REF_CTRL) =
									ClkCntrl;

		}
		if (BaseAddress == ZYNQMP_EMACPS_3_BASEADDR) {
			/* GEM3 1G clock configuration*/
			ClkCntrl =
			*(volatile unsigned int *)(CRL_GEM3_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv1 << CRL_GEM_DIV1_SHIFT;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV0_SHIFT;
			*(volatile unsigned int *)(CRL_GEM3_REF_CTRL) =
									ClkCntrl;
		}
#endif
	}
	if ((GemVersion == GEMVERSION_VERSAL) &&
		((Platform & PLATFORM_MASK_VERSAL) == PLATFORM_VERSALSIL)) {

#ifdef XPAR_PSV_CRL_0_S_AXI_BASEADDR
		if (BaseAddress == VERSAL_EMACPS_0_BASEADDR) {
			/* GEM0 1G clock configuration*/
#if defined (__aarch64__) && (EL1_NONSECURE == 1)
			Xil_Smc(PM_SET_DIVIDER_SMC_FID, (((u64)EmacPsInstancePtr->Config.S1GDiv0 << 32) | CLK_GEM0_REF), 0, 0, 0, 0, 0, 0);
#else
			ClkCntrl = Xil_In32((UINTPTR)CRL_GEM0_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_VERSAL_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV_VERSAL_SHIFT;
			Xil_Out32((UINTPTR)CRL_GEM0_REF_CTRL, ClkCntrl);
#endif
		}
		if (BaseAddress == VERSAL_EMACPS_0_BASEADDR) {

			/* GEM1 1G clock configuration*/
#if defined (__aarch64__) && (EL1_NONSECURE == 1)
			Xil_Smc(PM_SET_DIVIDER_SMC_FID, (((u64)EmacPsInstancePtr->Config.S1GDiv0 << 32) | CLK_GEM1_REF), 0, 0, 0, 0, 0, 0);
#else
			ClkCntrl = Xil_In32((UINTPTR)CRL_GEM1_REF_CTRL);
			ClkCntrl &= ~CRL_GEM_DIV_VERSAL_MASK;
			ClkCntrl |= EmacPsInstancePtr->Config.S1GDiv0 << CRL_GEM_DIV_VERSAL_SHIFT;
			Xil_Out32((UINTPTR)CRL_GEM1_REF_CTRL, ClkCntrl);
#endif
		}
#endif
	}
}