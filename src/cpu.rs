use bitfield::{Bit, BitRange};

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

enum AddressingMode {
    Absolute = 1,
    AbsoluteX = 2,
    AbsoluteY = 3,
    Accumulator = 4,
    Immediate = 5,
    Implied = 6,
    IndexedIndirect = 7,
    Indirect = 8,
    IndirectIndexed = 9,
    Relative = 10,
    ZeroPage = 11,
    ZeroPageX = 12,
    ZeroPageY = 13,
}

/// Instruction mode corresponding to the variants of the AddressingMode enum
const INSTRUCTION_MODES: [usize; 256] = [
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
const INSTRUCTION_SIZES: [usize; 256] = [
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
    2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
];

const INSTRUCTION_CYCLES: [usize; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

const INSTRUCTION_NAMES: [&str; 256] = [
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
    Reset,
    None,
}

bitfield!{
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
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
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
        };
        cpu.reset();
        cpu
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
        self.write(STACK | (sp as u16), val);
        self.sp -= 1;
    }

    fn pull(&mut self) -> u8 {
        self.sp += 1;
        self.read(STACK | (self.sp as u16))
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

    /// Reads a byte from memory
    fn read(&self, addr: u16) -> u8 {
        // TODO match
        if addr < 0x2000 {
            self.ram[(addr & 0x7FF) as usize]
        } else {
            0
        }
    }

    fn read16(&self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr + 1) as u16;
        (high << 8) | low
    }

    fn read16_wrap(&self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr & 0xFF00) as u16;
        (high << 8) | low
    }

    /// Writes a byte to memory
    fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.ram[(addr & 0x7FF) as usize] = val;
        }
    }

    pub fn step(&mut self) -> isize {
        if self.stall > 0 {
            self.stall -= 1;
        }

        self.trace();

        match self.interrupt {
            Interrupt::IRQ => self.irq(),
            Interrupt::NMI => self.nmi(),
            Interrupt::Reset => self.reset(),
            Interrupt::None => {}
        }

        let opcode = self.read(self.pc);
        let cycles = INSTRUCTION_CYCLES[opcode as usize];
        self.exec(opcode);
        self.pc += INSTRUCTION_SIZES[opcode as usize] as u16;
        self.cycles += cycles;

        return (self.cycles - cycles) as isize;
    }

    fn trace(&self) {
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
        println!(
            "{:4X}  {} {} {}  {} {:28} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}\n",
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

    fn absolute(&self) -> u16 {
        self.read16(self.pc + 1)
    }

    fn absolute_x(&mut self) -> u16 {
        let addr = self.read16(self.pc + 1) + self.x as u16;
        if Cpu::check_same_page(addr - self.x as u16, addr) {
            self.cycles += 1;
        }
        return addr;
    }

    fn absolute_y(&mut self) -> u16 {
        let addr = self.read16(self.pc + 1) + self.y as u16;
        if Cpu::check_same_page(addr - self.y as u16, addr) {
            self.cycles += 1;
        }
        return addr;
    }

    fn immediate(&self) -> u16 {
        self.pc + 1
    }

    fn indexed_indirect(&self) -> u16 {
        let addr = self.read(self.pc + 1) as u16 + self.x as u16;
        self.read16_wrap(addr)
    }

    fn indirect(&self) -> u16 {
        self.read16_wrap(self.read16(self.pc + 1))
    }

    fn indirect_indexed(&self) -> u16 {
        let addr = self.read(self.pc + 1) as u16 + self.y as u16;
        self.read16_wrap(addr)
    }

    fn relative(&self) -> u16 {
        let offset = self.read(self.pc + 1) as u16;
        if offset < 0x80 {
            return self.pc + 2 + offset;
        } else {
            return self.pc + 2 + offset - 0x100;
        }
    }

    fn zero_page(&self) -> u16 {
        self.read(self.pc + 1) as u16
    }

    fn zero_page_x(&self) -> u16 {
        (self.read(self.pc + 1) + self.x) as u16 & 0xFF
    }

    fn zero_page_y(&self) -> u16 {
        (self.read(self.pc + 1) + self.y) as u16 & 0xFF
    }

    fn exec(&mut self, opcode: u8) {
        match opcode {
            // ADC
            0x69 => {
                let addr = self.immediate();
                self.adc(addr);
            }
            0x65 => {
                let addr = self.zero_page();
                self.adc(addr);
            }
            0x75 => {
                let addr = self.zero_page_x();
                self.adc(addr)
            }
            0x6D => {
                let addr = self.absolute();
                self.adc(addr)
            }
            0x7D => {
                let addr = self.absolute_x();
                self.adc(addr)
            }
            0x79 => {
                let addr = self.absolute_y();
                self.adc(addr)
            }
            0x61 => {
                let addr = self.indexed_indirect();
                self.adc(addr)
            }
            0x71 => {
                let addr = self.indirect_indexed();
                self.adc(addr)
            }
            // SBC
            0xE9 => {
                let addr = self.immediate();
                self.sbc(addr);
            }
            0xE5 => {
                let addr = self.zero_page();
                self.sbc(addr);
            }
            0xF5 => {
                let addr = self.zero_page_x();
                self.sbc(addr)
            }
            0xED => {
                let addr = self.absolute();
                self.sbc(addr)
            }
            0xFD => {
                let addr = self.absolute_x();
                self.sbc(addr)
            }
            0xF9 => {
                let addr = self.absolute_y();
                self.sbc(addr)
            }
            0xE1 => {
                let addr = self.indexed_indirect();
                self.sbc(addr)
            }
            0xF1 => {
                let addr = self.indirect_indexed();
                self.sbc(addr)
            }
            // LDA
            0xa1 => {
                let addr = self.indexed_indirect();
                self.lda(addr)
            }
            0xa5 => {
                let addr = self.zero_page();
                self.lda(addr)
            }
            0xa9 => {
                let addr = self.immediate();
                self.lda(addr)
            }
            0xad => {
                let addr = self.absolute();
                self.lda(addr)
            }
            0xb1 => {
                let addr = self.indirect_indexed();
                self.lda(addr)
            }
            0xb5 => {
                let addr = self.zero_page_x();
                self.lda(addr)
            }
            0xb9 => {
                let addr = self.absolute_y();
                self.lda(addr)
            }
            0xbd => {
                let addr = self.absolute_x();
                self.lda(addr)
            }
            // LDX
            0xa2 => {
                let addr = self.immediate();
                self.ldx(addr)
            }
            0xa6 => {
                let addr = self.zero_page();
                self.ldx(addr)
            }
            0xb6 => {
                let addr = self.zero_page_y();
                self.ldx(addr)
            }
            0xae => {
                let addr = self.absolute();
                self.ldx(addr)
            }
            0xbe => {
                let addr = self.absolute_y();
                self.ldx(addr)
            }
            // LDY
            0xa0 => {
                let addr = self.immediate();
                self.ldy(addr)
            }
            0xa4 => {
                let addr = self.zero_page();
                self.ldy(addr)
            }
            0xb4 => {
                let addr = self.zero_page_x();
                self.ldy(addr)
            }
            0xac => {
                let addr = self.absolute();
                self.ldy(addr)
            }
            0xbc => {
                let addr = self.absolute_x();
                self.ldy(addr)
            }

            // Stores
            0x85 => {
                let addr = self.zero_page();
                self.sta(addr)
            }
            0x95 => {
                let addr = self.zero_page_x();
                self.sta(addr)
            }
            0x8d => {
                let addr = self.absolute();
                self.sta(addr)
            }
            0x9d => {
                let addr = self.absolute_x();
                self.sta(addr)
            }
            0x99 => {
                let addr = self.absolute_y();
                self.sta(addr)
            }
            0x81 => {
                let addr = self.indexed_indirect();
                self.sta(addr)
            }
            0x91 => {
                let addr = self.indirect_indexed();
                self.sta(addr)
            }

            0x86 => {
                let addr = self.zero_page();
                self.stx(addr)
            }
            0x96 => {
                let addr = self.zero_page_y();
                self.stx(addr)
            }
            0x8e => {
                let addr = self.absolute();
                self.stx(addr)
            }

            0x84 => {
                let addr = self.zero_page();
                self.sty(addr)
            }
            0x94 => {
                let addr = self.zero_page_x();
                self.sty(addr)
            }
            0x8c => {
                let addr = self.absolute();
                self.sty(addr)
            }

            // Comparisons
            0xc9 => {
                let a = self.a;
                let addr = self.immediate();
                self.compare(addr, a);
            }
            0xc5 => {
                let a = self.a;
                let addr = self.zero_page();
                self.compare(addr, a);
            }
            0xd5 => {
                let a = self.a;
                let addr = self.zero_page_x();
                self.compare(addr, a);
            }
            0xcd => {
                let a = self.a;
                let addr = self.absolute();
                self.compare(addr, a);
            }
            0xdd => {
                let a = self.a;
                let addr = self.absolute_x();
                self.compare(addr, a);
            }
            0xd9 => {
                let a = self.a;
                let addr = self.absolute_y();
                self.compare(addr, a);
            }
            0xc1 => {
                let a = self.a;
                let addr = self.indexed_indirect();
                self.compare(addr, a);
            }
            0xd1 => {
                let a = self.a;
                let addr = self.indirect_indexed();
                self.compare(addr, a);
            }

            0xe0 => {
                let x = self.x;
                let addr = self.immediate();
                self.compare(addr, x);
            }
            0xe4 => {
                let x = self.x;
                let addr = self.zero_page();
                self.compare(addr, x);
            }
            0xec => {
                let x = self.x;
                let addr = self.absolute();
                self.compare(addr, x);
            }

            0xc0 => {
                let y = self.y;
                let addr = self.immediate();
                self.compare(addr, y);
            }
            0xc4 => {
                let y = self.y;
                let addr = self.zero_page();
                self.compare(addr, y);
            }
            0xcc => {
                let y = self.y;
                let addr = self.absolute();
                self.compare(addr, y);
            }

            // Bitwise operations
            0x29 => {
                let addr = self.immediate();
                self.and(addr)
            }
            0x25 => {
                let addr = self.zero_page();
                self.and(addr)
            }
            0x35 => {
                let addr = self.zero_page_x();
                self.and(addr)
            }
            0x2d => {
                let addr = self.absolute();
                self.and(addr)
            }
            0x3d => {
                let addr = self.absolute_x();
                self.and(addr)
            }
            0x39 => {
                let addr = self.absolute_y();
                self.and(addr)
            }
            0x21 => {
                let addr = self.indexed_indirect();
                self.and(addr)
            }
            0x31 => {
                let addr = self.indirect_indexed();
                self.and(addr)
            }

            0x09 => {
                let addr = self.immediate();
                self.ora(addr)
            }
            0x05 => {
                let addr = self.zero_page();
                self.ora(addr)
            }
            0x15 => {
                let addr = self.zero_page_x();
                self.ora(addr)
            }
            0x0d => {
                let addr = self.absolute();
                self.ora(addr)
            }
            0x1d => {
                let addr = self.absolute_x();
                self.ora(addr)
            }
            0x19 => {
                let addr = self.absolute_y();
                self.ora(addr)
            }
            0x01 => {
                let addr = self.indexed_indirect();
                self.ora(addr)
            }
            0x11 => {
                let addr = self.indirect_indexed();
                self.ora(addr)
            }

            0x49 => {
                let addr = self.immediate();
                self.eor(addr)
            }
            0x45 => {
                let addr = self.zero_page();
                self.eor(addr)
            }
            0x55 => {
                let addr = self.zero_page_x();
                self.eor(addr)
            }
            0x4d => {
                let addr = self.absolute();
                self.eor(addr)
            }
            0x5d => {
                let addr = self.absolute_x();
                self.eor(addr)
            }
            0x59 => {
                let addr = self.absolute_y();
                self.eor(addr)
            }
            0x41 => {
                let addr = self.indexed_indirect();
                self.eor(addr)
            }
            0x51 => {
                let addr = self.indirect_indexed();
                self.eor(addr)
            }

            0x24 => {
                let addr = self.zero_page();
                self.bit(addr)
            }
            0x2c => {
                let addr = self.absolute();
                self.bit(addr)
            }

            // Shifts and rotates
            0x2a => self.rol_a(),
            0x26 => {
                let addr = self.zero_page();
                self.rol(addr)
            }
            0x36 => {
                let addr = self.zero_page_x();
                self.rol(addr)
            }
            0x2e => {
                let addr = self.absolute();
                self.rol(addr)
            }
            0x3e => {
                let addr = self.absolute_x();
                self.rol(addr)
            }

            0x6a => self.ror_a(),
            0x66 => {
                let addr = self.zero_page();
                self.ror(addr)
            }
            0x76 => {
                let addr = self.zero_page_x();
                self.ror(addr)
            }
            0x6e => {
                let addr = self.absolute();
                self.ror(addr)
            }
            0x7e => {
                let addr = self.absolute_x();
                self.ror(addr)
            }

            0x0a => self.asl_a(),
            0x06 => {
                let addr = self.zero_page();
                self.asl(addr)
            }
            0x16 => {
                let addr = self.zero_page_x();
                self.asl(addr)
            }
            0x0e => {
                let addr = self.absolute();
                self.asl(addr)
            }
            0x1e => {
                let addr = self.absolute_x();
                self.asl(addr)
            }

            0x4a => self.lsr_a(),
            0x46 => {
                let addr = self.zero_page();
                self.lsr(addr)
            }
            0x56 => {
                let addr = self.zero_page_x();
                self.lsr(addr)
            }
            0x4e => {
                let addr = self.absolute();
                self.lsr(addr)
            }
            0x5e => {
                let addr = self.absolute_x();
                self.lsr(addr)
            }

            // Increments and decrements
            0xe6 => {
                let addr = self.zero_page();
                self.inc(addr)
            }
            0xf6 => {
                let addr = self.zero_page_x();
                self.inc(addr)
            }
            0xee => {
                let addr = self.absolute();
                self.inc(addr)
            }
            0xfe => {
                let addr = self.absolute_x();
                self.inc(addr)
            }

            0xc6 => {
                let addr = self.zero_page();
                self.dec(addr)
            }
            0xd6 => {
                let addr = self.zero_page_x();
                self.dec(addr)
            }
            0xce => {
                let addr = self.absolute();
                self.dec(addr)
            }
            0xde => {
                let addr = self.absolute_x();
                self.dec(addr)
            }

            0xe8 => self.inx(),
            0xca => self.dex(),
            0xc8 => self.iny(),
            0x88 => self.dey(),

            // Register moves
            0xaa => self.tax(),
            0xa8 => self.tay(),
            0x8a => self.txa(),
            0x98 => self.tya(),
            0x9a => self.txs(),
            0xba => self.tsx(),

            // Flag operations
            0x18 => self.clc(),
            0x38 => self.sec(),
            0x58 => self.cli(),
            0x78 => self.sei(),
            0xb8 => self.clv(),
            0xd8 => self.cld(),
            0xf8 => self.sed(),

            // Branches
            0x10 => {
                let addr = self.relative();
                self.bpl(addr);
            }
            0x30 => {
                let addr = self.relative();
                self.bmi(addr);
            }
            0x50 => {
                let addr = self.relative();
                self.bvc(addr);
            }
            0x70 => {
                let addr = self.relative();
                self.bvs(addr);
            }
            0x90 => {
                let addr = self.relative();
                self.bcc(addr);
            }
            0xb0 => {
                let addr = self.relative();
                self.bcs(addr);
            }
            0xd0 => {
                let addr = self.relative();
                self.bne(addr);
            }
            0xf0 => {
                let addr = self.relative();
                self.beq(addr);
            }

            // Jumps
            0x4c => {
                let addr = self.absolute();
                self.jmp(addr);
            }
            0x6c => {
                let addr = self.indirect();
                self.jmp(addr);
            }

            // Procedure calls
            0x20 => {
                let addr = self.absolute();
                self.jsr(addr);
            }
            0x60 => self.rts(),
            0x00 => self.brk(),
            0x40 => self.rti(),

            // Stack operations
            0x48 => self.pha(),
            0x68 => self.pla(),
            0x08 => self.php(),
            0x28 => self.plp(),

            // No operation
            0xea => {}

            _ => {}
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
        self.a = self.read(addr);
        let a = self.a;
        self.check_negative_zero(a);
    }

    /// LDX - Load X Register
    fn ldx(&mut self, addr: u16) {
        self.x = self.read(addr);
        let x = self.x;
        self.check_negative_zero(x);
    }

    /// LDY - Load Y Register
    fn ldy(&mut self, addr: u16) {
        self.y = self.read(addr);
        let y = self.y;
        self.check_negative_zero(y);
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

        self.p.set_c(a as isize - m as isize - (1 - c) as isize >= 0);

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

    // TYA - Transfer Y to Accumulator
    fn tya(&mut self) {
        self.a = self.y;
        let a = self.a;
        self.check_negative_zero(a);
    }
}
