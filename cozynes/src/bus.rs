use crate::mem::Mem;

#[derive(Debug)]
pub struct Bus {
    cpu_vram: [u8; 2048],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            cpu_vram: [0; 2048],
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl Mem for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => self.cpu_vram[(addr & 0x7FF) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => todo!("PPU not supported yet"),
            _ => {
                debug!("Unmapped read at address {:#06x}", addr);
                0
            }
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => self.cpu_vram[(addr & 0x7FF) as usize] = value,
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => todo!("PPU not supported yet"),
            _ => debug!("Unmapped write at address {:#06x}", addr),
        }
    }
}
