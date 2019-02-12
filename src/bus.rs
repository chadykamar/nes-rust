
use controller::Controller;
use mapper::Mapper;
use ppu::Ppu;

/// Ram Size
const RAM_SIZE: usize = 0x800;

/// Connects all the NES components
pub struct Bus {
    ram: [u8; RAM_SIZE],
    mapper: Box<Mapper>,
    pub controller: Controller,
    ppu: Ppu,
}

impl Bus {

    pub fn new(mapper: Box<Mapper>, controller:Controller, ppu: Ppu) -> Bus {
        Bus{
            ram: [0; RAM_SIZE],
            mapper: mapper,
            controller: controller,
            ppu: ppu
        }
    }

    /// Implements the CPU's memory map
    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram[addr as usize % RAM_SIZE]
        } else if addr < 0x4000 {
            self.ppu.read_register(addr)
        } else if addr >= 0x6000 {
            self.mapper.read(addr)
        } else if addr == 0x4016 {
            self.controller.read()
        } else {
            unimplemented!()
        }
    }

    /// Implements the CPU's memory map
    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram[(addr % 0x800) as usize] = val;
        } else if addr >= 0x6000 {
            self.mapper.write(addr, val);
        } else if addr == 0x4016 {
            self.controller.write(val);
        }
    }
}