use crate::{mem::Mem, rom::Rom};

#[derive(Debug)]
pub struct Bus {
    ram: [u8; 2048],
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Self {
            ram: [0; 2048],
            rom,
        }
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl Mem for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => self.ram[(addr & 0x7FF) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                error!("PPU registers not implemented");
                0
            }
            0x8000..=0xFFFF => {
                let addr = (addr - 0x8000) as usize;
                if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    self.rom.prg_rom[addr % 0x4000]
                } else {
                    self.rom.prg_rom[addr]
                }
            }
            _ => {
                debug!("Unmapped read at address {:#06x}", addr);
                0
            }
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => self.ram[(addr & 0x7FF) as usize] = value,
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => error!("PPU not supported yet"),
            0x8000..=0xFFFF => panic!("Attempted to write to cartridge rom address {:#06x}", addr),
            _ => debug!("Unmapped write at address {:#06x}", addr),
        }
    }
}
