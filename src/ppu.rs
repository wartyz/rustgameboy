use crate::mmu::MMU;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

#[derive(Debug)]
pub struct PPU {
    mode: u8,
    mode_clock: usize,
    //    buffer: Vec<u32>,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mode: 0,
            //            buffer: vec![0; WIDTH * HEIGHT],
            mode_clock: 0,
        }
    }

    pub fn step(&mut self, cpu_clocks_passed: usize, mmu: &mut MMU) {
        // Comprueba si el LCD está habilitado
        let lcdc: u8 = mmu.read_byte(0xFF40);
        let is_lcd_enable = (lcdc & 0b1000_0000) != 0;

        // Si LCD está habilitado
        if is_lcd_enable {
            // incrementar el reloj interno
            self.mode_clock += cpu_clocks_passed;
            // probar en que modo estamos
            let mut ly: u8 = mmu.read_byte(0xFF44);
            if self.mode_clock > 456 && self.mode != 1 {
                ly = ly.wrapping_add(1);
                mmu.write_byte(0xFF44, ly);
                if ly <= 144 {
                    self.mode_clock = 0;
                }
            }

            match self.mode_clock {
                t if t <= 80 => self.mode = 2,
                t if t <= 252 => self.mode = 3, // Acumulado 80 + 172
                t if t <= 456 => self.mode = 0, // Acumulado 80 + 172 + 204
                t if t <= 4560 => self.mode = 1,// Cada 10 lineas
                t if t > 4560 => {
                    self.mode = 2;
                    self.mode_clock = 0;
                    if ly > 154 { //TODO: ERROR
                        mmu.write_byte(0xFF44, 0);
                    }
                }
                _ => panic!("mode_clock no manejado!"),
            }

            // cambiar los registros apropiados de la PPU (LY, LYC, STAT)
            // @TODO Check LYC behavior
            let lyc = mmu.read_byte(0xFF45);
            let stat_bit_0_to_2: u8 = match ly == lyc {
                true => 0b100 | self.mode, // bit 3 es flag de coincidencia (ly == lyc)
                false => self.mode,
            } as u8;
            let mut current_stat = mmu.read_byte(0xFF41);
            current_stat = current_stat & 0b11111000;
            current_stat = current_stat | stat_bit_0_to_2;
            // set registro STAT
            mmu.write_byte(0xFF41, current_stat);
        }
    }
}