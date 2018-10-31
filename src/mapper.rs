use rom::{INesHeader, Rom};

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

// func (m *Mapper2) Write(address uint16, value byte) {
// 	switch {
// 	case address < 0x2000:
// 		m.CHR[address] = value
// 	case address >= 0x8000:
// 		m.prgBank1 = int(value) % m.prgBanks
// 	case address >= 0x6000:
// 		index := int(address) - 0x6000
// 		m.SRAM[index] = value
// 	default:
// 		log.Fatalf("unhandled mapper2 write at address: 0x%04X", address)
// 	}
// }

// func (m *Mapper2) Read(address uint16) byte {
// 	switch {
// 	case address < 0x2000:
// 		return m.CHR[address]
// 	case address >= 0xC000:
// 		index := m.prgBank2*0x4000 + int(address-0xC000)
// 		return m.PRG[index]
// 	case address >= 0x8000:
// 		index := m.prgBank1*0x4000 + int(address-0x8000)
// 		return m.PRG[index]
// 	case address >= 0x6000:
// 		index := int(address) - 0x6000
// 		return m.SRAM[index]
// 	default:
// 		log.Fatalf("unhandled mapper2 read at address: 0x%04X", address)
// 	}
// 	return 0
// }
