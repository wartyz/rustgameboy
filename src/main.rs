/* Emulador que puede servir para comprobar que va bien
http://www.z80.info/zip/zemu.zip */
use gbrustemu::cpu::CPU;
use gbrustemu::mmu::MMU;
use gbrustemu::ppu::PPU;

use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Read;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    // Lee el fichero ROM
    let mut f = File::open("ROMS/DMG_ROM.bin").unwrap();
    let mut rom_file = Vec::<u8>::new();
    f.read_to_end(&mut rom_file).unwrap();

    // Pone el fichero ROM en la memoria RAM
    let mut mmu = MMU::new();
    mmu.from_rom_file(&rom_file);

    // Ejecuta instrucciones en RAM
    //println!("MMU ANTES: {:?}", mmu);
    let mut cpu = CPU::new();
    let mut ppu = PPU::new();

    //cpu.set_debug_flag(); // qUITAR ESTA LÍNEA SI NO SE QUIERE DEBUG

    let mut screen = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Prueba - ESC para salir",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.run_instruction(&mut mmu, &mut ppu);

        //        if ppu.is_lcd_enable() {
        //            for i in screen.iter_mut() {
        //                *i = 0;
        //            }
        //
        //            window.update_with_buffer(&screen).unwrap();
        //        }
    }

    //        println!("MMU state: {:?}", mmu);
    //        println!("CPU state: {:?}", cpu);
    //        println!("PPU state: {:?}", ppu);
}
