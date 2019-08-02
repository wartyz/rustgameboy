/*
$8000-$97FF - Character RAM
This area is RAM inside the GB unit, and is used exclusively for video purposes.
This area is also known as Tile RAM, since it holds tiles. Each tile is 8x8 pixels
of 2-bit color, which makes each tile 16 bytes long. This area is also divided up
into two modes of tiles, signed and unsigned. In unsigned mode, tiles are numbered
from 0-255 at $8000-$9000. In signed mode, tiles are numbered in two's complement
from -127 to 128 at $87FF-$97FF. I think... lol. Generally most ppl use 0-255 tiles,
since it's nice and easy. */

use std::fmt;

pub struct MMU {
    ram: [u8; 65_536], //0x0000 to 0xFFFF
}

impl fmt::Debug for MMU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n$FF40 - LCDC: {:b} Elige TILE, \n\
             $FF41 - STAT: {:b}, \n\
             $FF42 - SCY: {:#X}, \n\
             $FF43 - SCX: {:#X}, \n\
             $FF44 - LY: {:?}, \n\
             $FF45 - LYC: {:?}, \n\
             $FF46 - DMA: {:#X}, \n\
             $FF47 - BGP: {:b}, \n\
             $FF48 - OBP0: {:b}, \n\
             $FF49 - OBP1: {:b}, \n\
             $FF4A - WY: {:#X}, \n\
             $FF4B - WY: {:#X}, \n\


            BG Tile Data: {:?} modo unsigned\n
            BG Tile Map: {:?} bg tile map 0\n",
            &self.ram[0xFF40],
            &self.ram[0xFF41],
            &self.ram[0xFF42],
            &self.ram[0xFF43],
            &self.ram[0xFF44],
            &self.ram[0xFF45],
            &self.ram[0xFF46],
            &self.ram[0xFF47],
            &self.ram[0xFF48],
            &self.ram[0xFF49],
            &self.ram[0xFF4A],
            &self.ram[0xFF4B],
            &self.ram[0x8000..0x9000],
            &self.ram[0x9800..0x9C00],
            //            "VRAM: {:?}\n\nOAM RAM: {:?}\n\nIO RAM: {:?}\n\nH RAM: {:?}\n\n",
            //            &self.ram[0x8000..0xA001],
            //            &self.ram[0xFE00..0xFEA1],
            //            &self.ram[0xFF00..0xFF81],
            //            &self.ram[0xFE80..],
        )
    }
}

impl MMU {
    pub fn new() -> MMU {
        let mem = MMU { ram: [0; 65_536] }; // Poner 255 para ver cuando se pone 0
        mem
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn from_rom_file(&mut self, rom_file: &[u8]) {
        //let bytes = &rom_file[..rom_file.len()];
        let mut i: u16 = 0x0000;
        for &byte in rom_file.iter() {
            //println!("{:#x}", (byte as u16));
            self.write_byte(i, byte);
            i += 1;
        }
    }
}
