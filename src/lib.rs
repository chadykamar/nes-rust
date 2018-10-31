#[macro_use]
extern crate bitfield;
pub mod cpu;
pub mod mapper;
pub mod rom;

#[macro_use]
pub mod util;

use cpu::Cpu;
use mapper::Mapper;
use rom::Rom;

pub fn start(rom: Rom) {
    // let rom = Box::new(rom);
    println!("Loaded ROM: {}", rom.header);

    let mut mapper = Mapper::new(rom);
    let mut cpu = Cpu::new(mapper);

    // TODO: No reset when running nestest
    // cpu.reset();

    loop {
        cpu.step();
    }
}
