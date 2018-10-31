#[macro_use]
extern crate bitfield;

#[macro_use]
extern crate log;
extern crate log4rs;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Errors, Root};
use log4rs::encode::pattern::PatternEncoder;

pub mod cpu;
pub mod mapper;
pub mod rom;

#[macro_use]
pub mod util;

use cpu::Cpu;
use mapper::Mapper;
use rom::Rom;

use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn start(rom: Rom) {
    // let rom = Box::new(rom);

    let current_time = SystemTime::now();
    let timestamp = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let filename = format!("./logs/nes_{:?}.log", timestamp);

    // TODO handle errors

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("logs/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    trace!("Loaded ROM: {}", rom.header);

    let mapper = Mapper::new(rom);
    let mut cpu = Cpu::new(mapper);

    // TODO: No reset when running nestest
    // cpu.reset();

    loop {
        cpu.step();
    }
}
