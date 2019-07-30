/* Emulador que puede servir para comprobar que va bien
 http://www.z80.info/zip/zemu.zip */
use main::{MMU, CPU};

use std::fs::File;
use std::io::Read;

fn main() {
    let a0xFF00 = 5;

// Lee el fichero ROM
    let mut f = File::open("ROMS/DMG_ROM.bin").unwrap();
    let mut rom_file = Vec::<u8>::new();
    f.read_to_end(&mut rom_file);

    // Pone el fichero ROM en la memoria RAM
    let mut mmu = MMU::new();
    mmu.from_rom_file(&rom_file);

// Ejecuta instrucciones en RAM
    //println!("MMU ANTES: {:?}", mmu);
    let mut cpu = CPU::new();
    //cpu.set_debug_flag();
    loop {
        cpu.run_instruction(&mut mmu);
    }
}
