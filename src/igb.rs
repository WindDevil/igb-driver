use core::{ptr::NonNull, time::Duration};

use log::{debug, info};
use LinkMode::*;

use crate::{
    descriptor::{AdvRxDesc, AdvTxDesc},
    err::IgbError,
    phy::Phy,
    phy::IgbFlowControlType,
    regs::{Reg, CTRL, CTRL_EXT, RCTL, STATUS, TCTL},
    ring::{Ring, DEFAULT_RING_SIZE},
};

pub enum LinkMode {
    DirectCopper,
    SGMMII,
    InternalSerDesInterface
} 

pub struct Igb {
    reg: Reg,
    tx_ring: Ring<AdvTxDesc>,
    rx_ring: Ring<AdvRxDesc>,
    phy: Phy,
}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Result<Self, IgbError> {
        let reg = Reg::new(bar0);
        let tx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;
        let rx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;

        Ok(Self {
            reg,
            tx_ring,
            rx_ring,
            phy: Phy::new(reg),
        })
    }

    pub fn open(&mut self) -> Result<(), IgbError> {
        self.reg.disable_interrupts();

        self.reg.write_reg(CTRL::RST);

        self.reg.wait_for(
            |reg: CTRL| !reg.contains(CTRL::RST),
            Duration::from_millis(1),
            Some(1000),
        )?;
        self.reg.disable_interrupts();

        self.reg
            .modify_reg(|reg: CTRL_EXT| CTRL_EXT::DRV_LOAD | reg);

        self.setup_phy_and_the_link()?;

        self.init_stat();

        self.init_rx();
        self.init_tx();

        self.enable_interrupts();

        // self.reg
        //     .write_reg(CTRL::SLU | CTRL::FD | CTRL::SPD_1000 | CTRL::FRCDPX | CTRL::FRCSPD);
        self.reg
            .write_reg(CTRL::SLU | CTRL::FD | CTRL::SPD_1000);
        Ok(())
    }

    fn init_stat(&mut self) {
        //TODO
    }
    /// 4.5.9 Receive Initialization
    fn init_rx(&mut self) {
        // disable rx when configing.
        self.reg.write_reg(RCTL::empty());

        // set up multicast table array.
        self.reg.setup_mta();

        self.rx_ring.init();
         
        // it is best to leave the receive logic disabled (EN = 0b) until after the receive descriptor ring has been initialized
        // If VLANs are not used, software should clear VFE.
        self.reg.write_reg(RCTL::RXEN | RCTL::SZ_256);
    }

    fn init_tx(&mut self) {
        self.reg.write_reg(TCTL::empty());

        self.tx_ring.init();

        self.reg.write_reg(TCTL::EN);
    }

    fn set_link_mode(&mut self, mode:LinkMode) -> Result<(), IgbError>{
        match mode {
            DirectCopper => {
                self.reg.modify_reg(|reg| CTRL_EXT::LINK_MODE_DIRECT_COPPER|reg);
            },
            SGMMII => {
                self.reg.modify_reg(|reg| CTRL_EXT::LINK_MODE_SGMII|reg);
            },
            InternalSerDesInterface =>{
                self.reg.modify_reg(|reg| CTRL_EXT::LINK_MODE_INTERNAL_SERDES_INTERFACE|reg);
            },
        }
        Ok(())
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), IgbError> {
        info!("set phy and the link!");
        self.set_link_mode(DirectCopper)?;
        self.phy.power_up()?;
        self.setup_flow_control_mode()?;
        debug!("end of fn");
        Ok(())
    }

    fn setup_flow_control_mode(&mut self)-> Result<(), IgbError>{
        let fc_type = self.phy.get_fc_type()?;
        match fc_type {
            IgbFlowControlType::IgbFCNone => {
                self.reg.modify_reg(|reg:CTRL| reg & !(CTRL::RFCE|CTRL::TFCE));
            },
            IgbFlowControlType::IgbFCRxPause => {
                self.reg.modify_reg(|reg:CTRL| reg & !(CTRL::RFCE) | CTRL::TFCE );
            },
            IgbFlowControlType::IgbFCTxPause => {
                self.reg.modify_reg(|reg:CTRL|  reg &!(CTRL::TFCE) | CTRL::RFCE );
            },
            IgbFlowControlType::IgbFCFull => {
                self.reg.modify_reg(|reg:CTRL| reg | (CTRL::RFCE | CTRL::TFCE) );
            }
        }
        Ok(())
    }

    fn setup_vlan(&mut self) {
        self.reg.write_reg(RCTL::VFE);
        //TODO setup VFTA
        
    }

    pub fn mac(&self) -> [u8; 6] {
        self.reg.read_mac()
    }

    fn enable_interrupts(&self) {
        //TODO
    }

    pub fn status(&self) -> IgbStatus {
        let raw = self.reg.read_reg::<STATUS>();
        let speed_raw = (raw.bits() >> 6) & 0b11;

        IgbStatus {
            link_up: raw.contains(STATUS::LU),
            speed: match speed_raw {
                0 => Speed::Mb10,
                1 => Speed::Mb100,
                0b10 => Speed::Mb1000,
                _ => Speed::Mb1000,
            },
            full_duplex: raw.contains(STATUS::FD),
            phy_reset_asserted: raw.contains(STATUS::PHYRA),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IgbStatus {
    pub full_duplex: bool,
    pub link_up: bool,
    pub speed: Speed,
    pub phy_reset_asserted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Mb10,
    Mb100,
    Mb1000,
}
