use crate::controller::Controller;
use crate::mapper::Mapper;
use crate::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;



/// Connects all the NES components
pub struct Bus {
    ram: [u8; RAM_SIZE],
    mapper: Rc<RefCell<Box<Mapper>>>,
    pub controller: Controller,
    ppu: Rc<RefCell<Ppu>>,
}

impl Bus {
    pub fn new(
        mapper: Rc<RefCell<Box<Mapper>>>,
        controller: Controller,
        ppu: Rc<RefCell<Ppu>>,
    ) -> Bus {
        Bus {
            ram: [0; RAM_SIZE],
            mapper: mapper,
            controller: controller,
            ppu: ppu,
        }
    }

    /// Implements the CPU's memory map
    pub fn read(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram[addr as usize % RAM_SIZE]
        } else if addr < 0x4000 {
            self.ppu.borrow_mut().read_register(addr)
        } else if addr >= 0x6000 {
            self.mapper.borrow().read(addr)
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
            self.mapper.borrow_mut().write(addr, val);
        } else if addr == 0x4016 {
            self.controller.write(val);
        }
    }
}
