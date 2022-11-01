#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

#[derive(Debug)]
pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
}

#[derive(Debug)]
pub enum Error {
    UnsupportedVersion,
    InvalidHeader,
    InvalidMapper,
}

const NES_TAG: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];

const PRG_ROM_PAGE_SIZE: usize = 16 * 1024;
const CHR_ROM_PAGE_SIZE: usize = 8 * 1024;

impl Rom {
    pub fn new(raw: &[u8]) -> Result<Rom, Error> {
        if raw[0..4] != NES_TAG {
            return Err(Error::InvalidHeader);
        }

        let mapper = (raw[6] >> 4) | (raw[7] & 0xf0);
        let ines_version = (raw[7] >> 2) & 0x01;

        if ines_version != 0 {
            return Err(Error::UnsupportedVersion);
        }

        let four_screen = (raw[6] & 0x08) != 0;
        let vertical_screen = (raw[6] & 0x01) != 0;

        let mirroring = match (four_screen, vertical_screen) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = (raw[6] & 0x04) != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Self {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            mirroring,
        })
    }
}
