use crate::{err::IgbError, regs::{rxdctl, srrctl, txdctl, Reg, RCTL, RXDCTL_ENABLE, SRRCTL_BSIZEPACKET_MASK, TXDCTL_ENABLE, TXDCTL_WTHRESH_MASK, TXDCTL_WTHRESH_MASK_1B}};

pub trait Descriptor {
    fn new() -> Self;
    fn buffer_size(reg:Reg,reg_idx:u32) -> Result<u32,IgbError>;
    fn enable_queue(reg:Reg,reg_idx:u32);
}

#[derive(Clone, Copy)]
pub union AdvTxDesc {
    pub read: AdvTxDescRead,
    pub write: AdvTxDescWB,
}

impl Descriptor for AdvTxDesc {
    fn new() -> Self {
        AdvTxDesc {
            read: AdvTxDescRead {
                buffer_addr: 0,
                cmd_type_len: 0,
                olinfo_status: 0,
            },
        }
    }
    fn buffer_size(reg:Reg,reg_idx:u32) -> Result<u32,IgbError> {
        Err(IgbError::Unknown)
    }
    fn enable_queue(reg:Reg,reg_idx:u32) {
        let mut txdctl_value = reg.read_32(txdctl(reg_idx));
        txdctl_value |= TXDCTL_ENABLE;
        txdctl_value &= !TXDCTL_WTHRESH_MASK;
        txdctl_value |= TXDCTL_WTHRESH_MASK_1B;
        reg.write_32(txdctl(reg_idx), txdctl_value);
        loop {
            if reg.read_32(txdctl(reg_idx)) & TXDCTL_ENABLE != 0 {
                break;
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescRead {
    pub buffer_addr: u64,
    pub cmd_type_len: u32,
    pub olinfo_status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescWB {
    pub rsvd: u64,
    pub nxtseq_seed: u32,
    pub status: u32,
}

#[derive(Clone, Copy)]
pub union AdvRxDesc {
    pub read: AdvRxDescRead,
    pub write: AdvRxDescWB,
}

impl Descriptor for AdvRxDesc {
    fn new() -> Self {
        AdvRxDesc {
            read: AdvRxDescRead {
                pkt_addr: 0,
                hdr_addr: 0,
            },
        }
    }
    fn buffer_size(reg: Reg, reg_idx: u32) -> Result<u32, IgbError> {
        if reg.read_32(srrctl(reg_idx)) & SRRCTL_BSIZEPACKET_MASK == 0{
            match reg.read_reg::<RCTL>() & RCTL::SZ_MASK {
                RCTL::SZ_2048 => Ok(2048),
                RCTL::SZ_1024 => Ok(1024),
                RCTL::SZ_512 => Ok(512),
                RCTL::SZ_256 => Ok(256),
                _ => Err(IgbError::Unknown),
            }
        }else {
            Ok(reg.read_32(srrctl(reg_idx)) & SRRCTL_BSIZEPACKET_MASK)
        }
    }
    fn enable_queue(reg:Reg,reg_idx:u32) {
        let mut rxdctl_value = reg.read_32(rxdctl(reg_idx));
        rxdctl_value |= RXDCTL_ENABLE;
        reg.write_32(rxdctl(reg_idx), rxdctl_value);
        loop {
            if reg.read_32(rxdctl(reg_idx)) & RXDCTL_ENABLE != 0 {
                break;
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescRead {
    pub pkt_addr: u64,
    pub hdr_addr: u64,
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescWB {
    pub lo_dword: LoDword,
    pub hi_dword: HiDword,
    pub status_error: u32,
    pub length: u16,
    pub vlan: u16,
}

impl AdvRxDescRead {
    pub fn set_packet_address(&mut self, packet_buffer_address: u64) {
        self.pkt_addr = packet_buffer_address;
    }
}

#[derive(Clone, Copy)]
pub union LoDword {
    pub data: u32,
    pub hs_rss: HsRss,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HsRss {
    pub pkt_info: u16,
    pub hdr_info: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union HiDword {
    pub rss: u32, // RSS Hash
    pub csum_ip: CsumIp,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CsumIp {
    pub ip_id: u16, // IP id
    pub csum: u16,  // Packet Checksum
}
