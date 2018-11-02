use bitfield::BitRange;

use ndarray::Array3;

/// Width of the screen for NTSC systems
const SCREEN_WIDTH: usize = 256;

/// Height of the screen for NTSC systems
const SCREEN_HEIGHT: usize = 240;

const PALETTE: [u32; 64] = [
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E, 0x6E0040, 0x6C0600, 0x561D00, 0x333500,
    0x0B4800, 0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000, 0x000000, 0xADADAD, 0x155FD9,
    0x4240FF, 0x7527FE, 0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00, 0x6B6D00, 0x388700, 0x0C9300,
    0x008F32, 0x007C8D, 0x000000, 0x000000, 0x000000, 0xFFFEFF, 0x64B0FF, 0x9290FF, 0xC676FF,
    0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22, 0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE,
    0x4F4F4F, 0x000000, 0x000000, 0xFFFEFF, 0xC0DFFF, 0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA,
    0xFECCC5, 0xF7D8A5, 0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC, 0xB5EBF2, 0xB8B8B8, 0x000000,
    0x000000,
];

bitfield!{
    struct PpuCtrl(u8);
    impl Debug;
    pub base_nametable_addr, set_base_nametable_addr: 1, 0;
    pub vram_addr_incr, set_vram_addr_incr: 2;
    pub sprite_pattern_table_addr, set_sprite_pattern_table_addr: 3;
    pub background_pattern_table_addr, set_background_pattern_table_addr: 4;
    pub sprite_size, set_sprite_size: 5;
    pub master_slave_select, set_master_slave_select: 6;
    pub nmi_vblank, set_n: 7;
}

bitfield!{
    struct PpuMask(u8);
    impl Debug;
    pub grayscale, set_grayscale: 0;
    pub show_background_left, set_show_background_left: 1;
    pub show_sprites_left, set_show_sprites_left: 2;
    pub show_background, set_show_background: 3;
    pub show_sprites, set_show_sprites: 4;
    pub emphasize_red, set_emphasize_red: 5;
    pub emphasize_green, set_emphasize_green: 6;
    pub emphasize_blue, set_emphasize_blue: 7;
}

bitfield!{
    struct PpuStatus(u8);
    impl Debug;
    /// Least significant bits previously written into a PPU register
    /// (due to register not being updated for this address)
    pub lsb, set_lsb: 4, 0;
    /// Sprite overflow. The intent was for this flag to be set
    /// whenever more than eight sprites appear on a scanline, but a
    /// hardware bug causes the actual behavior to be more complicated
    /// and generate false positives as well as false negatives; see
    /// PPU sprite evaluation. This flag is set during sprite
    /// evaluation and cleared at dot 1 (the second dot) of the
    /// pre-render line.
    pub sprite_overflow, set_sprite_overflow: 5;
    /// Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    /// a nonzero background pixel; cleared at dot 1 of the pre-render
    /// line.  Used for raster timing.
    pub sprite_zero_hit, set_sprite_zero_hit: 6;
    /// Vertical blank has started (0: not in vblank; 1: in vblank).
    /// Set at dot 1 of line 241 (the line *after* the post-render
    /// line); cleared after reading $2002 and at dot 1 of the
    /// pre-render line.
    pub vblank_started, set_vblank_started: 7;
}

pub struct Ppu {
    scanline: usize,
    cycle: usize,
    frame: usize,
    screen: Array3<u8>,

    // Flags
    /// Even (true) or odd (false)
    even: bool,

    //Registers
    /// $2000 PPUCTRL
    ppu_ctrl: PpuCtrl,
    /// $2001 PPUMASK
    ppu_mask: PpuMask,
    /// $2002 PPUSTATUS
    ppu_status: PpuStatus,
    /// $2003 OAMADDR
    oam_addr: u8,
    /// $2004 OAMDATA
    oam_data: u8,
    /// $2005 PPUSCROLL
    ppu_scroll: u8,
    /// $2006 PPUADDR
    ppu_addr: u8,
    /// $2007 PPUDATA
    ppu_data: u8,
    /// $4014 OAMDMA
    oam_dma: u8,
    // Storage

    // palette_data: [u8; 32],
    // nametable
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            scanline: 0,
            cycle: 0,
            screen: Array3::zeros((256, 240, 3)),
            even: true,
            ppu_ctrl: PpuCtrl(0),
            ppu_mask: PpuMask(0),
            ppu_status: PpuStatus(0),
            oam_addr: 0,
            oam_data: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_dma: 0,
            frame: 0,
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x2002 => self.ppu_status.bit_range(7, 0),
            0x2004 => self.oam_data.bit_range(7, 0),
            0x2007 => self.ppu_data.bit_range(7, 0),
            _ => panic!("{:?} is not a readable register.", addr),
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            0x2000 => self.ppu_ctrl.set_bit_range(7, 0, val),
            0x2001 => self.ppu_mask.set_bit_range(7, 0, val),
            0x2003 => self.oam_addr = val,
            0x2004 => self.oam_data = val,
            0x2005 => self.ppu_scroll = val,
            0x2006 => self.ppu_addr = val,
            0x2007 => self.ppu_data = val,
            0x4014 => self.oam_dma = val,
            _ => panic!("{:?} is not a register!", addr),
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.frame = 0;
        self.ppu_ctrl.set_bit_range(7, 0, 0);
        self.ppu_mask.set_bit_range(7, 0, 0);
        self.oam_addr.set_bit_range(7, 0, 0);
    }

    fn tick(&mut self) {
        
        // TODO: NMI
        
        if self.ppu_mask.show_background() || self.ppu_mask.show_sprites() {
            if self.even && self.scanline == 261 && self.cycle == 339 {
                self.cycle = 0;
                self.scanline = 0;
                self.frame += 1;
                self.even = !self.even;
                return;
        }

        self.cycle += 1;

        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;
                self.even = !self.even;
            }
        }
    }

    pub fn step(&mut self) {}
}
