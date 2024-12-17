use log::{debug, error};
use IgbFlowControlType::*;

use crate::{
    err::IgbError,
    regs::{Reg, SwFwSync, MDIC, SWSM},
};

//* PHY Reg */
const PHY_CONTROL: u32 = 0;
const PHY_AUTO_NEGOTIATION_ADVERTISEMENT: u32 = 4;
const PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY: u32 = 5;
//* PHY Reg Mask */
//* PHY_AUTO_NEGOTIATION_ADVERTISEMENT Mask */
const PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE: u16 = 1<<10;
const PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR: u16 = 1<<11;
//* PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY Mask */
const PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE: u16 = 1<<10;
const PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_ASM_DIR: u16 = 1<<11;

const MII_CR_POWER_DOWN: u16 = 0x0800;
const MII_CR_AUTO_NEG_EN: u16 = 1<<12;
const MII_CR_RESTART_AUTO_NEG: u16 = 1<<9;


pub struct Phy {
    reg: Reg,
    addr: u32,
}

pub enum IgbFlowControlType{
    IgbFCNone,
    IgbFCTxPause,
    IgbFCRxPause,
    IgbFCFull
}

impl Phy {
    pub fn new(reg: Reg) -> Self {
        // let mdic = reg.read_reg::<MDIC>();
        // let addr = (mdic.bits() & MDIC::PHYADD.bits()) >> 21;
        // debug!("phy addr: {}", addr);
        Self { reg, addr: 1 }
    }

    pub fn read_mdic(&self, offset: u32) -> Result<u16, IgbError> {
        let mut mdic = MDIC::from_bits_retain((offset << 16) | (self.addr << 21)) | MDIC::OP_READ;
        self.reg.write_reg(mdic);

        loop {
            mdic = self.reg.read_reg::<MDIC>();
            if mdic.contains(MDIC::READY) {
                break;
            }
            if mdic.contains(MDIC::E) {
                error!("MDIC read error");
                return Err(IgbError::Unknown);
            }
        }

        Ok(mdic.bits() as u16)
    }

    pub fn write_mdic(&self, offset: u32, data: u16) -> Result<(), IgbError> {
        let mut mdic = MDIC::from_bits_retain((offset << 16) | (self.addr << 21) | (data as u32))
            | MDIC::OP_WRITE;
        self.reg.write_reg(mdic);

        loop {
            mdic = self.reg.read_reg::<MDIC>();
            if mdic.contains(MDIC::READY) {
                break;
            }
            if mdic.contains(MDIC::E) {
                error!("MDIC read error");
                return Err(IgbError::Unknown);
            }
        }

        Ok(())
    }

    pub fn aquire_sync(&self, mask: u16) -> Result<Synced, IgbError> {
        Synced::new(self.reg, mask)
    }

    pub fn power_up(&self) -> Result<(), IgbError> {
        let mut mii_reg = self.read_mdic(PHY_CONTROL)?;
        mii_reg &= !MII_CR_POWER_DOWN;
        mii_reg |= MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG;
        self.write_mdic(PHY_CONTROL, mii_reg)
    }

    pub fn get_fc_type(&self) -> Result<IgbFlowControlType, IgbError> {
        let ana = self.read_mdic(PHY_AUTO_NEGOTIATION_ADVERTISEMENT)?;
        let anbpa = self.read_mdic(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY)?;
        /* Two bits in the Auto Negotiation Advertisement Register
		 * (Address 4) and two bits in the Auto Negotiation Base
		 * Page Ability Register (Address 5) determine flow control
		 * for both the PHY and the link partner.  The following
		 * table, taken out of the IEEE 802.3ab/D6.0 dated March 25,
		 * 1999, describes these PAUSE resolution bits and how flow
		 * control is determined based upon these settings.
		 * NOTE:  DC = Don't Care
		 *
		 *   LOCAL DEVICE  |   LINK PARTNER
		 * PAUSE | ASM_DIR | PAUSE | ASM_DIR | NIC Resolution
		 *-------|---------|-------|---------|--------------------
		 *   0   |    0    |  DC   |   DC    | e1000_fc_none
		 *   0   |    1    |   0   |   DC    | e1000_fc_none
		 *   0   |    1    |   1   |    0    | e1000_fc_none
		 *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
		 *   1   |    0    |   0   |   DC    | e1000_fc_none
		 *   1   |   DC    |   1   |   DC    | e1000_fc_full
		 *   1   |    1    |   0   |    0    | e1000_fc_none
		 *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
		 *
		 * Are both PAUSE bits set to 1?  If so, this implies
		 * Symmetric Flow Control is enabled at both ends.  The
		 * ASM_DIR bits are irrelevant per the spec.
		 *
		 * For Symmetric Flow Control:
		 *
		 *   LOCAL DEVICE  |   LINK PARTNER
		 * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
		 *-------|---------|-------|---------|--------------------
		 *   1   |   DC    |   1   |   DC    | E1000_fc_full
		 *
		 */
        if !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE) 
        && !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR){
            Ok(IgbFCNone)
        }else if !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE) 
        && ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE){
            Ok(IgbFCNone)
        }else if !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_ASM_DIR){
            Ok(IgbFCNone)
        }else if !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE)
        && anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_ASM_DIR){
            Ok(IgbFCTxPause)
        }else if ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && !ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE){
            Ok(IgbFCNone)
        }else if ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE) {
            Ok(IgbFCFull)
        }else if ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_ASM_DIR) {
            Ok(IgbFCNone)
        }else if ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_PAUSE)
        && ana.check_bit(PHY_AUTO_NEGOTIATION_ADVERTISEMENT_ASM_DIR)
        && !anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_PAUSE)
        && anbpa.check_bit(PHY_AUTO_NEGOTIATION_BASE_PAGE_ABILITY_ASM_DIR) {
            Ok(IgbFCRxPause)
        }else {
            Err(IgbError::NotImplemented)
        }
    }
}

trait CheckBit {
    fn check_bit(&self, mask: u16) -> bool;
}

impl CheckBit for u16 {
    fn check_bit(&self, mask: u16) -> bool {
        (self & mask) != 0
    }
}


pub struct Synced {
    reg: Reg,
    mask: u16,
}

impl Synced {
    pub fn new(reg: Reg, mask: u16) -> Result<Self, IgbError> {
        let semaphore = Semaphore::new(reg)?;
        let swmask = mask as u32;
        let fwmask = (mask as u32) << 16;
        let mut swfw_sync = SwFwSync::empty();
        loop {
            swfw_sync = reg.read_reg::<SwFwSync>();
            if (swfw_sync.bits() & (swmask | fwmask)) == 0 {
                break;
            }
        }

        swfw_sync |= SwFwSync::from_bits_retain(swmask);
        reg.write_reg(swfw_sync);

        drop(semaphore);
        Ok(Self { reg, mask })
    }
}

impl Drop for Synced {
    fn drop(&mut self) {
        let semaphore = Semaphore::new(self.reg).unwrap();
        let mask = self.mask as u32;
        self.reg.modify_reg(|mut swfw_sync: SwFwSync| {
            swfw_sync.remove(SwFwSync::from_bits_retain(mask));
            swfw_sync
        });

        drop(semaphore);
    }
}

pub struct Semaphore {
    reg: Reg,
}

impl Semaphore {
    pub fn new(reg: Reg) -> Result<Self, IgbError> {
        loop {
            let swsm = reg.read_reg::<SWSM>();

            reg.write_reg(swsm | SWSM::SWESMBI);

            if reg.read_reg::<SWSM>().contains(SWSM::SWESMBI) {
                break;
            }
        }

        Ok(Self { reg })
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        self.reg.modify_reg(|mut reg: SWSM| {
            reg.remove(SWSM::SMBI);
            reg.remove(SWSM::SWESMBI);
            reg
        });
    }
}
