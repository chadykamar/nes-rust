#[macro_use]
extern crate bitfield;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate ndarray;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub mod cpu;
pub mod mapper;
pub mod ppu;
pub mod rom;

#[macro_use]
pub mod util;

use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;
use rom::Rom;

use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};


/// Initializes and configures logging using log4rs
fn init_logging() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("logs/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

/// Starts the emulator
pub fn start(rom: Rom) {
    // let rom = Box::new(rom);

    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // let filename = format!("./logs/nes_{:?}.log", timestamp);

    // TODO handle errors

    init_logging();

    println!("Loaded ROM: {}", rom.header);

    let mapper = Mapper::new(rom);
    let mut cpu = Cpu::new(mapper);
    let ppu = Ppu::new();

    cpu.reset();

    loop {
        cpu.step();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use File;

    use Cpu;
    use Mapper;
    use Rom;

    #[test]
    fn golden_log() {
        let path = Path::new("roms/nestest.nes");
        let rom = Rom::load(&mut File::open(&path).unwrap()).unwrap();
        let mapper = Mapper::new(rom);
        let mut cpu = Cpu::new(mapper);

        let file = File::open("roms/nestest.log").unwrap();
        let buf_reader = BufReader::new(file);
        for (i, line) in buf_reader.lines().enumerate() {
            println!("{:?}", i + 1);
            let line = line.unwrap();
            let mut split = line.split_whitespace();

            let registers = cpu.registers();

            // pc
            let mut token = split.next().unwrap();
            let pc = u16::from_str_radix(token, 16).unwrap();
            assert_eq!(pc, registers.0);

            token = split.next().unwrap();
            while !token.starts_with("A:") {
                token = split.next().unwrap();
            }

            // a
            let a = u8::from_str_radix(&token[2..], 16).unwrap();
            assert_eq!(a, registers.2);

            // x
            token = split.next().unwrap();
            let x = u8::from_str_radix(&token[2..], 16).unwrap();
            assert_eq!(x, registers.3);

            // y
            token = split.next().unwrap();
            let y = u8::from_str_radix(&token[2..], 16).unwrap();
            assert_eq!(y, registers.4);

            // p
            token = split.next().unwrap();
            let p = u8::from_str_radix(&token[2..], 16).unwrap();
            assert_eq!(p, registers.5);

            // sp
            token = split.next().unwrap();
            let sp = u8::from_str_radix(&token[3..], 16).unwrap();
            assert_eq!(sp, registers.1);

            // cyc
            token = split.next().unwrap();
            if token == "CYC:" {
                token = split.next().unwrap();
                let cycles = token.parse::<usize>().unwrap();
                assert_eq!(cycles, registers.6);
            } else {
                let cycles = token[4..].parse::<usize>().unwrap();
                assert_eq!(cycles, registers.6);
            }

            cpu.step();
        }
    }

}
