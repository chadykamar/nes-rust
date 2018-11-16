use bitfield::{Bit, BitRange};

bitfield!{
    /// Represents the statuses of the buttons. Every button is represented by
    /// one bit with
    pub struct Buttons(u8);
    impl Debug;
    pub a, set_a: 0;
    pub b, set_b: 1;
    pub select, set_select: 2;
    pub start, set_start: 3;
    pub up, set_up: 4;
    pub down, set_down: 5;
    pub left, set_left: 6;
    pub right, set_right: 7;
}

/// The main Controller structure.
pub struct Controller {
    /// A bitfield representing the current status of the controller.
    pub buttons: Buttons,

    /// The index of the next bit (button) to be read
    index: u8,

    ///
    strobe: u8,
}

impl Controller {
    /// Initialize
    pub fn new() -> Controller {
        Controller {
            buttons: Buttons(0),
            index: 0,
            strobe: 0,
        }
    }

    pub fn raw_buttons(&self) -> u8 {
        self.buttons.bit_range(7, 0)
    }

    pub fn read(&mut self) -> u8 {
        let val = if self.index < 8 && self.buttons.bit(self.index as usize) {
            1
        } else {
            0
        };

        self.index += 1;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }
        return val;
    }

    pub fn write(&mut self, val: u8) {
        self.strobe = val;
        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }
}
