#[macro_use]
extern crate bitfield;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate ndarray;

extern crate sdl2;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

pub mod controller;
pub mod cpu;
pub mod mapper;
pub mod ppu;
pub mod rom;

#[macro_use]
pub mod util;

use controller::Controller;
use cpu::Cpu;
use mapper::{Mapper, MapperZero};
use ppu::Ppu;
use ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use rom::Rom;

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
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

fn sdl_start() {}

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

    // TODO initilize mapper from heaader

    // FIXME Refactor to not use RefCell if possible
    let mapper = mapper::init(rom);

    let mapper = Rc::new(RefCell::new(mapper));

    let controller = Rc::new(RefCell::new(Controller::new()));

    let mut cpu = Cpu::new(mapper.clone(), controller.clone());
    let mut ppu = Ppu::new(mapper.clone());

    cpu.reset();

    let sdl_context = sdl2::init().unwrap();
    sdl_context.mouse().show_cursor(false);

    let video_subsystem = sdl_context.video().unwrap();

    // let audio_subsystem = sdl_context.audio().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem
        .window("NES", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        ).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Z => controller.borrow_mut().buttons.set_a(true),
                    Keycode::X => controller.borrow_mut().buttons.set_b(true),
                    Keycode::Backspace => controller.borrow_mut().buttons.set_select(true),
                    Keycode::Return => controller.borrow_mut().buttons.set_start(true),
                    Keycode::Up => controller.borrow_mut().buttons.set_up(true),
                    Keycode::Down => controller.borrow_mut().buttons.set_down(true),
                    Keycode::Left => controller.borrow_mut().buttons.set_left(true),
                    Keycode::Right => controller.borrow_mut().buttons.set_right(true),
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Z => controller.borrow_mut().buttons.set_a(false),
                    Keycode::X => controller.borrow_mut().buttons.set_b(false),
                    Keycode::Backspace => controller.borrow_mut().buttons.set_select(false),
                    Keycode::Return => controller.borrow_mut().buttons.set_start(false),
                    Keycode::Up => controller.borrow_mut().buttons.set_up(false),
                    Keycode::Down => controller.borrow_mut().buttons.set_down(false),
                    Keycode::Left => controller.borrow_mut().buttons.set_left(false),
                    Keycode::Right => controller.borrow_mut().buttons.set_right(false),
                    _ => {}
                },
                _ => {}
            }
        }
        canvas.present();
        let cpu_cycles = cpu.step();

        for _ in 0..cpu_cycles {
            ppu.step();
        }

        texture
            .update(None, &ppu.screen, SCREEN_WIDTH as usize * 3)
            .unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use std::rc::Rc;
    use File;

    use Controller;
    use Cpu;
    use Rom;
    use {Mapper, MapperZero};

    #[test]
    fn golden_log() {
        let path = Path::new("test_roms/nestest.nes");
        let rom = Rom::load(&mut File::open(&path).unwrap()).unwrap();
        let mapper: Box<Mapper> = Box::new(MapperZero::new(rom));
        let mapper = Rc::new(RefCell::new(mapper));
        let controller = Rc::new(RefCell::new(Controller::new()));
        let mut cpu = Cpu::new(mapper, controller);

        let file = File::open("test_roms/nestest.log").unwrap();
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
