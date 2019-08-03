use crate::mmu::MMU;
//use minifb::Key;
//use minifb::Window;
//use minifb::WindowOptions;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

//Colores ALPHA,R,G,B
pub const DARKEST_GREEN: u32 = 0xFF0F380F;
pub const DARK_GREEN: u32 = 0xFF306230;
pub const LIGHT_GREEN: u32 = 0xFF8BAC0F;
pub const LIGHTEST_GREEN: u32 = 0xFF9BBC0F;

#[derive(Debug)]
pub struct PPU {
    mode: u8,
    mode_clock: usize,
    buffer: Vec<u32>,
}

impl PPU {
    pub fn new() -> PPU {
        let ppu = PPU {
            mode: 0,
            buffer: vec![0; WIDTH * HEIGHT],
            mode_clock: 0,
        };

        ppu
    }

    // Registro LCDC
    pub fn get_lcdc(&self, mmu: &MMU) -> u8 {
        mmu.read_byte(0xFF40)
    }

    /// Registro BGP     BGP Palette Data(R/W)
    /// Asigna escala de grises a los números de color de los tiles BG y de ventana
    pub fn get_bgp(&self, mmu: &MMU) -> u8 {
        mmu.read_byte(0xFF47)
    }

    pub fn is_lcd_enable(&self, mmu: &MMU) -> bool {
        (self.get_lcdc(mmu) & 0b1000_0000) != 0
    }

    pub fn get_tile_set(&self, mmu: &MMU) -> [[u8; 16]; 256] {
        // @TODO check LCDC
        let mut tile_set = [[0; 16]; 256];

        for i in 0..256 {
            tile_set[i] = self.get_tile(&mmu, (0x8000 + (i * 16)) as u16);
        }
        tile_set
    }

    pub fn get_tile_map(&self, mmu: &MMU) {}

    pub fn get_tile(&self, mmu: &MMU, first_tile_byte_addr: u16) -> [u8; 16] {
        let mut tile = [0; 16];
        for i in 0..16 {
            tile[i] = mmu.read_byte(first_tile_byte_addr + i as u16);
        }
        tile
    }

    /// Primera fase pares de bits a paleta de background
    pub fn transform_pair_into_bgp_palette(&self, mmu: &MMU, pixel_pair: u8) -> u8 {
        //println!("bgp_palette = {:b}", self.get_bgp(&mmu));
        let bgp_palette = self.get_bgp(&mmu);
        match pixel_pair {
            0b00 => bgp_palette & 0b0000_0011,
            0b01 => (bgp_palette & 0b0000_1100) >> 2,
            0b10 => (bgp_palette & 0b0011_0000) >> 4,
            0b11 => (bgp_palette & 0b1100_0000) >> 6, // TODO: ERROR el pone 4

            _ => bgp_palette & 0b0000_0011,
        }
    }

    /// Segunda fase paleta de background a color para que lo entienda minifb
    pub fn transform_from_bgp_to_minifb_color(&self, bgp_palette: u8) -> u32 {
        match bgp_palette {
            0b00 => LIGHTEST_GREEN,
            0b01 => LIGHT_GREEN,
            0b10 => DARK_GREEN,
            0b11 => DARKEST_GREEN,

            _ => LIGHTEST_GREEN,
        }
    }

    /// Convierte tile en un arreglo de bits ARGB para que lo entienda minifb
    pub fn transform_tile_to_minifb_tile(&self, mmu: &MMU, tile: [u8; 16]) -> [[u32; 8]; 8] {
        let mut minifb_tile: [[u32; 8]; 8] = [[0; 8]; 8];
        for i in (0..tile.len()).step_by(2) {
            let pixel_part_1 = tile[i];
            let pixel_part_2 = tile[i + 1];
            //            println!("first byte line: {:08b}", tile[i]);
            //            println!("second byte line: {:08b}", tile[i + 1]);
            for j in 0..8 {
                let bit_part_1 = pixel_part_1 & (1 << j) != 0;
                let bit_part_2 = pixel_part_2 & (1 << j) != 0;

                let pair = ((bit_part_1 as u8) << 1) | (bit_part_2 as u8);
                //println!("pair {:b}", pair);

                // Transforma este par en una paleta BGP
                // 76     54     32     10        <-(bits en bgp_palette)
                // color3 color2 color1 color 0
                let bgp_palette = self.transform_pair_into_bgp_palette(&mmu, pair);
                //println!("bgp_palette {:b}", bgp_palette);
                // Transforma en color MINIFB
                let minifb = self.transform_from_bgp_to_minifb_color(bgp_palette);

                minifb_tile[i / 2][j] = minifb;
            }
        }

        minifb_tile
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
                // Esto ocurre en HBLANK
                ly = ly.wrapping_add(1);
                mmu.write_byte(0xFF44, ly);
                if ly <= 144 {
                    self.mode_clock = 0;
                }
            }

            match self.mode_clock {
                t if t <= 80 => self.mode = 2,
                t if t <= 252 => self.mode = 3,  // Acumulado 80 + 172
                t if t <= 456 => self.mode = 0,  // Acumulado 80 + 172 + 204
                t if t <= 4560 => self.mode = 1, // Cada 10 lineas
                t if t > 4560 => {
                    self.mode = 2;
                    self.mode_clock = 0;
                    if ly > 154 {
                        // Es correcto, un frame entero cada 154 scanlines
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
