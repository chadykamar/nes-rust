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

// struct Sprite {
//     position: u8,
//     pattern: u8,
//     priority: u8,
//     index: u8,
// }

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
    /// Sprite attributes, including palette priority, and flipping.
    attributes: SpriteAttributes,
}

///
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

/// The
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
    scanline: usize,
    cycle: usize,
    frame: usize,
    screen: Array3<u8>,
    vram_addr: u16,
    temp_vram_addr: u16,

    // Mem'
    /// Object Attribute Memory which contains a display list of up to 64
    /// sprites, where each sprite's information occupies 4 bytes.
    oam: [&'a Sprite; 64],

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

    // palette: [u8; 32],
    // nametable: [u8; 0x800],
    sprites: Vec<&'a Sprite>,

    // Flags
    /// Write toggle.
    write: bool,
    /// Even (true) or odd (false)
    even: bool,

    // Temporary variables
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
    pub fn new() -> Ppu<'static> {
        Ppu {
            scanline: 0,
            cycle: 0,
            frame: 0,
            screen: Array3::zeros((256, 240, 3)),

            image_palette: [0; 16],
            sprite_palette: [0; 16],

            // TODO initialize properly
            oam: [&Sprite {
                y: 0,
                tile: 0,
                x: 0,
                attributes: SpriteAttributes(0),
            }; 64],
            nametable: 0,
            vram_addr: 0,
            temp_vram_addr: 0,
            attribute_table: 0,
            low_tile: 0,
            high_tile: 0,
            tile_data: 0,
            sprites: vec![],

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
    /// spritelist (that is, Object Attribute Memory) to see which to draw:
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
        self.sprites = vec![];

        for sprite in self.oam.iter() {
            let row = self.scanline - sprite.y as usize;
            if row >= height {
                continue;
            }
            if self.sprites.len() < 8 {
                self.sprites.push(sprite);
            }
        }
        if self.sprites.len() > 8 {
            self.ppu_status.set_sprite_overflow(true);
        }
    }

    fn render_pixel(&mut self) {}

    pub fn step(&mut self) {
        // TODO NMI and VBLANK
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
