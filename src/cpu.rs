
/// Stack offset 
const STACK: u16 = 0x100;
/// NMI vector
const NMI_VECTOR: u16 = 0xFFFA;
/// Reset vector
const RESET_VECTOR: u16= 0xFFFC;
/// IRQ/BRK vector
const IRQ_BRK_VECTOR: u16 = 0xFFFE;

/// The CPU struct
pub struct Cpu {
    ram: [u8; 0x800],
    reset: bool,
    nmi: bool,
    nmi_edge_detected: bool,
    interrupt: bool,
    // Registers
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    status: u8, // The status register is made up of 5 flags and 3 unused bits
}

enum AddressingMode {
    Absolute(u16),
}



impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            ram: [0; 0x800],
            reset: true,
            nmi: false,
            nmi_edge_detected: false,
            interrupt: false,
            pc: 0xC000,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            status: 0x34
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x34;
        self.sp = 0xFD;
    }

    // Stack

    fn push(&mut self, val: u8) {
        self.write(STACK | (self.sp as u16), val);
        self.sp -= 1;

    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.read(STACK | (self.sp as u16))

    }

    /// Reads a byte from memory
    fn read(&self, addr: u16) -> u8 {
        self.memory_access(addr, None)
    }

    /// Writes a byte to memory
    fn write(&self, addr: u16, val: u8) -> u8 {
        self.memory_access(addr, Some(val))
    }

    /// Implements the CPU's memory map
    fn memory_access(&self, addr: u16, val: Option<u8>) -> u8 {
        // FIXME placeholder
        0
        // if addr < 0x2000 {
        //     match val {
        //         Some(val) => self.ram[addr & 0x7FF] = val
        //         None => self.ram[addr & 0x7FF] 
        // }
    }

    fn step(&mut self) {
        let opcode = self.read(self.pc);
        let op = self.decode(opcode);
        op();
        self.pc += 1;

        self.reset = false;
    }

    fn decode(&self, opcode: u8) -> Box<Fn()> {
        Box::new(move || println!("No!"))
    }


}
