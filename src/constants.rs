#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::all)]
// list of all NIC registers and some structs
// copied and changed from the ixy C driver and DPDK

/*******************************************************************************

Copyright (c) 2001-2020, Intel Corporation
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

 1. Redistributions of source code must retain the above copyright notice,
    this list of conditions and the following disclaimer.

 2. Redistributions in binary form must reproduce the above copyright
    notice, this list of conditions and the following disclaimer in the
    documentation and/or other materials provided with the distribution.

 3. Neither the name of the Intel Corporation nor the names of its
    contributors may be used to endorse or promote products derived from
    this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.

***************************************************************************/

/* Vendor ID */
pub const IGB_INTEL_VENDOR_ID: u32 = 0x8086;

/* Device IDs */
//* transform by qwq not checked clearly */
pub const IGB_DEV_ID_82576: u32 = 0x10C9;
pub const IGB_DEV_ID_82576_FIBER: u32 = 0x10E6;
pub const IGB_DEV_ID_82576_SERDES: u32 = 0x10E7;
pub const IGB_DEV_ID_82576_QUAD_COPPER: u32 = 0x10E8;
pub const IGB_DEV_ID_82576_QUAD_COPPER_ET2: u32 = 0x1526;
pub const IGB_DEV_ID_82576_NS: u32 = 0x150A;
pub const IGB_DEV_ID_82576_NS_SERDES: u32 = 0x1518;
pub const IGB_DEV_ID_82576_SERDES_QUAD: u32 = 0x150D;
pub const IGB_DEV_ID_82575EB_COPPER: u32 = 0x10A7;
pub const IGB_DEV_ID_82575EB_FIBER_SERDES: u32 = 0x10A9;
pub const IGB_DEV_ID_82575GB_QUAD_COPPER: u32 = 0x10D6;
pub const IGB_DEV_ID_82580_COPPER: u32 = 0x150E;
pub const IGB_DEV_ID_82580_FIBER: u32 = 0x150F;
pub const IGB_DEV_ID_82580_SERDES: u32 = 0x1510;
pub const IGB_DEV_ID_82580_SGMII: u32 = 0x1511;
pub const IGB_DEV_ID_82580_COPPER_DUAL: u32 = 0x1516;
pub const IGB_DEV_ID_82580_QUAD_FIBER: u32 = 0x1527;
pub const IGB_DEV_ID_I350_COPPER: u32 = 0x1521;
pub const IGB_DEV_ID_I350_FIBER: u32 = 0x1522;
pub const IGB_DEV_ID_I350_SERDES: u32 = 0x1523;
pub const IGB_DEV_ID_I350_SGMII: u32 = 0x1524;
pub const IGB_DEV_ID_I350_DA4: u32 = 0x1546;
pub const IGB_DEV_ID_I210_COPPER: u32 = 0x1533;
pub const IGB_DEV_ID_I210_COPPER_OEM1: u32 = 0x1534;
pub const IGB_DEV_ID_I210_COPPER_IT: u32 = 0x1535;
pub const IGB_DEV_ID_I210_FIBER: u32 = 0x1536;
pub const IGB_DEV_ID_I210_SERDES: u32 = 0x1537;
pub const IGB_DEV_ID_I210_SGMII: u32 = 0x1538;
pub const IGB_DEV_ID_I210_COPPER_FLASHLESS: u32 = 0x157B;
pub const IGB_DEV_ID_I210_SERDES_FLASHLESS: u32 = 0x157C;
pub const IGB_DEV_ID_I210_SGMII_FLASHLESS: u32 = 0x15F6;
pub const IGB_DEV_ID_I211_COPPER: u32 = 0x1539;
pub const IGB_DEV_ID_I354_BACKPLANE_1GBPS: u32 = 0x1F40;
pub const IGB_DEV_ID_I354_SGMII: u32 = 0x1F41;
pub const IGB_DEV_ID_I354_BACKPLANE_2_5GBPS: u32 = 0x1F45;
pub const IGB_DEV_ID_DH89XXCC_SGMII: u32 = 0x0438;
pub const IGB_DEV_ID_DH89XXCC_SERDES: u32 = 0x043A;
pub const IGB_DEV_ID_DH89XXCC_BACKPLANE: u32 = 0x043C;
pub const IGB_DEV_ID_DH89XXCC_SFP: u32 = 0x0440;

// unused/unsupported by ixy
pub fn IGB_BY_MAC(_hw: u32, _r: u32) -> u32 {
    0
}

/* General Registers */
pub const IGB_CTRL: u32 = 0x00000;
pub const IGB_STATUS: u32 = 0x00008;
pub const IGB_CTRL_EXT: u32 = 0x00018;