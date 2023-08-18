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

    return true;
}
