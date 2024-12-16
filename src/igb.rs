#![allow()]
use core::{default, ptr::NonNull};

use log::{debug, info};

const DRIVER_NAME: &str = "igb";

//* 意义不明的三个常量 */
const MAX_QUEUES: u16 = 64;

const PKT_BUF_ENTRY_SIZE: usize = 2048;
const MIN_MEMPOOL_SIZE: usize = 4096;
//* 意义不明的三个常量 */


enum NicResolution{
    IgbFcNone,
    IgbFcRxPause,
    IgbFcTxPause,
    IgbFcFull,
}

pub struct Igb {bar0: NonNull<u8>}

impl Igb {

    const MDIO_ADDRESS: u32 = 0b00001;

    pub fn new(bar0: NonNull<u8>) -> Self {
        let igb = Igb {bar0};

        igb.disable_interrupts();
        igb.global_reset();
        igb.read_status();

        return igb;
    }

    pub fn disable_interrupts(&self) {
        unsafe { self.bar0.add(0x0150C).cast::<u32>().write_volatile(0x0000_0000) };
        info!("start disable interrupts");

        let ims = unsafe { self.bar0.add(0x01508).cast::<u32>().read_volatile() };
        
        if ims != 0 {
            panic!("interrupts not disabled");
        }

        debug!("interrupts disabled");
    }

    pub fn global_reset(&self) {
        // global reset , ILOS is initally set to 0
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl | 1<<26) };

        info!("start reset");

        loop {
            let ctr = unsafe { self.bar0.cast::<u32>().read_volatile() };
            if ctr & (1<<26) == 0 {
                break;
            }
        }

        debug!("reset");

        // the interrupts need to be disabled also after issuing a global reset
        self.disable_interrupts();
    }

    pub fn read_status(&self) {
        let status = unsafe { self.bar0.add(0x00008).cast::<u32>().read_volatile() };
        let fd = status & 1<<0;
        let lu = status & 1<<1;
        let speed = (status & 0b11 <<6)>>6;

        debug!("igb status : fd: {}, lu: {}, speed: {:#b}", fd, lu, speed);
    }

    pub fn disable_rx_tx_flow(&self){
        info!("disable RX and TX flow");
        let rctl = unsafe { self.bar0.add(0x00100).cast::<u32>().read_volatile() };
        unsafe { self.bar0.add(0x00100).cast::<u32>().write_volatile(rctl&(!(1<<1))) };
        debug!("Rx flow disabled");
        let tctl = unsafe { self.bar0.add(0x00400).cast::<u32>().read_volatile() };
        unsafe { self.bar0.add(0x00400).cast::<u32>().write_volatile(tctl&(!(1<<1))) };
        debug!("Tx flow disabled");
    }

    pub fn enable_rx_tx_flow(&self){
        info!("enable RX and TX flow");
        let rctl = unsafe { self.bar0.add(0x00100).cast::<u32>().read_volatile() };
        unsafe { self.bar0.add(0x00100).cast::<u32>().write_volatile(rctl|(1<<1)) };
        debug!("Rx flow enabled");
        let tctl = unsafe { self.bar0.add(0x00400).cast::<u32>().read_volatile() };
        unsafe { self.bar0.add(0x00400).cast::<u32>().write_volatile(tctl|(1<<1)) };
        debug!("Tx flow enabled");
    }

    pub fn set_rx_packet_buffer_size(&self, size: u32) {
        if size > 0b0111_1111{
            panic!("size too big");
        }
        self.disable_rx_tx_flow();
        unsafe { self.bar0.add(0x2404).cast::<u32>().write_volatile(size) };
        self.enable_rx_tx_flow();
    }

    pub fn set_tx_packet_buffer_size(&self, size: u32) {
        if size > 0b0011_1111{
            panic!("size too big");
        }
        self.disable_rx_tx_flow();
        unsafe { self.bar0.add(0x3404).cast::<u32>().write_volatile(size) };
        self.enable_rx_tx_flow();
    }

    pub fn forcing_mac_speed(&self, speed: u32) {
        if speed > 0b11 {
            panic!("speed too big");
        }
        if speed == 0b11 {
            panic!("speed 0b11 not used");
        }
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl | (1<<11) ) };
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        let frcspd = (ctrl & (1<<11))>>11;
        debug!("frcspd set to {:#b}", frcspd);
        //? 这里有一点非常重要,就是在设置一个位的时候,假如我们不知道其原始状态,我们应该先将其清零,然后再设置
        //? 一句话,就是要考虑原状态
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl & (!(0b11<<8)) | (speed<<8) ) };
        debug!("speed set to {:#b}", speed);
    }

    pub fn using_internal_phy_direct_linkspeed_indication(&self) {
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl & (!(1<<11)) ) };
        // CTRL-Bit5 Reserved. Must be set to 0b.- Was ASDE
        debug!("using internal phy direct link speed indication");
        let status = unsafe { self.bar0.add(0x00008).cast::<u32>().read_volatile() };
        let speed = status & 0b11 <<6;
        debug!("igb status : speed: {:#b}", speed);
    }

    pub fn forcing_duplex_mode(&self, duplex: u32) {
        if duplex > 0b1 {
            panic!("no such duplex mode");
        }
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl | (1<<12) ) };
        let ctrl = unsafe { self.bar0.cast::<u32>().read_volatile() };
        let frcdplx = (ctrl & (1<<12))>>12;
        debug!("frcdplx set to {:#b}", frcdplx);
        unsafe { self.bar0.cast::<u32>().write_volatile(ctrl & !(1) | duplex) };
        debug!("duplex set to {:#b}", duplex);
    }

    pub fn read_mdi(&self,reg_addr: u32) -> u32 {
        //* R: 1<<28 0b  */
        let mdic = 0x0000_0000 | 
                        reg_addr<<16 | 
                        Self::MDIO_ADDRESS<<21 | 
                        0b10<<26 | 
                        1<<28 ;
        unsafe { self.bar0.add(0x00020).cast::<u32>().write_volatile(mdic) };
        loop {
            let mdic = unsafe { self.bar0.add(0x00020).cast::<u32>().read_volatile() };
            if mdic & 1<<30 == 1 {
                debug!("mdi read e");
                return 0;
            }
            if mdic & 1<<28 == 0 {
                debug!("read mdi done");
                return mdic&0xFFFF;
            }
        }
    }

    pub fn write_mdi(&self,reg_addr: u32, value: u32) {
        if value > 0xFFFF {
            panic!("value too big");
        }
        //* W: 1<<29 0b  */
        let mdic = 0x0000_0000 | 
                        reg_addr<<16 | 
                        Self::MDIO_ADDRESS<<21 | 
                        0b01<<26 | 
                        1<<29 | 
                        value;
        unsafe { self.bar0.add(0x00020).cast::<u32>().write_volatile(mdic) };
        loop {
            let mdic = unsafe { self.bar0.add(0x00020).cast::<u32>().read_volatile() };
            if mdic & 1<<30 == 1 {
                debug!("mdi write e");
                return;
            }
            if mdic & 1<<28 == 1 {
                debug!("write mdi done");
                return;
            }
        }
    }

    pub fn phy_link_setup(&self) {
        let mut ctrl_ext = unsafe { self.bar0.add(0x00018).cast::<u32>().read_volatile() };
        ctrl_ext = ctrl_ext & !(0b11<<22) | 0b00<<22;
        unsafe { self.bar0.add(0x00018).cast::<u32>().write_volatile(ctrl_ext) };
        debug!("phy link mode direct copper");
        let ana = self.read_mdi(4);
        let anbpa = self.read_mdi(5);
        let local = ana & 0b11<<10;
        let partner = anbpa & 0b11<<10;
        match local {
            0 => todo!(),
            default => todo!()
        }
    }

    pub fn serdes_link_setup(&self) {
        todo!()
    }

    pub fn sgmii_link_setup(&self) {
        todo!()
    }

}

pub struct IgbNetBuf {

}

pub struct IgbDevice {

}