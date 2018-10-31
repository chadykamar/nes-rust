use rom::Rom;

pub struct Mapper {
    rom: Rom,
    bank: usize,
}

impl Mapper {
    pub fn new(rom: Rom) -> Mapper {
        Mapper { rom: rom, bank: 0 }
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x1FFF => return self.rom.chr[addr as usize],
            0x6000...0x7FFF => return self.rom.sram[(addr - 0x6000) as usize],
            0x8000...0xBFFF => return self.rom.prg[self.bank * 0x4000 + (addr - 0x8000) as usize],
            a if a >= 0xC000 => {
                return self.rom.prg
                    [(self.rom.header.prg_rom_size as usize - 1) * 0x4000 + (a - 0xC000) as usize]
            }
            _ => panic!("NO!"),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x2000 => self.rom.chr[addr as usize] = val,
            0x6000...0x8000 => self.rom.sram[(addr - 0x6000) as usize] = val,
            a if a >= 0x8000 => self.bank = (val % self.rom.header.prg_ram_size) as usize,
            _ => println!("Invalid write location {}", addr),
        }
    }
}
