use core::time::Duration;

use alloc::vec::Vec;
use dma_api::{DVec, Direction};

use crate::{descriptor::Descriptor, err::IgbError, regs::{rdbah, rdbal, rdlen, rxdctl, srrctl, Reg, RCTL, RXDCTL_ENABLE, SRRCTL_BSIZEPACKET_MASK}};

pub const DEFAULT_RING_SIZE: usize = 256;

pub struct Ring<D: Descriptor> {
    pub descriptors: DVec<D>,
    reg: Reg,
    //? 这里的大小可能还需要优化，暂时用u32
    count: u32,
    queue_index: u32,
    reg_idx: u32,
    rx_buffers: Vec<DVec<u8>>
}

impl<D: Descriptor> Ring<D> {
    pub fn new(reg: Reg, size: usize) -> Result<Self, IgbError> {
        let descriptors =
            DVec::zeros(size, 4096, Direction::Bidirectional).ok_or(IgbError::NoMemory)?;

        Ok(Self { 
            descriptors, 
            reg,
            count: 0,
            queue_index: 0,
            reg_idx: 0,
            rx_buffers: Vec::new()
        })
    }

    pub fn init(&mut self) {
        //* Allocate a region of memory for the receive descriptor list. */
        //? 已经在new的时候完成
        //* 初始化 recive buffers 的空间设置 */
        //* Receive buffers of appropriate size should be allocated and pointers to these buffers should be stored in the descriptor ring. */
        //? 这里我们直接不设置，采用RCTL.BSIZE的默认值 2048Bytes
        // let srrctl = self.reg.read_32(srrctl(self.reg_idx));
        // srrctl = srrctl & !(SRRCTL_BSIZEPACKET_MASK) & ;
        let buffer_size = if self.reg.read_32(srrctl(self.reg_idx)) & SRRCTL_BSIZEPACKET_MASK == 0{
            match self.reg.read_reg::<RCTL>() & RCTL::SZ_MASK {
                RCTL::SZ_2048 => 2048,
                RCTL::SZ_1024 => 1024,
                RCTL::SZ_512 => 512,
                RCTL::SZ_256 => 256,
                _ => 2048
            }
        }else {
            self.reg.read_32(srrctl(self.reg_idx)) & SRRCTL_BSIZEPACKET_MASK
        };
        let size = self.count * buffer_size;
        let rx_buffer = DVec::<u8>::zeros(size as usize, buffer_size as usize, Direction::Bidirectional).unwrap_or_else(|| panic!("Failed to allocate rx buffer"));
        self.rx_buffers.push(rx_buffer);
        //* 初始化 ring buffer 基地址寄存器 和 descriptor 长度寄存器 */
        let phy_addr = self.descriptors.bus_addr();
        self.reg.write_32(rdbal(self.reg_idx), (phy_addr & 0x00000000ffffffff) as u32);
        self.reg.write_32(rdbah(self.reg_idx), (phy_addr >> 32) as u32);
        self.reg.write_32(rdlen(self.reg_idx), self.count*128);
        //* Enable header split and header replication */
        //? 这里我们不设置
        //* enable queue */
        let mut rxdctl_value = self.reg.read_32(rxdctl(self.reg_idx));
        rxdctl_value |= RXDCTL_ENABLE;
        self.reg.write_32(rxdctl(self.reg_idx), rxdctl_value);
        loop {
            if self.reg.read_32(rxdctl(self.reg_idx)) & RXDCTL_ENABLE != 0 {
                break;
            }
        }
        
    }
}