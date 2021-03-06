use crate::rom::Rom;

pub fn init(rom: Rom) -> Box<Mapper> {
    match rom.header.mapper() {
        0 => Box::new(MapperZero::new(rom)),
        id @ _ => panic!("Unimplemented mapper {}", id),
    }
}

pub trait Mapper {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn step(&mut self);
}

pub struct MapperZero {
    rom: Rom,
    bank: usize,
}

impl MapperZero {
    pub fn new(rom: Rom) -> MapperZero {
        MapperZero { rom: rom, bank: 0 }
    }
}

impl Mapper for MapperZero {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x1FFF => self.rom.chr[addr as usize],
            0x6000...0x7FFF => self.rom.sram[(addr - 0x6000) as usize],
            0x8000...0xBFFF => self.rom.prg[self.bank * 0x4000 + (addr - 0x8000) as usize],
            a if a >= 0xC000 => {
                self.rom.prg
                    [(self.rom.header.prg_rom_size as usize - 1) * 0x4000 + (a - 0xC000) as usize]
            }
            _ => panic!("NO!"),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x2000 => self.rom.chr[addr as usize] = val,
            0x6000...0x8000 => self.rom.sram[(addr - 0x6000) as usize] = val,
            a if a >= 0x8000 => self.bank = (val % self.rom.header.prg_ram_size) as usize,
            _ => println!("Invalid write location {}", addr),
        }
    }

    fn step(&mut self) {}
}
