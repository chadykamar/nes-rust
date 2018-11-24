use crate::util;

use std::fmt;
use std::io::{self, Read};
use std::vec::Vec;

#[derive(Debug)]
pub enum RomLoadError {
    /// IO error while reading the ROM image
    IoError(io::Error),
    /// The ROM image has an invalid format
    FormatError,
}

impl From<io::Error> for RomLoadError {
    fn from(err: io::Error) -> Self {
        RomLoadError::IoError(err)
    }
}

/// A ROM image
pub struct Rom {
    pub header: INesHeader,
    /// PRG-ROM
    pub prg: Vec<u8>,
    /// CHR-ROM
    pub chr: Vec<u8>,
    /// SRAM
    pub sram: [u8; 0x2000],
}

impl Rom {
    pub fn load(r: &mut Read) -> Result<Rom, RomLoadError> {
        let mut header = [0u8; 16];
        r#try!(util::read_to_buf(&mut header, r));

        let header = INesHeader {
            magic: [header[0], header[1], header[2], header[3]],
            prg_rom_size: header[4],
            chr_rom_size: header[5],
            control_byte_1: header[6],
            control_byte_2: header[7],
            prg_ram_size: header[8],
            zero: [0; 7],
        };

        if header.magic != *b"NES\x1a" {
            return Err(RomLoadError::FormatError);
        }

        let prg_bytes = header.prg_rom_size as usize * 16384;
        let mut prg_rom = vec![0u8; prg_bytes];
        r#try!(util::read_to_buf(&mut prg_rom, r));

        let chr_bytes = header.chr_rom_size as usize * 8192;
        let mut chr_rom = vec![0u8; chr_bytes];
        r#try!(util::read_to_buf(&mut chr_rom, r));

        Ok(Rom {
            header: header,
            prg: prg_rom,
            chr: chr_rom,
            sram: [0; 0x2000],
        })
    }
}

pub struct INesHeader {
    /// Should contain 'N' 'E' 'S' '\x1a' to identify the file as an iNES file.
    pub magic: [u8; 4],
    /// Number of 16 KB PRG-ROM banks.
    ///
    /// The PRG-ROM (Program ROM) is the area of ROM used to store the program
    /// code.
    pub prg_rom_size: u8,
    /// Number of 8 KB CHR-ROM / VROM banks.
    ///
    /// The names CHR-ROM (Character ROM)
    /// and VROM are used synonymously to refer to the area of ROM used to
    /// store graphics information, the pattern tables.
    pub chr_rom_size: u8,
    /// MMMMATPA
    ///
    /// * M: Low nibble of mapper number
    /// * A: 0xx0: vertical arrangement/horizontal mirroring (CIRAM A10 = PPU A11)
    ///      0xx1: horizontal arrangement/vertical mirroring (CIRAM A10 = PPU A10)
    ///      1xxx: four-screen VRAM
    /// * T: ROM contains a trainer
    /// * P: Cartridge has persistent memory
    pub control_byte_1: u8,
    /// MMMMVVPU
    ///
    /// * M: High nibble of mapper number
    /// * V: If 0b10, all following flags are in NES 2.0 format
    /// * P: ROM is for the PlayChoice-10
    /// * U: ROM is for VS Unisystem
    pub control_byte_2: u8,
    /// Number of 8 KB RAM banks.
    ///
    /// For compatibility with previous versions of the iNES format, we assume
    /// 1 page of RAM when this is 0.
    pub prg_ram_size: u8,
    /// always zero
    pub zero: [u8; 7],
}

impl INesHeader {
    /// Returns the mapper ID.
    pub fn mapper(&self) -> u8 {
        (self.control_byte_2 & 0xf0) | (self.control_byte_1 >> 4)
    }

    /// Returns the low nibble of the mapper ID.
    pub fn ines_mapper(&self) -> u8 {
        self.control_byte_1 >> 4
    }

    pub fn trainer(&self) -> bool {
        (self.control_byte_1 & 0x04) != 0
    }
}

impl fmt::Display for INesHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "PRG-ROM: {} KB, CHR-ROM: {} KB, Mapper: {} ({}), Trainer: {}",
            self.prg_rom_size as u32 * 16,
            self.chr_rom_size as u32 * 8,
            self.mapper(),
            self.ines_mapper(),
            self.trainer(),
        )
    }
}
