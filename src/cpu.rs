use bitfield::bitfield;
use bitfield::BitRange;
use log::Level::Debug;
use log::{info, log_enabled};

use crate::controller::Controller;
use crate::mapper::{init, Mapper};
use crate::ppu::Ppu;

use std::cell::RefCell;
use std::rc::Rc;

/// Stack offset
const STACK: u16 = 0x100;
/// NMI vector
const NMI_VECTOR: u16 = 0xFFFA;
/// Reset vector
const RESET_VECTOR: u16 = 0xFFFC;
/// IRQ/BRK vector
const IRQ_BRK_VECTOR: u16 = 0xFFFE;

/// Ram Size
const RAM_SIZE: usize = 0x800;

/// Instruction mode corresponding to each opcode as resolved by the `resolve_address` function.
static INSTRUCTION_MODES: [usize; 256] = [
    6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2,
    2, 2, 2, 1, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3,
    6, 3, 2, 2, 2, 2, 6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12,
    12, 6, 3, 6, 3, 2, 2, 2, 2, 6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 8, 1, 1, 1, 10, 9, 6, 9,
    12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1, 10,
    9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1,
    1, 1, 10, 9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6,
    5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2, 5, 7, 5, 7, 11, 11, 11, 11,
    6, 5, 6, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
];

/// The number of bytes of each instruction in bytes
static INSTRUCTION_SIZES: [usize; 256] = [
    1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 0, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
    3, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 0, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
    1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 0, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
    1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 0, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
    2, 2, 0, 2, 2, 2, 2, 2, 1, 0, 1, 0, 3, 3, 3, 3, 2, 2, 0, 0, 2, 2, 2, 2, 1, 3, 1, 0, 0, 3, 0, 0,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 0, 3, 3, 3, 3,
    2, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 0, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
    2, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3, 2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,
];

static INSTRUCTION_CYCLES: [usize; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

/// The number of cycles used by each instruction when a page is crossed
static CYCLES_PAGE_CROSS: [usize; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0,
];

/// Name of the instruction
static INSTRUCTION_NAMES: [&str; 256] = [
    "BRK", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO", "PHP", "ORA", "ASL", "ANC", "NOP",
    "ORA", "ASL", "SLO", "BPL", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO", "CLC", "ORA",
    "NOP", "SLO", "NOP", "ORA", "ASL", "SLO", "JSR", "AND", "KIL", "RLA", "BIT", "AND", "ROL",
    "RLA", "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA", "BMI", "AND", "KIL", "RLA",
    "NOP", "AND", "ROL", "RLA", "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA", "RTI",
    "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE", "PHA", "EOR", "LSR", "ALR", "JMP", "EOR",
    "LSR", "SRE", "BVC", "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE", "CLI", "EOR", "NOP",
    "SRE", "NOP", "EOR", "LSR", "SRE", "RTS", "ADC", "KIL", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA", "BVS", "ADC", "KIL", "RRA", "NOP",
    "ADC", "ROR", "RRA", "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA", "NOP", "STA",
    "NOP", "SAX", "STY", "STA", "STX", "SAX", "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX",
    "SAX", "BCC", "STA", "KIL", "AHX", "STY", "STA", "STX", "SAX", "TYA", "STA", "TXS", "TAS",
    "SHY", "STA", "SHX", "AHX", "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX", "TAY",
    "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX", "BCS", "LDA", "KIL", "LAX", "LDY", "LDA",
    "LDX", "LAX", "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX", "CPY", "CMP", "NOP",
    "DCP", "CPY", "CMP", "DEC", "DCP", "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "KIL", "DCP", "NOP", "CMP", "DEC", "DCP", "CLD", "CMP", "NOP", "DCP", "NOP",
    "CMP", "DEC", "DCP", "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC", "INX", "SBC",
    "NOP", "SBC", "CPX", "SBC", "INC", "ISC", "BEQ", "SBC", "KIL", "ISC", "NOP", "SBC", "INC",
    "ISC", "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

enum Interrupt {
    IRQ,
    NMI,
    None,
}

bitfield! {
    struct ProcessorStatus(u8);
    impl Debug;
    pub get_c, set_c: 0;
    pub get_z, set_z: 1;
    pub get_i, set_i: 2;
    pub get_d, set_d: 3;
    pub get_b, set_b: 4;
    pub get_v, set_v: 6;
    pub get_n, set_n: 7;
}

/// The CPU struct
pub struct Cpu {
    
    ram: [u8; RAM_SIZE],
    ppu: Rc<RefCell<Ppu>>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    pub controller: Controller,
    cycles: usize, // Cycles remaining
    stall: usize,  // Cycles to stall the CPU for (for catch-up)
    interrupt: Interrupt,
    // Registers
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    p: ProcessorStatus, // The status register is made up of 5 flags and 3 unused bits
}

impl Cpu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>, controller: Controller, ppu: Rc<RefCell<Ppu>>) -> Cpu {
        Cpu {
            ppu: ppu,
            mapper: mapper,
            controller: controller,

            ram: [0; RAM_SIZE],
            cycles: 0,
            stall: 0,
            interrupt: Interrupt::None,
            pc: 0xC000,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            p: ProcessorStatus(0x24),
        }
    }

    pub fn registers(&self) -> (u16, u8, u8, u8, u8, u8, usize) {
        (
            self.pc,
            self.sp,
            self.a,
            self.x,
            self.y,
            self.p.bit_range(7, 0),
            (self.cycles * 3) % 341,
        )
    }

    pub fn reset(&mut self) {
        self.p.set_bit_range(7, 0, 0x24);
        self.sp = 0xFD;
        self.pc = self.read16(RESET_VECTOR)
    }

    pub fn trigger_nmi(&mut self) {
        self.interrupt = Interrupt::NMI;
    }

    pub fn trigger_irq(&mut self) {
        if self.p.get_i() {
            self.interrupt = Interrupt::IRQ;
        }
    }

    fn read(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.ram[addr as usize % RAM_SIZE]
        } else if addr < 0x4000 {
            self.ppu.borrow_mut().read_register(addr)
            // unimplemented!()
        } else if addr >= 0x6000 {
            self.mapper.borrow_mut().read(addr)
        } else if addr == 0x4016 {
            self.controller.read()
        } else {
            unimplemented!()
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram[(addr % 0x800) as usize] = val;
        } else if addr >= 0x6000 {
            self.mapper.borrow_mut().write(addr, val);
        } else if addr == 0x4016 {
            self.controller.write(val);
        }
    }

    // Util

    // Sets the zero flag if the argument is zero
    fn check_zero(&mut self, val: u8) {
        self.p.set_z(val == 0);
    }

    // Sets the negative flag if the argument is negative (high bit is set)
    fn check_negative(&mut self, val: u8) {
        self.p.set_n(val & 0x80 != 0);
    }

    // Sets the negative flag if the argument is negative (high bit is set)
    // and the zero flag if the argument is zero
    fn check_negative_zero(&mut self, val: u8) {
        self.check_zero(val);
        self.check_negative(val);
    }

    fn check_same_page(addr1: u16, addr2: u16) -> bool {
        addr1 & 0xFF00 != addr2 & 0xFF00
    }

    // Stack

    fn push(&mut self, val: u8) {
        let sp = self.sp;
        self.write(STACK | u16::from(sp), val);
        self.sp -= 1;
    }

    fn pull(&mut self) -> u8 {
        self.sp += 1;
        self.read(STACK | u16::from(self.sp))
    }

    fn pull16(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        (high << 8) | low
    }

    fn push16(&mut self, val: u16) {
        let high = (val >> 8) as u8;
        let low = (val & 0xFF) as u8;
        self.push(high);
        self.push(low);
    }

    fn read16(&mut self, addr: u16) -> u16 {
        let low = u16::from(self.read(addr));
        let high = u16::from(self.read(addr + 1));
        (high << 8) | low
    }

    fn read16_wrap(&mut self, addr: u16) -> u16 {
        let low = u16::from(self.read(addr));
        let high = u16::from(self.read((addr & 0xFF00) as u16 | u16::from(addr as u8 + 1)));
        (high << 8) | low
    }

    /// Resolves the addressing mode for the given `opcode`.
    fn resolve_address(&mut self, opcode: u8) -> Option<u16> {
        match INSTRUCTION_MODES[opcode as usize] {
            1 => Some(self.absolute()),
            2 => Some(self.absolute_x(opcode)),
            3 => Some(self.absolute_y(opcode)),
            4 => None,
            5 => Some(self.immediate()),
            6 => None,
            7 => Some(self.indexed_indirect()),
            8 => Some(self.indirect()),
            9 => Some(self.indirect_indexed(opcode)),
            10 => Some(self.relative()),
            11 => Some(self.zero_page()),
            12 => Some(self.zero_page_x()),
            13 => Some(self.zero_page_y()),
            _ => None,
        }
    }

    /// Executes a CPU instruction
    pub fn step(&mut self) -> isize {
        if self.stall > 0 {
            self.stall -= 1;
            return 1;
        }

        self.trace();

        match self.interrupt {
            Interrupt::IRQ => self.irq(),
            Interrupt::NMI => self.nmi(),
            Interrupt::None => {}
        }
        self.interrupt = Interrupt::None;

        let cy = self.cycles;

        let opcode = self.read(self.pc);
        let cycles = INSTRUCTION_CYCLES[opcode as usize];

        let addressing_mode = self.resolve_address(opcode);

        self.pc += INSTRUCTION_SIZES[opcode as usize] as u16;
        self.cycles += cycles;

        self.exec(opcode, addressing_mode);

        (self.cycles - cy) as isize
    }

    /// Logs the current state of the CPU.
    fn trace(&mut self) {
        let opcode = self.read(self.pc) as usize;
        let bytes = INSTRUCTION_SIZES[opcode];
        let name = INSTRUCTION_NAMES[opcode];
        let first_byte = format!("{:02X}", self.read(self.pc));
        let mut second_byte = format!("{:02X}", self.read(self.pc + 1));
        let mut third_byte = format!("{:02X}", self.read(self.pc + 2));
        if bytes < 2 {
            second_byte = String::from("  ");
        }
        if bytes < 3 {
            third_byte = String::from("  ");
        }
        let p: u8 = self.p.bit_range(7, 0);
        if log_enabled!(Debug) {
            println!("Logging should work.")
        }
        info!(
            "{:#X}  {} {} {}  {} {:28} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}\n",
            self.pc,
            first_byte,
            second_byte,
            third_byte,
            name,
            "",
            self.a,
            self.x,
            self.y,
            p,
            self.sp,
            (self.cycles * 3) % 341
        );
    }

    // Addressing modes

    fn absolute(&mut self) -> u16 {
        self.read16(self.pc + 1)
    }

    fn absolute_x(&mut self, opcode: u8) -> u16 {
        let addr = self.read16(self.pc + 1) + u16::from(self.x);
        if Cpu::check_same_page(addr - u16::from(self.x), addr) {
            self.cycles += CYCLES_PAGE_CROSS[opcode as usize];
        }
        addr
    }

    fn absolute_y(&mut self, opcode: u8) -> u16 {
        let addr = self.read16(self.pc + 1) + u16::from(self.y);
        if Cpu::check_same_page(addr - u16::from(self.y), addr) {
            self.cycles += CYCLES_PAGE_CROSS[opcode as usize];
        }
        addr
    }

    fn immediate(&self) -> u16 {
        self.pc + 1
    }

    fn indexed_indirect(&mut self) -> u16 {
        let addr = u16::from(self.read(self.pc + 1) + self.x);
        self.read16_wrap(addr)
    }

    fn indirect(&mut self) -> u16 {
        let addr = self.read16(self.pc + 1);
        self.read16_wrap(addr)
    }

    fn indirect_indexed(&mut self, opcode: u8) -> u16 {
        let addr = self.read(self.pc + 1);
        let addr = self.read16_wrap(u16::from(addr)) + u16::from(self.y);
        if Cpu::check_same_page(addr - u16::from(self.y), addr) {
            self.cycles += CYCLES_PAGE_CROSS[opcode as usize];
        }
        addr
    }

    fn relative(&mut self) -> u16 {
        let offset = u16::from(self.read(self.pc + 1));
        if offset < 0x80 {
            self.pc + 2 + offset
        } else {
            self.pc + 2 + offset - 0x100
        }
    }

    fn zero_page(&mut self) -> u16 {
        u16::from(self.read(self.pc + 1))
    }

    fn zero_page_x(&mut self) -> u16 {
        u16::from(self.read(self.pc + 1) + self.x) & 0xFF
    }

    fn zero_page_y(&mut self) -> u16 {
        u16::from(self.read(self.pc + 1) + self.y) & 0xFF
    }

    /// Executes the instruciton for the given opcode (with the given address
    /// if applicable).
    fn exec(&mut self, opcode: u8, addr: Option<u16>) {
        // TODO: Could refactor into an array
        match opcode {
            0x69 => self.adc(addr.unwrap()),
            0x65 => self.adc(addr.unwrap()),
            0x75 => self.adc(addr.unwrap()),
            0x6D => self.adc(addr.unwrap()),
            0x7D => self.adc(addr.unwrap()),
            0x79 => self.adc(addr.unwrap()),
            0x61 => self.adc(addr.unwrap()),
            0x71 => self.adc(addr.unwrap()),

            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.sbc(addr.unwrap()),

            0xa1 => self.lda(addr.unwrap()),
            0xa5 => self.lda(addr.unwrap()),
            0xa9 => self.lda(addr.unwrap()),
            0xad => self.lda(addr.unwrap()),
            0xb1 => self.lda(addr.unwrap()),
            0xb5 => self.lda(addr.unwrap()),
            0xb9 => self.lda(addr.unwrap()),
            0xbd => self.lda(addr.unwrap()),

            0xa2 => self.ldx(addr.unwrap()),
            0xa6 => self.ldx(addr.unwrap()),
            0xb6 => self.ldx(addr.unwrap()),
            0xae => self.ldx(addr.unwrap()),
            0xbe => self.ldx(addr.unwrap()),

            0xa0 => self.ldy(addr.unwrap()),
            0xa4 => self.ldy(addr.unwrap()),
            0xb4 => self.ldy(addr.unwrap()),
            0xac => self.ldy(addr.unwrap()),
            0xbc => self.ldy(addr.unwrap()),

            0x85 => self.sta(addr.unwrap()),
            0x95 => self.sta(addr.unwrap()),
            0x8d => self.sta(addr.unwrap()),
            0x9d => self.sta(addr.unwrap()),
            0x99 => self.sta(addr.unwrap()),
            0x81 => self.sta(addr.unwrap()),
            0x91 => self.sta(addr.unwrap()),

            0x86 => self.stx(addr.unwrap()),
            0x96 => self.stx(addr.unwrap()),
            0x8e => self.stx(addr.unwrap()),

            0x84 => self.sty(addr.unwrap()),
            0x94 => self.sty(addr.unwrap()),
            0x8c => self.sty(addr.unwrap()),

            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                let a = self.a;
                self.compare(addr.unwrap(), a);
            }

            0xe0 | 0xe4 | 0xec => {
                let x = self.x;
                self.compare(addr.unwrap(), x);
            }

            0xc0 | 0xC4 | 0xCC => {
                let y = self.y;
                self.compare(addr.unwrap(), y);
            }

            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(addr.unwrap()),

            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ora(addr.unwrap()),

            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.eor(addr.unwrap()),

            0x24 | 0x2C => self.bit(addr.unwrap()),

            // Shifts and rotates
            0x2a => self.rol_a(),
            0x26 => self.rol(addr.unwrap()),
            0x36 => self.rol(addr.unwrap()),
            0x2e => self.rol(addr.unwrap()),
            0x3e => self.rol(addr.unwrap()),

            0x6a => self.ror_a(),
            0x66 => self.ror(addr.unwrap()),
            0x76 => self.ror(addr.unwrap()),
            0x6e => self.ror(addr.unwrap()),
            0x7e => self.ror(addr.unwrap()),

            0x0a => self.asl_a(),
            0x06 => self.asl(addr.unwrap()),
            0x16 => self.asl(addr.unwrap()),
            0x0e => self.asl(addr.unwrap()),
            0x1e => self.asl(addr.unwrap()),

            0x4a => self.lsr_a(),
            0x46 => self.lsr(addr.unwrap()),
            0x56 => self.lsr(addr.unwrap()),
            0x4e => self.lsr(addr.unwrap()),
            0x5e => self.lsr(addr.unwrap()),

            // Increments and decrements
            0xe6 => self.inc(addr.unwrap()),
            0xf6 => self.inc(addr.unwrap()),
            0xee => self.inc(addr.unwrap()),
            0xfe => self.inc(addr.unwrap()),

            0xc6 => self.dec(addr.unwrap()),
            0xd6 => self.dec(addr.unwrap()),
            0xce => self.dec(addr.unwrap()),
            0xde => self.dec(addr.unwrap()),

            0xe8 => self.inx(),
            0xca => self.dex(),
            0xc8 => self.iny(),
            0x88 => self.dey(),

            0xaa => self.tax(),
            0xa8 => self.tay(),
            0x8a => self.txa(),
            0x98 => self.tya(),
            0x9a => self.txs(),
            0xba => self.tsx(),

            0x18 => self.clc(),
            0x38 => self.sec(),
            0x58 => self.cli(),
            0x78 => self.sei(),
            0xb8 => self.clv(),
            0xd8 => self.cld(),
            0xf8 => self.sed(),

            0x10 => self.bpl(addr.unwrap()),
            0x30 => self.bmi(addr.unwrap()),
            0x50 => self.bvc(addr.unwrap()),
            0x70 => self.bvs(addr.unwrap()),
            0x90 => self.bcc(addr.unwrap()),
            0xb0 => self.bcs(addr.unwrap()),
            0xd0 => self.bne(addr.unwrap()),
            0xf0 => self.beq(addr.unwrap()),

            0x4c => self.jmp(addr.unwrap()),
            0x6c => self.jmp(addr.unwrap()),

            0x20 => self.jsr(addr.unwrap()),
            0x60 => self.rts(),
            0x00 => self.brk(),
            0x40 => self.rti(),

            0x48 => self.pha(),
            0x68 => self.pla(),
            0x08 => self.php(),
            0x28 => self.plp(),

            0xea => {}

            // Illegal opcodes

            // NOP
            0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => {}
            // Double NOP
            0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 | 0xc2 | 0xd4
            | 0xe2 | 0xf4 => {}
            // Triple NOP
            0x0c | 0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {}

            // LAX
            0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => self.lax(addr.unwrap()),

            // SAX
            0x87 | 0x97 | 0x83 | 0x8F => self.sax(addr.unwrap()),

            // SBC
            0xEB => self.sbc(addr.unwrap()),

            // DCP
            0xC7 | 0xD7 | 0xCF | 0xDF | 0xDB | 0xC3 | 0xD3 => self.dcp(addr.unwrap()),

            // ISC
            0xE7 | 0xF7 | 0xEF | 0xFF | 0xFB | 0xE3 | 0xF3 => self.isc(addr.unwrap()),

            // SLO
            0x07 | 0x17 | 0x0F | 0x1F | 0x1B | 0x03 | 0x13 => self.slo(addr.unwrap()),

            // RLA
            0x27 | 0x37 | 0x2F | 0x3F | 0x3B | 0x23 | 0x33 => self.rla(addr.unwrap()),

            // SRE
            0x47 | 0x57 | 0x4F | 0x5F | 0x5B | 0x43 | 0x53 => self.sre(addr.unwrap()),

            // RRA
            0x67 | 0x77 | 0x6F | 0x7F | 0x7B | 0x63 | 0x73 => self.rra(addr.unwrap()),

            _ => unimplemented!(),
        }
    }

    fn nmi(&mut self) {
        let pc = self.pc;
        self.push16(pc);
        self.php();
        self.pc = self.read16(NMI_VECTOR);
        self.p.set_i(true);
        self.cycles += 7;
    }

    fn irq(&mut self) {
        let pc = self.pc;
        self.push16(pc);
        self.php();
        self.pc = self.read16(IRQ_BRK_VECTOR);
        self.p.set_i(true);
        self.cycles += 7;
    }

    // Operations

    /// ADC - Add with Carry
    fn adc(&mut self, addr: u16) {
        let a = self.a;
        let m = self.read(addr);
        let c = self.p.get_c() as u8;

        let result = a + m + c;

        self.a = result;
        self.check_negative_zero(result);

        // Check if result overflows bit 7
        self.p.set_c(a as u16 + m as u16 + c as u16 > 0xFF);

        // Check if sign is incorrect
        self.p
            .set_v((a ^ m) & 0x80 == 0 && (a ^ self.a) & 0x80 != 0);
    }

    /// AND - Logical AND
    fn and(&mut self, addr: u16) {
        self.a &= self.read(addr);
        let a = self.a;
        self.check_negative_zero(a);
    }

    // ASL - Arithmetic Shift Left (accumulator)
    fn asl_a(&mut self) {
        self.p.set_c((self.a >> 7) & 1 == 1);
        self.a <<= 1;
        let a = self.a;
        self.check_negative_zero(a);
    }

    // ASL - Arithmetic Shift Left
    fn asl(&mut self, addr: u16) {
        let m = self.read(addr);
        self.p.set_c((m >> 7) & 1 == 1);
        self.write(addr, m << 1);
        self.check_negative_zero(m << 1);
    }

    // TODO Refactor branch ops

    /// BCC - Branch if Carry Clear
    fn bcc(&mut self, addr: u16) {
        if !self.p.get_c() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BCS - Branch if Carry Set
    fn bcs(&mut self, addr: u16) {
        if self.p.get_c() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BEQ - Branch if Equal
    fn beq(&mut self, addr: u16) {
        if self.p.get_z() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BIT - Bit Test
    fn bit(&mut self, addr: u16) {
        let m = self.read(addr);
        self.p.set_v((m >> 6) & 1 == 1);
        let a = self.a;
        self.check_zero(m & a);
        self.check_negative(m);
    }

    /// BMI - Branch if Minus
    fn bmi(&mut self, addr: u16) {
        if self.p.get_n() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BNE - Branch if Not Equal
    fn bne(&mut self, addr: u16) {
        if !self.p.get_z() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BPL - Branch if Positive
    fn bpl(&mut self, addr: u16) {
        if !self.p.get_n() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BRK - Force Interrupt
    fn brk(&mut self) {
        let pc = self.pc;
        self.pc = self.read16(IRQ_BRK_VECTOR);
        self.push16(pc);
        self.php();
        self.sei();
    }

    /// BVC - Branch if Overflow Clear
    fn bvc(&mut self, addr: u16) {
        if !self.p.get_v() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// BVS - Branch if Overflow Set
    fn bvs(&mut self, addr: u16) {
        if self.p.get_v() {
            self.cycles += 1;
            if Cpu::check_same_page(self.pc, addr) {
                self.cycles += 1;
            }
            self.pc = addr;
        }
    }

    /// CLC - Clear Carry Flag
    fn clc(&mut self) {
        self.p.set_c(false);
    }

    /// CLD - Clear Decimal Mode
    fn cld(&mut self) {
        self.p.set_d(false);
    }

    /// CLI - Clear Interrupt Disable
    fn cli(&mut self) {
        self.p.set_i(false);
    }

    /// CLV - Clear Overflow Flag
    fn clv(&mut self) {
        self.p.set_v(false);
    }

    /// Comparison, used for:
    /// * CMP - Compare
    /// * CPX - Compare X Register
    /// * CPY - Compare Y register
    fn compare(&mut self, addr: u16, register_val: u8) {
        let m = self.read(addr);
        self.check_negative_zero(register_val - m);
        self.p.set_c(register_val >= m);
    }

    /// DEC - Decrement Memory
    fn dec(&mut self, addr: u16) {
        let m = self.read(addr);
        self.write(addr, m - 1);
        self.check_negative_zero(m - 1);
    }

    /// DEX - Decrement X Register
    fn dex(&mut self) {
        self.x -= 1;
        let x = self.x;
        self.check_negative_zero(x);
    }

    /// DEY - Decrement Y Register
    fn dey(&mut self) {
        self.y -= 1;
        let y = self.y;
        self.check_negative_zero(y);
    }

    /// EOR - Exclusive OR
    fn eor(&mut self, addr: u16) {
        self.a ^= self.read(addr);
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// INC - Increment Memory
    fn inc(&mut self, addr: u16) {
        let m = self.read(addr);
        self.write(addr, m + 1);
        self.check_negative_zero(m + 1);
    }

    /// INX - Increment X Register
    fn inx(&mut self) {
        self.x += 1;
        let x = self.x;
        self.check_negative_zero(x);
    }

    /// INY - Increment Y Register
    fn iny(&mut self) {
        self.y += 1;
        let y = self.y;
        self.check_negative_zero(y);
    }

    /// JMP - Jump
    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// JSR - Jump to Subroutine
    fn jsr(&mut self, addr: u16) {
        let pc = self.pc;
        self.push16(pc - 1);
        self.pc = addr;
    }

    /// LDA - Load Accumulator
    fn lda(&mut self, addr: u16) {
        let a = self.read(addr);
        self.a = a;
        self.check_negative_zero(a);
    }

    /// LDX - Load X Register
    fn ldx(&mut self, addr: u16) {
        let m = self.read(addr);
        self.x = m;
        self.check_negative_zero(m);
    }

    /// LDY - Load Y Register
    fn ldy(&mut self, addr: u16) {
        let m = self.read(addr);
        self.y = m;
        self.check_negative_zero(m);
    }

    /// LSR - Logical Shift Right (Accumulator)
    fn lsr_a(&mut self) {
        self.p.set_c(self.a & 1 == 1);
        self.a >>= 1;
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// LSR - Logical Shift Right
    fn lsr(&mut self, addr: u16) {
        let m = self.read(addr);
        self.p.set_c(m & 1 == 1);
        self.write(addr, m >> 1);
        self.check_negative_zero(m >> 1);
    }

    /// ORA - Logical Inclusive OR
    fn ora(&mut self, addr: u16) {
        self.a |= self.read(addr);
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// PHA - Push Accumulator
    fn pha(&mut self) {
        let a = self.a;
        self.push(a);
    }

    /// PHP - Push Processor Status
    fn php(&mut self) {
        let p: u8 = self.p.bit_range(7, 0);
        self.push(p | 0x10);
    }

    /// PLA - Pull Accumulator
    fn pla(&mut self) {
        self.a = self.pull();
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// PLP - Pull Processor Status
    fn plp(&mut self) {
        let p = self.pull() & 0xEF | 0x20;
        self.p.set_bit_range(7, 0, p);
    }

    /// ROL - Rotate Left (Accumulator)
    fn rol_a(&mut self) {
        let c = self.p.get_c() as u8;
        self.p.set_c(self.a >> 7 & 1 == 1);
        self.a = (self.a << 1) | c;
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// ROL - Rotate Left
    fn rol(&mut self, addr: u16) {
        let c = self.p.get_c() as u8;
        let m = self.read(addr);
        self.p.set_c((m >> 7) & 1 == 1);
        self.write(addr, (m << 1) | c);
        self.check_negative_zero((m << 1) | c);
    }

    /// ROR - Rotate Right (Accumulator)
    fn ror_a(&mut self) {
        let c = self.p.get_c() as u8;
        self.p.set_c(self.a & 1 == 1);
        self.a = (self.a >> 1) | (c << 7);
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// ROR - Rotate Right
    fn ror(&mut self, addr: u16) {
        let c = self.p.get_c() as u8;
        let m = self.read(addr);
        self.p.set_c(m & 1 == 1);
        self.write(addr, (m >> 1) | (c << 7));
        self.check_negative_zero((m >> 1) | (c << 7));
    }

    /// RTI - Return from Interrupt
    fn rti(&mut self) {
        let p = self.pull();
        self.p.set_bit_range(7, 0, p & 0xEF | 0x20);
        self.pc = self.pull16();
    }

    /// RTS - Return from Subroutine
    fn rts(&mut self) {
        self.pc = self.pull16() + 1;
    }

    /// SBC - Subtract with Carry
    fn sbc(&mut self, addr: u16) {
        let a = self.a;
        let m = self.read(addr);
        let c = self.p.get_c() as u8;
        let res = a - m - (1 - c);
        self.a = res;
        self.check_negative_zero(res);

        self.p
            .set_c(a as isize - m as isize - (1 - c) as isize >= 0);

        self.p
            .set_v((a ^ m) & 0x80 != 0 && (a ^ self.a) & 0x80 != 0);
    }

    /// SEC - Set Carry Flag
    fn sec(&mut self) {
        self.p.set_c(true);
    }

    /// SED - Set Decimal Flag
    fn sed(&mut self) {
        self.p.set_d(true);
    }

    /// SEI - Set Interrupt Disable
    fn sei(&mut self) {
        self.p.set_i(true);
    }

    // STA - Store Accumulator
    fn sta(&mut self, addr: u16) {
        let a = self.a;
        self.write(addr, a);
    }

    /// STX - Store X Register
    fn stx(&mut self, addr: u16) {
        let x = self.x;
        self.write(addr, x);
    }

    /// STY - Story Y Register
    fn sty(&mut self, addr: u16) {
        let y = self.y;
        self.write(addr, y);
    }

    /// TAX - Transfer Accumulator to X
    fn tax(&mut self) {
        self.x = self.a;
        let x = self.x;
        self.check_negative_zero(x);
    }

    /// TAY - Transfer Accumulator to Y
    fn tay(&mut self) {
        self.y = self.a;
        let y = self.y;
        self.check_negative_zero(y);
    }

    /// TSX - Transfer Stack Pointer to X
    fn tsx(&mut self) {
        self.x = self.sp;
        let x = self.x;
        self.check_negative_zero(x);
    }

    /// TXA - Transfer X to Accumulator
    fn txa(&mut self) {
        self.a = self.x;
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// TXS - Transfer X to Stack Pointer
    fn txs(&mut self) {
        self.sp = self.x;
    }

    /// TYA - Transfer Y to Accumulator
    fn tya(&mut self) {
        self.a = self.y;
        let a = self.a;
        self.check_negative_zero(a);
    }

    // Illegal ops

    /// LAX - Load Accumulator and X Register with Memory
    fn lax(&mut self, addr: u16) {
        let m = self.read(addr);
        self.a = m;
        self.x = m;
        self.check_negative_zero(m);
    }

    /// SAX - Store Accumulator AND X Register
    fn sax(&mut self, addr: u16) {
        let val = self.a & self.x;
        self.write(addr, val);
        // self.check_negative_zero(val);
    }

    /// DCP - Decrement and Compare (with accumulator)
    fn dcp(&mut self, addr: u16) {
        self.dec(addr);
        self.compare(addr, self.a);
    }

    /// ISC - Increment And Subtract (from accumulator) with Carry
    fn isc(&mut self, addr: u16) {
        self.inc(addr);
        self.sbc(addr);
    }

    /// SLO - Shift Left and OR (with accumulator)
    fn slo(&mut self, addr: u16) {
        let m = self.read(addr);
        self.p.set_c((m >> 7) & 1 == 1);
        let val = m << 1;
        self.a |= val;
        self.check_negative_zero(self.a);
        self.write(addr, val);
    }

    /// RLA - Rotate Left then AND (with accumulator)
    fn rla(&mut self, addr: u16) {
        let c = self.p.get_c() as u8;
        let m = self.read(addr);
        self.p.set_c((m >> 7) & 1 == 1);
        let val = (m << 1) | c;
        self.a &= val;
        self.check_negative_zero(self.a);
        self.write(addr, val);
    }

    /// SRE - Shift Right then EOR (XOR) (with accumulator)
    fn sre(&mut self, addr: u16) {
        let m = self.read(addr);
        self.p.set_c(m & 1 == 1);
        let val = m >> 1;
        self.a ^= val;
        self.check_negative_zero(self.a);
        self.write(addr, val);
    }

    /// RRA - Rotate Right then Add (to accumulator) with carry
    fn rra(&mut self, addr: u16) {
        // let c = self.p.get_c() as u8;
        // let m = self.read(addr);
        // self.p.set_c(m & 1 == 1);
        // let val = (m >> 1) | c;
        self.ror(addr);
        self.adc(addr);
        // self.write(addr, val);
    }
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use std::rc::Rc;

    use super::{Controller, Cpu, Mapper, Ppu};
    use crate::mapper;
    use crate::rom::Rom;

    #[test]
    fn golden_log() {
        let path = Path::new("test_roms/nestest.nes");
        let rom = Rom::load(&mut File::open(&path).unwrap()).unwrap();
        let mapper = mapper::init(rom);
        let mut mapper = Rc::new(RefCell::new(mapper));
        let controller = Controller::default();
        let ppu = Rc::new(RefCell::new(Ppu::new(mapper.clone())));
        let mut cpu = Cpu::new(mapper.clone(), controller, ppu);

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
