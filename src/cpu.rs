use bitfield::BitRange;
use mapper::Mapper;

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
    mapper: Mapper,
}

impl Cpu {
    pub fn new(mapper: Mapper) -> Cpu {
        Cpu {
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
            mapper: mapper,
        }
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
            self.ram[(addr % 0x0800) as usize]
        } else if addr >= 0x6000 {
            self.mapper.read(addr)
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
        } else if addr >= 0x6000 {
            self.mapper.write(addr, val);
        }
    }

    fn resolve_address(&mut self, mode_no: usize) -> Option<u16> {
        match mode_no {
            1 => Some(self.absolute()),
            2 => Some(self.absolute_x()),
            3 => Some(self.absolute_y()),
            4 => None,
            5 => Some(self.immediate()),
            6 => None,
            7 => Some(self.indexed_indirect()),
            8 => Some(self.indirect()),
            9 => Some(self.indirect_indexed()),
            10 => Some(self.relative()),
            11 => Some(self.zero_page()),
            12 => Some(self.zero_page_x()),
            13 => Some(self.zero_page_y()),
            _ => None,
        }
    }

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

        // FIXME problme is here, order is wrong, can't get address with wrong pc

        let opcode = self.read(self.pc);
        let cycles = INSTRUCTION_CYCLES[opcode as usize];
        let mode_no = INSTRUCTION_MODES[opcode as usize];

        let addressing_mode = self.resolve_address(mode_no);

        self.pc += INSTRUCTION_SIZES[opcode as usize] as u16;
        self.cycles += cycles;

        self.exec(opcode, addressing_mode);

        return (self.cycles - cy) as isize;
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
        trace!(
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

    fn exec(&mut self, opcode: u8, addr: Option<u16>) {
        match opcode {
            0x69 => self.adc(addr.unwrap()),
            0x65 => self.adc(addr.unwrap()),
            0x75 => self.adc(addr.unwrap()),
            0x6D => self.adc(addr.unwrap()),
            0x7D => self.adc(addr.unwrap()),
            0x79 => self.adc(addr.unwrap()),
            0x61 => self.adc(addr.unwrap()),
            0x71 => self.adc(addr.unwrap()),

            0xE9 => self.sbc(addr.unwrap()),
            0xE5 => self.sbc(addr.unwrap()),
            0xF5 => self.sbc(addr.unwrap()),
            0xED => self.sbc(addr.unwrap()),
            0xFD => self.sbc(addr.unwrap()),
            0xF9 => self.sbc(addr.unwrap()),
            0xE1 => self.sbc(addr.unwrap()),
            0xF1 => self.sbc(addr.unwrap()),

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

            // Comparisons
            0xc9 => {
                let a = self.a;
                self.compare(addr.unwrap(), a);
            }
            0xc5 => {
                let a = self.a;
                self.compare(addr.unwrap(), a);
            }
            0xd5 => {
                let a = self.a;
                self.compare(addr.unwrap(), a);
            }
            0xcd => {
                let a = self.a;
                self.compare(addr.unwrap(), a);
            }
            0xdd => {
                let a = self.a;

                self.compare(addr.unwrap(), a);
            }
            0xd9 => {
                let a = self.a;

                self.compare(addr.unwrap(), a);
            }
            0xc1 => {
                let a = self.a;

                self.compare(addr.unwrap(), a);
            }
            0xd1 => {
                let a = self.a;

                self.compare(addr.unwrap(), a);
            }

            0xe0 => {
                let x = self.x;

                self.compare(addr.unwrap(), x);
            }
            0xe4 => {
                let x = self.x;

                self.compare(addr.unwrap(), x);
            }
            0xec => {
                let x = self.x;

                self.compare(addr.unwrap(), x);
            }

            0xc0 => {
                let y = self.y;

                self.compare(addr.unwrap(), y);
            }
            0xc4 => {
                let y = self.y;

                self.compare(addr.unwrap(), y);
            }
            0xcc => {
                let y = self.y;

                self.compare(addr.unwrap(), y);
            }

            // Bitwise operations
            0x29 => self.and(addr.unwrap()),
            0x25 => self.and(addr.unwrap()),
            0x35 => self.and(addr.unwrap()),
            0x2d => self.and(addr.unwrap()),
            0x3d => self.and(addr.unwrap()),
            0x39 => self.and(addr.unwrap()),
            0x21 => self.and(addr.unwrap()),
            0x31 => self.and(addr.unwrap()),

            0x09 => self.ora(addr.unwrap()),
            0x05 => self.ora(addr.unwrap()),
            0x15 => self.ora(addr.unwrap()),
            0x0d => self.ora(addr.unwrap()),
            0x1d => self.ora(addr.unwrap()),
            0x19 => self.ora(addr.unwrap()),
            0x01 => self.ora(addr.unwrap()),
            0x11 => self.ora(addr.unwrap()),

            0x49 => self.eor(addr.unwrap()),
            0x45 => self.eor(addr.unwrap()),
            0x55 => self.eor(addr.unwrap()),
            0x4d => self.eor(addr.unwrap()),
            0x5d => self.eor(addr.unwrap()),
            0x59 => self.eor(addr.unwrap()),
            0x41 => self.eor(addr.unwrap()),
            0x51 => self.eor(addr.unwrap()),

            0x24 => self.bit(addr.unwrap()),
            0x2c => self.bit(addr.unwrap()),

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

    // TYA - Transfer Y to Accumulator
    fn tya(&mut self) {
        self.a = self.y;
        let a = self.a;
        self.check_negative_zero(a);
    }
}
