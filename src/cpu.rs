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

    pub fn step(&mut self) {
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
        let op = self.decode(opcode);
        op();
        self.pc += 1;
        self.cycles += cycles;
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

    fn resolve_addressing(mode: AddressingMode) -> u16 {
        // match mode {}
        return 0;
    }

    fn decode(&self, opcode: u8) -> Box<Fn()> {
        Box::new(move || println!("No!"))
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

    fn adc(&self, addr: u16) {
        let a = self.a;
        let m = self.read(addr);
    }

    // PHP - Push Processor Status
    fn php(&mut self) {
        let p: u8 = self.p.bit_range(7, 0);
        self.push(p | 0x10);
    }
}
