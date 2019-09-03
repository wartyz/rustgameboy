/* Emulador que puede servir para comprobar que va bien
   http://www.z80.info/zip/zemu.zip
   tetris desensamblado:
   https://github.com/osnr/tetris/blob/master/tetris.asm
*/
use gbrustemu::cpu::CPU;
use gbrustemu::mmu::MMU;
use gbrustemu::ppu::{LIGHTEST_GREEN, PPU, SCREEN_HEIGHT, SCREEN_WIDTH};

use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Read;

use std::time::Instant;

//const WIDTH: usize = 160;
//const HEIGHT: usize = 144;
const WIDTH: usize = 256;
const HEIGHT: usize = 256;

fn main() {
    // Lee el fichero ROM
    let mut f = File::open("ROMS/tetris.gb").unwrap();
    let mut rom_file = Vec::<u8>::new();
    f.read_to_end(&mut rom_file).unwrap();

    let mut mmu = MMU::new();
    // Pone el fichero ROM en la memoria RAM
    mmu.from_rom_file(&rom_file);

    // Ejecuta instrucciones en RAM
    let mut cpu = CPU::new();
    let mut ppu = PPU::new();

    let mut screen = vec![LIGHTEST_GREEN; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut window = Window::new(
        "Prueba - ESC para salir",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    while window.is_open() && !window.is_key_down(Key::Escape) {
        //let now = Instant::now();
        cpu.run_instruction(&mut mmu, &mut ppu);

        if ppu.is_lcd_enable(&mmu) {
            if mmu.dirty_viewport_flag || mmu.dirty_vram_flag {
                let current_viewport = ppu.get_viewport();
                window.update_with_buffer(current_viewport).unwrap();
            }
        }
        //let new_now = Instant::now();
        //println!("{:?}", new_now.duration_since(now));
    }
}
