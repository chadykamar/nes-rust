

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

use crate::controller::Controller;
use crate::cpu::Cpu;
use crate::mapper::{Mapper, MapperZero};
use crate::ppu::Ppu;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::rom::Rom;

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

    let mapper = mapper::init(rom);
    // let mut ppu = Ppu::new();
    // let mut ppu = Rc::new(RefCell::new(ppu));
    let controller = Controller::default();

    let mut cpu = Cpu::new(mapper, controller);

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
                    Keycode::Z => cpu.controller.buttons.set_a(true),
                    Keycode::X => cpu.controller.buttons.set_b(true),
                    Keycode::Backspace => cpu.controller.buttons.set_select(true),
                    Keycode::Return => cpu.controller.buttons.set_start(true),
                    Keycode::Up => cpu.controller.buttons.set_up(true),
                    Keycode::Down => cpu.controller.buttons.set_down(true),
                    Keycode::Left => cpu.controller.buttons.set_left(true),
                    Keycode::Right => cpu.controller.buttons.set_right(true),
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Z => cpu.controller.buttons.set_a(false),
                    Keycode::X => cpu.controller.buttons.set_b(false),
                    Keycode::Backspace => cpu.controller.buttons.set_select(false),
                    Keycode::Return => cpu.controller.buttons.set_start(false),
                    Keycode::Up => cpu.controller.buttons.set_up(false),
                    Keycode::Down => cpu.controller.buttons.set_down(false),
                    Keycode::Left => cpu.controller.buttons.set_left(false),
                    Keycode::Right => cpu.controller.buttons.set_right(false),
                    _ => {}
                },
                _ => {}
            }
        }
        canvas.present();
        let cpu_cycles = cpu.step();

        // for _ in 0..cpu_cycles {
        //     ppu.borrow_mut().step();
        // }

        // texture
        //     .update(None, &ppu.borrow_mut().screen, SCREEN_WIDTH as usize * 3)
        //     .unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
