pub trait Mem {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, value: u8);

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;
        self.write_byte(addr, lo);
        self.write_byte(addr.wrapping_add(1), hi);
    }
}
