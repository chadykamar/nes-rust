use bitfield::BitRange;

use mapper::Mapper;
use ndarray::Array2;

use std::rc::Rc;
use std::cell::RefCell;

/// Width of the screen for NTSC systems
pub const SCREEN_WIDTH: usize = 256;

/// Height of the screen for NTSC systems
pub const SCREEN_HEIGHT: usize = 240;

pub const PALETTE: [u8; 192] = [
    0x7C, 0x7C, 0x7C, 0x00, 0x00, 0xFC, 0x00, 0x00, 0xBC, 0x44, 0x28, 0xBC, 0x94, 0x00, 0x84, 0xA8,
    0x00, 0x20, 0xA8, 0x10, 0x00, 0x88, 0x14, 0x00, 0x50, 0x30, 0x00, 0x00, 0x78, 0x00, 0x00, 0x68,
    0x00, 0x00, 0x58, 0x00, 0x00, 0x40, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xBC, 0xBC, 0xBC, 0x00, 0x78, 0xF8, 0x00, 0x58, 0xF8, 0x68, 0x44, 0xFC, 0xD8, 0x00, 0xCC, 0xE4,
    0x00, 0x58, 0xF8, 0x38, 0x00, 0xE4, 0x5C, 0x10, 0xAC, 0x7C, 0x00, 0x00, 0xB8, 0x00, 0x00, 0xA8,
    0x00, 0x00, 0xA8, 0x44, 0x00, 0x88, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xF8, 0xF8, 0xF8, 0x3C, 0xBC, 0xFC, 0x68, 0x88, 0xFC, 0x98, 0x78, 0xF8, 0xF8, 0x78, 0xF8, 0xF8,
    0x58, 0x98, 0xF8, 0x78, 0x58, 0xFC, 0xA0, 0x44, 0xF8, 0xB8, 0x00, 0xB8, 0xF8, 0x18, 0x58, 0xD8,
    0x54, 0x58, 0xF8, 0x98, 0x00, 0xE8, 0xD8, 0x78, 0x78, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xFC, 0xFC, 0xFC, 0xA4, 0xE4, 0xFC, 0xB8, 0xB8, 0xF8, 0xD8, 0xB8, 0xF8, 0xF8, 0xB8, 0xF8, 0xF8,
    0xA4, 0xC0, 0xF0, 0xD0, 0xB0, 0xFC, 0xE0, 0xA8, 0xF8, 0xD8, 0x78, 0xD8, 0xF8, 0x78, 0xB8, 0xF8,
    0xB8, 0xB8, 0xF8, 0xD8, 0x00, 0xFC, 0xFC, 0xF8, 0xD8, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

bitfield!{
    /// Byte 2 of a `Sprite`
    /// Bits 2, 3 and 4 are unimplemented.
    #[derive(Copy, Clone)]
    struct SpriteAttributes(u8);
    impl Debug;
    /// Palette (4 to 7) of sprite
    pub palette, set_palette: 1, 0;
    /// Priority (0: in front of background; 1: behind background)
    pub priority, set_priority: 5;
    /// Flip sprite horizontally.
    /// Flipping does not change the position of the sprite's bounding box, just
    /// the position of pixels within the sprite. If, for example, a sprite co-
    /// vers (120, 130) through (127, 137), it'll still cover the same area
    /// when flipped.
    pub flip_h, set_flip_h: 6;
    /// Flip sprite vertically.
    /// * In 8x16 mode, vertical flip flips each of the subtiles and also exchan-
    /// ges their position; the odd-numbered tile of a vertically flipped sprite
    /// is drawn on top.
    /// * Flipping does not change the position of the sprite's bounding box, just
    /// the position of pixels within the sprite. If, for example, a sprite
    /// covers (120, 130) through (127, 137), it'll still cover the same area
    /// when flipped.
    pub flip_v, set_flip_v: 7;
}

#[derive(Copy, Clone)]
struct Sprite {
    /// Y position of top of sprite.
    /// Sprite data is delayed by one scanline; you must subtract 1 from
    /// the sprite's Y coordinate before writing it here. Hide a sprite by
    /// writing any values in $EF-$FF here. Sprites are never displayed on the
    /// first line of the picture, and it is impossible to place a sprite
    /// partially off the top of the screen.
    y: u8,
    /// Tile index number.
    /// * For 8x8 sprites, this is the tile number of this sprite within the
    /// pattern table selected in bit 3 of PPUCTRL ($2000).
    /// * For 8x16 sprites, the PPU ignores the pattern table selection and
    /// selects a pattern table from bit 0 of this number.
    tile: u8,
    /// Sprite attributes, including palette priority, and flipping.
    attributes: SpriteAttributes,
    /// X position of left side of sprite.
    /// X-scroll values of $F9-FF results in parts of the sprite to be past the
    /// right edge of the screen, thus invisible. It is not possible to have a
    /// sprite partially visible on the left edge. Instead, left-clipping
    /// through PPUMASK ($2001) can be used to simulate this effect.
    x: u8,
}

///
bitfield!{
    /// The PPUSCTRL register
    struct PpuCtrl(u8);
    impl Debug;

    /// Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    pub base_nametable_addr, set_base_nametable_addr: 1, 0;
    /// VRAM address increment per CPU read/write of PPUDATA
    /// (0: add 1, going across; 1: add 32, going down)
    pub vram_addr_incr, set_vram_addr_incr: 2;
    /// Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; 
    /// ignored in 8x16 mode)
    pub sprite_pattern_table_addr, set_sprite_pattern_table_addr: 3;
    /// Background pattern table address (0: $0000; 1: $1000)
    pub background_pattern_table_addr, set_background_pattern_table_addr: 4;
    /// Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    pub sprite_size, set_sprite_size: 5;
    /// PPU Master/Slave select (0: read backdrop from EXT pins; 1: output
    /// color on EXT pins)
    pub master_slave_select, set_master_slave_select: 6;
    /// Generate an NMI at the start of the vertical blanking interval
    /// (0: off, 1: on)
    /// 
    /// Vblank is the time between the end of the final line of a frame and the
    /// beginning of the first line of the next frame.
    pub nmi_vblank, set_nmi_vblank: 7;
}

bitfield!{
    /// The PPUMASK register
    struct PpuMask(u8);
    impl Debug;
    /// Greyscale (0: normal color, 1: produce a greyscale display)
    pub grayscale, set_grayscale: 0;
    // 1: Show background in leftmost 8 pixels of screen, 0: Hide
    pub show_background_left, set_show_background_left: 1;
    /// 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    pub show_sprites_left, set_show_sprites_left: 2;
    /// 1: Show background, 0: Hide background
    pub show_background, set_show_background: 3;
    /// 1: Show sprites, 0: Hide sprites
    pub show_sprites, set_show_sprites: 4;
    /// Emphasize red. 
    /// 
    /// Note that the emphasis bits are applied independently of bit 0, so they
    /// will still tint the color of the grey image.
    pub emphasize_red, set_emphasize_red: 5;
    /// Emphasize green
    /// 
    /// Note that the emphasis bits are applied independently of bit 0, so they
    /// will still tint the color of the grey image.
    pub emphasize_green, set_emphasize_green: 6;
    /// Emphasize blue
    /// 
    /// Note that the emphasis bits are applied independently of bit 0, so they
    /// will still tint the color of the grey image.
    pub emphasize_blue, set_emphasize_blue: 7;
}

bitfield!{
    /// The PPUSTATUS Register implementation.
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

pub struct Ppu<'a> {
    mapper: Rc<RefCell<Box<Mapper>>>,
    scanline: usize,
    cycle: usize,
    frame: usize,
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    vram_addr: u16,
    temp_vram_addr: u16,


    /// Object Attribute Memory which contains a display list of up to 64
    /// sprites, where each sprite's information occupies 4 bytes.
    primary_oam: [&'a Sprite; 64],
    /// Secondary OAM contains 
    secondary_oam: Vec<(&'a Sprite, usize)>,

    nt: [u8; 0x800],

    // The NES uses two palettes, each with 16 entries, the image palette ($3F00-$3F0F) and the
    // sprite palette ($3F10-$3F1F). Since only 64 unique values are needed,
    // bits 6 and 7 can be ignored.
    /// The image palette shows colors available for background tiles.
    /// It does not store the actual color values, only the index for the color
    /// in the system palette `PALETTE`.
    image_palette: [u8; 16],
    /// The sprite palette shows the colours currently available for sprites.
    /// It does not store the actual color values, only the index for the color
    /// in the system palette `PALETTE`.
    sprite_palette: [u8; 16],

    // nametable: [u8; 0x800],
    

    // Flags
    /// Write toggle.
    write: bool,
    /// Even (true) or odd (false)
    even: bool,

    // Temporary variables
    // TODO: Refactor
    nametable: u8,
    attribute_table: u8,
    low_tile: u8,
    high_tile: u8,
    tile_data: usize,

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

impl<'a> Ppu<'a> {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Ppu<'static> {
        Ppu {
            mapper: mapper,
            scanline: 0,
            cycle: 0,
            frame: 0,
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            nt: [0; 0x800],

            image_palette: [0; 16],
            sprite_palette: [0; 16],

            primary_oam: [&Sprite {
                y: 0,
                tile: 0,
                x: 0,
                attributes: SpriteAttributes(0),
            }; 64],
            secondary_oam: vec![],
            nametable: 0,
            vram_addr: 0,
            temp_vram_addr: 0,
            attribute_table: 0,
            low_tile: 0,
            high_tile: 0,
            tile_data: 0,

            even: true,
            write: true,

            ppu_ctrl: PpuCtrl(0),
            ppu_mask: PpuMask(0),
            ppu_status: PpuStatus(0),
            oam_addr: 0,
            oam_data: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_dma: 0,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x1FFF => self.mapper.borrow_mut().read(addr),
            0x2000...0x3EFF => self.nt[addr as usize % 0x800],
            0x3F00...0x3F0F => self.image_palette[addr as usize],
            0x3F10...0x3F1F => self.sprite_palette[addr as usize],
            0x3F20...0x3FFF => self.read(((addr - 0x3F00) % 32) + 0x3F00),
            _ => panic!("Invalid read address {:?}", addr)
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1FFF => self.mapper.borrow_mut().write(addr, val),
            0x2000...0x3EFF => self.nt[addr as usize % 0x800] = val,
            0x3F00...0x3F0F => self.image_palette[addr as usize] = val,
            0x3F10...0x3F1F => self.sprite_palette[addr as usize] = val,
            0x3F20...0x3FFF => self.write(((addr - 0x3F00) % 32) + 0x3F00, val),
            _ => panic!("Invalid write address {:?}", addr)
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

    /// PPU sprite evaluation is an operation done by the PPU once each
    /// scanline. It prepares the set of sprites and fetches their data to be
    /// rendered on the next scanline. Each scanline, the PPU reads the
    /// Object Attribute Memory to see which to draw:
    /// * First, it clears the list of sprites to draw.
    /// * Second, it reads through OAM, checking which sprites will be on this
    /// scanline. It chooses the first eight it finds that do.
    /// * Third, if eight sprites were found, it checks
    /// (in a wrongly-implemented fashion) for further sprites on the scanline
    /// to see if the sprite overflow flag should be set.
    /// * Fourth, using the details for the eight (or fewer) sprites chosen,
    /// it determines which pixels each has on the scanline and where to draw them.
    fn evaluate_sprites(&mut self) {
        let height = if self.ppu_ctrl.sprite_size() { 8 } else { 16 };
        self.secondary_oam = vec![];

        let mut count = 0;
        for sprite in self.primary_oam.iter() {
            let row = self.scanline - sprite.y as usize;
            if row >= height {
                continue;
            }
            if self.secondary_oam.len() < 8 {
                self.secondary_oam.push((sprite, row));
            }
            count += 1;
        }
        if count > 8 {
            self.ppu_status.set_sprite_overflow(true);
        }
    }

    fn sprite_pixel(&self) -> (usize, u32, Option<Sprite>) {
        if !self.ppu_mask.show_sprites() {
            return (0, 0, None);
        }
        let x = self.cycle - 1;
        if x < 8 && !self.ppu_mask.show_sprites_left() {
            return (0, 0, None);
        }

        for (i, (sprite, row)) in self.secondary_oam.iter().enumerate() {
            let x = self.cycle - 1;
            let mut offset = x - sprite.x as usize;
            if offset < 0 || offset > 7 {
                continue;
            }
            offset = 7 - offset;
            let color = (self.sprite_pattern(sprite, *row) >> (offset * 4)) & 0x0F;
            if color % 4 == 0 {
                continue;
            }
            return (i, color, Some(**sprite));
        }
        (0, 0, None)
    }

    fn sprite_pattern(&self, sprite: &Sprite, mut row: usize) -> u32 {
        let mut tile = sprite.tile;

        row = (if self.ppu_ctrl.sprite_size() { 15 } else { 7 } as usize) - row;

        let addr = if !self.ppu_ctrl.sprite_size() {
            if sprite.attributes.flip_v() {
                row = 7 - row;
            }

            let table_addr = 0x1000 * self.ppu_ctrl.sprite_pattern_table_addr() as u16;

            table_addr + 16 * tile as u16 + row as u16
        } else {
            if sprite.attributes.flip_v() {
                row = 15 - row;
            }

            let table_addr = 0x1000 * (tile as u16 & 1);
            if row > 7 {
                tile += 1;
                row -= 8;
            }

            table_addr + 16 * tile as u16 + row as u16
        };

        let palette = sprite.attributes.palette() + 4;

        let mut low_tile = self.read(addr);
        let mut high_tile = self.read(addr + 8);

        let mut pattern: u32 = 0;

        for _ in 0..7 {
            let mut a;
            let mut b;

            if sprite.attributes.flip_h() {
                a = (low_tile & 1) << 0;
                b = (high_tile & 1) << 1;
                low_tile >>= 1;
                high_tile >>= 1;
            } else {
                a = (low_tile & 0x80) >> 7;
                b = (high_tile & 0x80) >> 6;
                low_tile <<= 1;
                high_tile <<= 1;
            }
            pattern <<= 4;
            pattern |= (palette | a | b) as u32;
        }
        pattern
    }

    fn background_pixel(&self) -> u32 {
        if !self.ppu_mask.show_background() {
            return 0;
        }
        let x = self.cycle - 1;
        if x < 8 && !self.ppu_mask.show_background_left() {
            return 0;
        }

        (self.tile_data >> 32) as u32 >> ((7 - self.ppu_scroll) * 4) & 0x0F
    }

    fn render_pixel(&mut self) {
        let (x, y) = (self.cycle - 1, self.scanline);

        let mut background_color = self.background_pixel();
        let (i, sprite_color, sprite) = self.sprite_pixel();

        let color = match (background_color % 4 == 0, sprite_color % 4 == 0) {
            (true, true) => 0,
            (true, false) => sprite_color | 0x10,
            (false, true) => background_color,
            (false, false) => {
                self.ppu_status.set_sprite_zero_hit(i == 0 && x < 255);

                if sprite.unwrap().attributes.priority() {
                    background_color
                } else {
                    sprite_color | 0x10
                }
            }
        };
        let addr = (color as u16) % 64;
        let index = self.read(((addr - 0x3F00) % 32) + 0x3F00) as usize;
        let r = PALETTE[index as usize * 3 + 2];
        let g = PALETTE[index as usize * 3 + 1];
        let b = PALETTE[index as usize * 3 + 0];

        self.screen[(y * SCREEN_WIDTH + x) * 3 + 0] = r;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 1] = g;
        self.screen[(y * SCREEN_WIDTH + x) * 3 + 2] = b;

        println!("{} {} {}", r, g, b);

    }

    pub fn step(&mut self) {
        // TODO NMI and VBLANK
        self.tick();
        let pre_render_line = self.scanline == 261;
        let visible_line = self.scanline < 240;


        let pre_fetch_cycle = 321 <= self.cycle && self.cycle <= 336;
        let visible_cycle = 1 <= self.cycle && self.cycle <= 256;

        if self.ppu_mask.show_background() {
            if visible_line && visible_cycle {
                self.render_pixel()
            }

            if (pre_render_line || visible_line) && (pre_fetch_cycle || visible_cycle) {
                self.tile_data <<= 4;
                match self.cycle % 8 {
                    1 => {
                        let addr = 0x2000 | (self.vram_addr & 0x0FFF);
                        self.nametable = self.read_register(addr);
                    }
                    3 => {
                        let a = self.vram_addr;
                        let addr = 0x23C0 | (a & 0x0C00) | ((a >> 4) & 0x38) | ((a >> 2) & 0x07);
                        let shift = ((a >> 4) & 4) | (a & 2);
                        self.attribute_table = ((self.read_register(addr) >> shift) & 3) << 2;
                    }
                    5 => {
                        let fine_y = (self.vram_addr >> 12) & 7;
                        let table = self.ppu_ctrl.background_pattern_table_addr() as u16;
                        let addr = 0x1000 * table + self.nametable as u16 * 16 + fine_y;
                        self.low_tile = self.read_register(addr);
                    }
                    7 => {
                        let fine_y = (self.vram_addr >> 12) & 7;
                        let table = self.ppu_ctrl.background_pattern_table_addr() as u16;
                        let addr = 0x1000 * table + self.nametable as u16 * 16 + fine_y;
                        self.high_tile = self.read_register(addr + 8);
                    }
                    0 => {
                        let mut data: usize = 0;
                        for _ in 0..8 {
                            let a = self.attribute_table;
                            let low = (self.low_tile & 0x80) >> 7;
                            let high = (self.high_tile & 0x80) >> 6;
                            self.low_tile <<= 1;
                            self.high_tile <<= 1;
                            data <<= 4;
                            data |= a as usize | low as usize | high as usize;
                        }
                        self.tile_data |= data;
                    }
                    _ => {}
                }
            }
            if pre_render_line && 280 <= self.cycle && self.cycle <= 304 {
                self.vram_addr = (self.vram_addr & 0x841F) | (self.temp_vram_addr & 0x7BE0);
            }

            if pre_render_line || visible_line {
                if pre_fetch_cycle && self.cycle % 8 == 0 {
                    if self.vram_addr & 0x001F == 31 {
                        self.vram_addr &= 0xFFE0;
                        self.vram_addr ^= 0x0400;
                    } else {
                        self.vram_addr += 1;
                    }
                }
                if self.cycle == 256 {
                    if self.vram_addr & 0x7000 != 0x7000 {
                        self.vram_addr += 0x1000
                    } else {
                        self.vram_addr &= 0x8FFF;
                        let mut y = (self.vram_addr & 0x03E0) >> 5;
                        if y == 29 {
                            y = 0;
                            self.vram_addr ^= 0x0800;
                        } else if y == 31 {
                            y = 0;
                        } else {
                            y += 1;
                        }
                        self.vram_addr = (self.vram_addr & 0xFC1F) | (y << 5);
                    }
                }
                if self.cycle == 257 {
                    self.vram_addr = (self.vram_addr & 0xFBE0) | (self.temp_vram_addr & 0x041F);
                }
            }
        }

        if self.ppu_mask.show_sprites() {
            if self.cycle == 257 {
                if visible_line {
                    self.evaluate_sprites();
                } else {
                    // Sprites

                }
            }
        }

        // vblank
        if self.scanline == 241 && self.cycle == 1 {}
        if self.scanline == 261 && self.cycle == 1 {
            self.ppu_status.set_sprite_zero_hit(false);
            self.ppu_status.set_sprite_overflow(false);
        }
    }
}
