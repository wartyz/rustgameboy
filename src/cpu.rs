use crate::{MMU, Instruction};
use std::fmt;

//#[derive(Debug)]
pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    //t -> time cycles
    t: usize,
    //m -> machine cycles
    m: usize,
    debug: bool,
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CPU \n{{A: {:#X}, B: {:#X}, C: {:#X}, D: {:#X}, E: {:#X}, H: {:#X}, L: {:#X}}} \
        \nflags-> {{Z: {:?}, N: {:?}, H: {:?}, C: {:?}}}\
        \n{{PC: {:#X}, SP: {:#X}}}\n",
               self.a, self.b, self.c, self.d, self.e, self.h, self.l,
               self.get_z_flag(), self.get_n_flag(), self.get_h_flag(),
               self.get_c_flag(), self.pc, self.sp)
    }
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
            t: 0,
            m: 0,
            debug: false,
        };
        cpu
    }


    // DEBUG **********************************
    pub fn set_debug_flag(&mut self) {
        self.debug = true;
    }
    pub fn reset_debug_flag(&mut self) {
        self.debug = false;
    }
    // FIN DEBUG ******************************

    /// Función general que usan las demás funciones de flag
    /// Recibe una máscara indicando el flag a leer y devuelve true o false segun sea 1 o 0
    fn get_flag(&self, bit_mask: u8) -> bool {
        (self.f & bit_mask) != 0
    }
    // Funciones GET de FLAGS
    fn get_z_flag(&self) -> bool {
        self.get_flag(0b1000_0000)
    }
    fn get_n_flag(&self) -> bool {
        self.get_flag(0b0100_0000)
    }
    fn get_h_flag(&self) -> bool {
        self.get_flag(0b0010_0000)
    }
    fn get_c_flag(&self) -> bool {
        self.get_flag(0b0001_0000)
    }

    // Funciones SET de FLAGS
    fn set_z_flag(&mut self) {
        self.f = self.f | 0b1000_0000;
    }
    fn reset_z_flag(&mut self) {
        self.f = self.f & 0b0111_1111;
    }
    fn set_n_flag(&mut self) {
        self.f = self.f | 0b0100_0000;
    }
    fn reset_n_flag(&mut self) {
        self.f = self.f & 0b1011_1111;
    }
    fn set_h_flag(&mut self) {
        self.f = self.f | 0b0010_0000;
    }
    fn reset_h_flag(&mut self) {
        self.f = self.f & 0b1101_1111;
    }
    fn set_c_flag(&mut self) {
        self.f = self.f | 0b0001_0000;
    }
    fn reset_c_flag(&mut self) {
        self.f = self.f & 0b1110_1111;
    }

    // Funciones de Stack
    /// Pone en el stack un valor de 16 bits y modifica el puntero
    pub fn push_to_stack(&mut self, mmu: &mut MMU, addr: u16) {
        let addr_0: u8 = ((addr & 0xFF00) >> 8) as u8;
        let addr_1: u8 = (addr & 0x00FF) as u8;

        mmu.write_byte(self.sp, addr_1);
        self.sp -= 1;
        mmu.write_byte(self.sp, addr_0);
        self.sp -= 1;
    }

    /// Saca del stack un valor de 16 bits y modifica el puntero
    pub fn pop_from_stack(&mut self, mmu: &mut MMU) -> u16 {
        self.sp += 1;
        let addr_0 = mmu.read_byte(self.sp);
        self.sp += 1;
        let addr_1 = mmu.read_byte(self.sp);

        let addr_016 = (addr_0 as u16) << 8;
        let addr = addr_016 | (addr_1 as u16);
        addr
    }

    // Fin de funciones de stack

    fn do_bit_opcode(&mut self, bit: bool) {
        if bit {
            // bit = 0
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        self.set_h_flag();

        self.pc += 2;
    }

    /// Devuelve true si hay acarreo de medio byte entre bit 11 y 12 eun un u16 en suma
    fn calc_half_carry_on_u16_sum(&self, value_a: u16, value_b: u16) -> bool {
        ((value_a & 0xFFF) + (value_b & 0xFFF)) & 0x1000 == 0x1000
    }

    /// Devuelve true si hay acarreo de medio byte entre bit 11 y 12 eun un u16 en resta
    fn calc_half_carry_on_u16_sub(&self, value_a: u16, value_b: u16) -> bool {
        (value_a & 0xFFF) < (value_b & 0xFFF)
    }

    /// Devuelve true si hay acarreo de medio byte en suma
    fn calc_half_carry_on_u8_sum(&self, value_a: u8, value_b: u8) -> bool {
        ((value_a & 0xF) + (value_b & 0xF)) & 0x10 == 0x10
    }

    /// Devuelve true si hay acarreo de medio byte en resta
    fn calc_half_carry_on_u8_sub(&self, value_a: u8, value_b: u8) -> bool {
        (value_a & 0xF) < (value_b & 0xF)
    }

    fn do_inc_d16(&mut self, register_value: u16) -> u16 {
        // Probar si hay acarreo de medio byte entre bits 11 y 12
        if self.calc_half_carry_on_u16_sum(register_value, 1) {
            self.set_h_flag();
        } else {
            self.reset_h_flag();
        }

        let new_register_value = register_value.wrapping_add(1);

        self.pc += 1;
        self.t += 4;
        self.m += 1;
        // set the flags
        if new_register_value == 0 {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        new_register_value
    }

    fn do_sum(&mut self, register_value_a: u8, register_value_b: u8) -> u8 {
        // Probar si hay acarreo de medio byte
        if self.calc_half_carry_on_u8_sum(register_value_a, register_value_b) {
            self.set_h_flag();
        } else {
            self.reset_h_flag();
        }

        let new_register_value_a = register_value_a.wrapping_add(register_value_b);

        self.pc += 1;
        self.t += 4;
        self.m += 1;
        // Establece los flags
        if new_register_value_a == 0 {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        new_register_value_a
    }

    fn do_inc_n(&mut self, register_value: u8) -> u8 {
        self.do_sum(register_value, 1)
    }

    fn do_sub(&mut self, register_value_a: u8, register_value_b: u8) -> u8 {
        // Probar si hay acarreo de medio byte
        if self.calc_half_carry_on_u8_sub(register_value_a, register_value_b) {
            self.reset_h_flag(); // TODO: Creo que es al reves
        } else {
            self.set_h_flag();
        }

        let new_register_value_a = register_value_a.wrapping_sub(register_value_b);

        self.pc += 1;
        self.t += 4;
        self.m += 1;
        // set the flags
        if new_register_value_a == 0 {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        new_register_value_a
    }

    fn do_dec_n(&mut self, register_value: u8) -> u8 {
        self.do_sub(register_value, 1)
    }

    fn do_rl_n(&mut self, register_value: u8) -> u8 {
        // TODO: ERROR RL n, siendo n un registro el flag C debe ser copiado al bit 0 ver http://jgmalcolm.com/z80/advanced/shif#rl
        // TODO: Realmente es una rotación de 9 bits usando el flag C
        // TODO: El algoritmo deberia ser:
        // TODO: 1)guardamos en un temporal el valor del flag C
        // TODO: 2)El flag C toma el valor del bit 7
        // TODO: 3)mover 1 bit a la izquierda el registro
        // TODO: 4)Poner en bit 0 el valor del temporal
        let c_flag: bool = (0b1000_0000 & register_value) != 0;
        if c_flag {
            self.set_c_flag();
        } else {
            self.reset_c_flag();
        }
        // Rotación
        let new_register_value = register_value << 1;
        //maneja flags
        if new_register_value == 0 {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        self.reset_h_flag();

        self.pc += 2;
        self.t += 8;
        self.t += 2; // TODO: ERROR
        new_register_value
    }


    fn decode(&mut self, byte: u8, mmu: &MMU) -> Instruction {
        if self.debug { println!("Decodificando PC: {:#X}", self.pc); }

        // Preparar variables especiales

        // El valor inmediato de 16 bits
        let n1 = mmu.read_byte(self.pc + 1) as u16;

        let n2 = mmu.read_byte(self.pc + 2) as u16;

        // Invirtiendo posición ya que es BIG ENDIAN
        let d16: u16 = (n2 << 8) | n1;

        // En caso de prefijo CB, n1 es un OPCODE
        let cb_opcode = n1;
        // En caso de prefijo CB, n2 es n1
        let cb_n1 = n2;
        let cb_n2 = mmu.read_byte(self.pc + 3) as u16;
        // Invirtiendo posición ya que es BIG ENDIAN
        let cb_d16: u16 = (cb_n2 << 8) | cb_n1;

        match byte {
            0x01 => Instruction::LdBc(d16),
            0x11 => Instruction::LdDe(d16),
            0x21 => Instruction::LdHl(d16),
            0x31 => Instruction::LdSp(d16),

            0xE0 => Instruction::LdFf00U8a(n1 as u8),
            0xF0 => Instruction::LdAFf00U8(n1 as u8),

            0xE2 => Instruction::LdFf00Ca,

            0x18 => Instruction::Jr(n1 as i8),

            0x20 => Instruction::JrNz(n1 as i8),
            0x28 => Instruction::JrZ(n1 as i8),
            0x30 => Instruction::JrNc(n1 as i8),
            0x38 => Instruction::JrC(n1 as i8),

            0x3E => Instruction::LdA(n1 as u8),
            0x06 => Instruction::LdB(n1 as u8), // LD nn,n
            0x0E => Instruction::LdC(n1 as u8),
            0x16 => Instruction::LdD(n1 as u8),
            0x1E => Instruction::LdE(n1 as u8),
            0x26 => Instruction::LdH(n1 as u8),
            0x2E => Instruction::LdL(n1 as u8),

            0x7F => Instruction::LdAa,
            0x47 => Instruction::LdBa,
            0x4F => Instruction::LdCa,
            0x57 => Instruction::LdDa,
            0x5F => Instruction::LdEa,
            0x67 => Instruction::LdHa,
            0x6F => Instruction::LdLa,

            0x1A => Instruction::LdADe,
            0x0A => Instruction::LdABc,

            0x78 => Instruction::LdAb,
            0x79 => Instruction::LdAc,
            0x7A => Instruction::LdAd,
            0x7B => Instruction::LdAe,
            0x7C => Instruction::LdAh,
            0x7D => Instruction::LdAl,

            0x77 => Instruction::LdHlA,

            0xEA => Instruction::LdXxA(d16),

            0x32 => Instruction::LddHlA,
            0x22 => Instruction::LdiHlA,

            0xAF => Instruction::XorA,
            0xA8 => Instruction::XorB,
            0xA9 => Instruction::XorC,
            0xAA => Instruction::XorD,
            0xAB => Instruction::XorE,
            0xAC => Instruction::XorH,
            0xAD => Instruction::XorL,
            0xAE => Instruction::XorHL,

            0x3C => Instruction::IncA,
            0x04 => Instruction::IncB,
            0x0C => Instruction::IncC,
            0x14 => Instruction::IncD,
            0x1C => Instruction::IncE,
            0x24 => Instruction::IncH,
            0x2C => Instruction::IncL,

            // TODO: Posible error INC HL
            0x23 => Instruction::IncHlNoflags,
            // TODO: Posible error INC (HL)
            0x34 => Instruction::IncHl,

            0x13 => Instruction::IncDe,
            0x03 => Instruction::IncBc,
            0x33 => Instruction::IncSp,

            0x3D => Instruction::DecA,
            0x05 => Instruction::DecB,
            0x0D => Instruction::DecC,
            0x15 => Instruction::DecD,
            0x1D => Instruction::DecE,
            0x25 => Instruction::DecH,
            0x2D => Instruction::DecL,
            0x35 => Instruction::DecHl,

            0xC4 => Instruction::CallNz(d16),
            0xD4 => Instruction::CallNc(d16),
            0xCC => Instruction::CallZ(d16),
            0xDC => Instruction::Callc(d16),
            0xCD => Instruction::Call(d16),
            0xC9 => Instruction::Ret,

            0xF5 => Instruction::PushAf,
            0xC5 => Instruction::PushBc,
            0xD5 => Instruction::PushDe,
            0xE5 => Instruction::PushHl,
            0xF1 => Instruction::PopAf,
            0xC1 => Instruction::PopBc,
            0xD1 => Instruction::PopDe,
            0xE1 => Instruction::PopHl,

            0x17 => Instruction::RLA,

            0xBF => Instruction::CpA,
            0xB8 => Instruction::CpB,
            0xB9 => Instruction::CpC,
            0xBA => Instruction::CpD,
            0xBB => Instruction::CpE,
            0xBC => Instruction::CpH,
            0xBD => Instruction::CpL,
            0xBE => Instruction::CpHL,
            0xFE => Instruction::Cp(n1 as u8),

            0xCB => { // Opcode especial
                match cb_opcode {
                    0x40 => Instruction::BitbB(0b0000_0001),
                    0x41 => Instruction::BitbC(0b0000_0001),
                    0x42 => Instruction::BitbD(0b0000_0001),
                    0x43 => Instruction::BitbE(0b0000_0001),
                    0x44 => Instruction::BitbH(0b0000_0001),
                    0x45 => Instruction::BitbL(0b0000_0001),
                    0x46 => Instruction::BitbHL(0b0000_0001),
                    0x47 => Instruction::BitbA(0b0000_0001),

                    0x48 => Instruction::BitbB(0b0000_0010),
                    0x49 => Instruction::BitbC(0b0000_0010),
                    0x4A => Instruction::BitbD(0b0000_0010),
                    0x4B => Instruction::BitbE(0b0000_0010),
                    0x4C => Instruction::BitbH(0b0000_0010),
                    0x4D => Instruction::BitbL(0b0000_0010),
                    0x4E => Instruction::BitbHL(0b0000_0010),
                    0x4F => Instruction::BitbA(0b0000_0010),

                    0x50 => Instruction::BitbB(0b0000_0100),
                    0x51 => Instruction::BitbC(0b0000_0100),
                    0x52 => Instruction::BitbD(0b0000_0100),
                    0x53 => Instruction::BitbE(0b0000_0100),
                    0x54 => Instruction::BitbH(0b0000_0100),
                    0x55 => Instruction::BitbL(0b0000_0100),
                    0x56 => Instruction::BitbHL(0b0000_0100),
                    0x57 => Instruction::BitbA(0b0000_0100),

                    0x58 => Instruction::BitbB(0b0000_1000),
                    0x59 => Instruction::BitbC(0b0000_1000),
                    0x5A => Instruction::BitbD(0b0000_1000),
                    0x5B => Instruction::BitbE(0b0000_1000),
                    0x5C => Instruction::BitbH(0b0000_1000),
                    0x5D => Instruction::BitbL(0b0000_1000),
                    0x5E => Instruction::BitbHL(0b0000_1000),
                    0x5F => Instruction::BitbA(0b0000_1000),

                    0x7C => Instruction::BitbH(0b1000_0000),

                    0x17 => Instruction::RlA,
                    0x10 => Instruction::RlB,
                    0x11 => Instruction::RlC,
                    0x12 => Instruction::RlD,
                    0x13 => Instruction::RlE,
                    0x14 => Instruction::RlH,
                    0x15 => Instruction::RlL,
                    0x16 => Instruction::RlHl,

                    _ => panic!(
                        "DECODIFICACION PREFIJO CB: cb_opcode no reconocido \
                         Estado de MMU {:?}\n Opcode CB: {:#X} en PC {:#X}\n ESTADO CPU: {:?}",
                        mmu, cb_opcode, self.pc as u16, self
                    )
                }
            }

            0xEE => Instruction::Xor(n1 as u8),

            _ => panic!(
                "\nESTADO DE MMU: {:?} \nESTADO CPU: {:?}\nDECODIFICACION: byte no reconocido {:#X} en PC {:#X}",
                mmu, self, byte, self.pc,
            )
        }
    }

    /// Establece el valor de un registro
    fn set_register(&mut self, register_name: &str, register_value: u8) {
        match register_name {
            "a" => { self.a = register_value; }
            "b" => { self.b = register_value; }
            "c" => { self.c = register_value; }
            "d" => { self.d = register_value; }
            "e" => { self.e = register_value; }
            "f" => { self.f = register_value; }
            "h" => { self.h = register_value; }
            "l" => { self.l = register_value; }
            _ => {
                panic!(
                    "función set_register: Registro {:?} no conocido",
                    &register_name
                );
            }
        }
    }

    /// Devuelve el valor de un registro
    fn get_register(&self, register_name: &str) -> u8 {
        match register_name {
            "a" => self.a,
            "b" => self.b,
            "c" => self.c,
            "d" => self.d,
            "e" => self.e,
            "f" => self.f,
            "h" => self.h,
            "l" => self.l,
            _ => {
                panic!(
                    "función set_register: Registro {:?} no conocido",
                    &register_name
                );
            }
        }
    }

    /// Copia el valor de un registro llamado from a otro llamado to
    fn do_ld_reg_to_reg(&mut self, to: &str, from: &str) {
        // Pongo yo uppercase
        if self.debug { println!("LD {:?},{:?}", to.to_ascii_uppercase(), from.to_uppercase()) }
        self.set_register(to, self.get_register(from));

        self.pc += 1;
        self.t += 4;
        self.m += 1;
    }


    fn execute(&mut self, instruction: &Instruction, mmu: &mut MMU) {
        if self.debug { println!("Ejecutando PC: {:#X}", self.pc); }

        match instruction {
            Instruction::LdA(n) => {
                if self.debug {
                    println!("LD A, n: {:#X}", *n);
                }
                self.a = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdB(n) => {
                if self.debug {
                    println!("LD B, n: {:#X}", *n);
                }
                self.b = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdC(n) => {
                if self.debug {
                    println!("LD C, n: {:#X}", *n);
                }
                self.c = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdD(n) => {
                if self.debug {
                    println!("LD D, n: {:#X}", *n);
                }
                self.d = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdE(n) => {
                if self.debug {
                    println!("LD E, n: {:#X}", *n);
                }
                self.e = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdH(n) => {
                if self.debug {
                    println!("LD H, n: {:#X}", *n);
                }
                self.h = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdL(n) => {
                if self.debug {
                    println!("LDC, n: {:#X}", *n);
                }
                self.l = *n;
                self.pc += 2;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdSp(d16) => {
                if self.debug {
                    println!("LD SP, d16: {:#X}", d16);
                }
                self.sp = *d16; // LD SP,d16
                self.pc += 3;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LdBc(d16) => {
                if self.debug {
                    println!("LD BC, d16 {:#X}", d16);
                }
                self.b = ((d16 & 0xFF00) >> 8) as u8;
                self.c = (d16 & 0x00FF) as u8;
                self.pc += 3;
                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LdDe(d16) => {
                if self.debug {
                    println!("LD DE, d16 {:#X}", d16);
                }
                self.d = ((d16 & 0xFF00) >> 8) as u8;
                self.e = (d16 & 0x00FF) as u8;
                self.pc += 3;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LdHl(d16) => {
                if self.debug {
                    println!("LD HL, d16 {:#X}", d16);
                }
                self.h = ((d16 & 0xFF00) >> 8) as u8;
                self.l = (d16 & 0x00FF) as u8;
                if self.debug {
                    println!("LD HL despues, H: {:#X}, L: {:#X}", self.h, self.l);
                }
                self.pc += 3;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::LdAa => {
                self.do_ld_reg_to_reg("a", "a");
            }
            Instruction::LdBa => {
                self.do_ld_reg_to_reg("b", "a");
            }
            Instruction::LdCa => {
                self.do_ld_reg_to_reg("c", "a");
            }
            Instruction::LdDa => {
                self.do_ld_reg_to_reg("d", "a");
            }
            Instruction::LdEa => {
                self.do_ld_reg_to_reg("e", "a");
            }
            Instruction::LdHa => {
                self.do_ld_reg_to_reg("h", "a");
            }
            Instruction::LdLa => {
                self.do_ld_reg_to_reg("l", "a");
            }
            Instruction::LdADe => {
                if self.debug {
                    println!(
                        "LD A(DE) antes,     A: {:#X} D: {:#X}, E: {:#X}",
                        self.a, self.d, self.e
                    );
                }
                let d16 = (self.d as u16) << 8;
                let de: u16 = d16 | (self.e as u16);
                self.a = mmu.read_byte(d16); //TODO: ERROR es de no d16
                self.pc += 1;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
                if self.debug {
                    println!(
                        "LD A(DE) despues,   A: {:#X} D: {:#X}, E: {:#X}",
                        self.a, self.d, self.e
                    );
                }
            }

            Instruction::LdAb => {
                self.do_ld_reg_to_reg("a", "b");
            }

            Instruction::LdAc => {
                self.do_ld_reg_to_reg("a", "c");
            }

            Instruction::LdAd => {
                self.do_ld_reg_to_reg("a", "d");
            }

            Instruction::LdAe => {
                self.do_ld_reg_to_reg("a", "e");
            }

            Instruction::LdAh => {
                self.do_ld_reg_to_reg("a", "h");
            }

            Instruction::LdAl => {
                self.do_ld_reg_to_reg("a", "l");
            }

            Instruction::LdHlA => {
                if self.debug {
                    println!(
                        "LD (HL),A antes,   A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
                let h16 = (self.h as u16) << 8;
                let hl: u16 = h16 | (self.l as u16);
                mmu.write_byte(hl, self.a);

                self.pc += 1;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente

                if self.debug {
                    println!(
                        "LD (HL),A despues, A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
            }

            Instruction::LdXxA(d16) => {
                if self.debug { println!("LD (XX),A     XX: {:#X}", d16); }

                mmu.write_byte(*d16, self.a);

                self.pc += 3;

                self.t += 16;
                self.m += 4; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LddHlA => {
                if self.debug {
                    println!(
                        "LD (HL-),A antes,   A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
                let h16 = (self.h as u16) << 8;
                let mut hl: u16 = h16 | (self.l as u16);
                mmu.write_byte(hl, self.a);

                hl = hl.wrapping_sub(1);

                self.h = ((hl & 0xFF00) >> 8) as u8;
                self.l = (hl & 0x00FF) as u8;

                self.pc += 1;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente

                if self.debug {
                    println!(
                        "LD (HL-),A despues, A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
            }

            Instruction::LdiHlA => {
                if self.debug {
                    println!(
                        "LD (HL+),A antes,   A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
                let h16 = (self.h as u16) << 8;
                let mut hl: u16 = h16 | (self.l as u16);
                mmu.write_byte(hl, self.a);

                hl = hl.wrapping_add(1);

                self.h = ((hl & 0xFF00) >> 8) as u8;
                self.l = (hl & 0x00FF) as u8;

                self.pc += 1;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente

                if self.debug {
                    println!(
                        "LD (HL+),A despues, A: {:#X} H: {:#X}, L: {:#X}",
                        self.a, self.h, self.l
                    );
                }
            }

            Instruction::LdFf00U8a(n) => {
                if self.debug {
                    println!("LD ($FF00+u8),A     n:u8 {:#X}", n);
                }
                let addr: u16 = 0xFF00 + *n as u16;
                mmu.write_byte(addr, self.a);

                self.pc += 2;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LdAFf00U8(n) => {
                if self.debug {
                    println!("LD A,($FF00+u8)     n:u8 {:#X}", n);
                }
                let addr: u16 = 0xFF00 + *n as u16;
                self.a = mmu.read_byte(addr);

                self.pc += 2;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::LdFf00Ca => {
                if self.debug {
                    println!("LD ($FF00+C),A     C: {:#X}", self.c);
                }
                let addr: u16 = 0xFF00 + self.c as u16;
                mmu.write_byte(addr, self.a);

                self.pc += 1;

                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::XorA => {
                if self.debug {
                    println!("XorA, antes   A: {:#X} Z: {:?}", self.a, self.get_z_flag());
                }
                self.a ^= self.a;
                if self.a == 0 {
                    self.set_z_flag();
                }
                if self.debug {
                    println!("XorA, despues A: {:#X} Z: {:?}", self.a, self.get_z_flag());
                }
                self.pc += 1;
                self.t += 4;
                self.m += 1; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbA(bit_mask) => {
                if self.debug {
                    println!("Bit b,A   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.a & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbB(bit_mask) => {
                if self.debug {
                    println!("Bit b,B   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.b & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbC(bit_mask) => {
                if self.debug {
                    println!("Bit b,C   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.c & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::BitbD(bit_mask) => {
                if self.debug {
                    println!("Bit b,D   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.d & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbE(bit_mask) => {
                if self.debug {
                    println!("Bit b,E   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.e & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbH(bit_mask) => {
                if self.debug {
                    println!("Bit b,H   b: {:b}", bit_mask);
                }
                if self.debug {
                    println!(
                        "BIT n,H - antes,   b: {:b}, H: {:#X} Z: {:?}, N: {:?}, H(bit): {:?}, C: {:?}",
                        bit_mask,
                        self.h,
                        self.get_z_flag(),
                        self.get_n_flag(),
                        self.get_h_flag(),
                        self.get_c_flag()
                    );
                }
                let bit_test: u8 = self.h & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                if self.debug {
                    println!(
                        "BIT n,H - despues, b: {:b}, H: {:#X} Z: {:?}, N: {:?}, H(bit): {:?}, C: {:?}",
                        bit_mask,
                        self.h,
                        self.get_z_flag(),
                        self.get_n_flag(),
                        self.get_h_flag(),
                        self.get_c_flag()
                    );
                }
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::BitbL(bit_mask) => {
                if self.debug {
                    println!("Bit b,L   b: {:b}", bit_mask);
                }
                let bit_test: u8 = self.l & *bit_mask;
                self.do_bit_opcode(*bit_mask != bit_test);
                self.t += 8;
                self.m += 2; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::JrNz(n) => {
                if self.debug {
                    println!("JR NZ n: {:#X}", n);
                }
                self.pc = self.pc + 2;
                if self.debug {
                    println!(
                        "JR NZ, n - antes,   n: {:#X}, Z: {:?}, PC: {:#X}",
                        n,
                        self.get_z_flag(),
                        self.pc
                    );
                }

                if !self.get_z_flag() {
                    self.pc = self.pc.wrapping_add(*n as u16);
                    self.t += 12;
                    self.m += 3; //TODO: Creo que teniendo los t states es suficiente
                } else {
                    self.t += 8;
                    self.m += 2; //TODO: Creo que teniendo los t states es suficiente
                }
                if self.debug {
                    println!(
                        "JR NZ, n - despues, n: {:#X}, Z: {:?}, PC: {:#X}",
                        n,
                        self.get_z_flag(),
                        self.pc
                    );
                }
            }

            Instruction::JrZ(n) => {
                if self.debug {
                    println!("JR Z n: {:#X}", n);
                }
                self.pc = self.pc + 2;
                if self.get_z_flag() {
                    self.pc = self.pc.wrapping_add(*n as u16);
                    self.t += 12;
                    self.m += 3; //TODO: Creo que teniendo los t states es suficiente
                } else {
                    self.t += 8;
                    self.m += 2; //TODO: Creo que teniendo los t states es suficiente
                }
            }

            Instruction::JrNc(n) => {
                if self.debug {
                    println!("JR NC n: {:#X}", n);
                }
                self.pc = self.pc + 2;
                if !self.get_c_flag() {
                    self.pc = self.pc.wrapping_add(*n as u16);
                    self.t += 12;
                    self.m += 3; //TODO: Creo que teniendo los t states es suficiente
                } else {
                    self.t += 8;
                    self.m += 2; //TODO: Creo que teniendo los t states es suficiente
                }
            }

            Instruction::JrC(n) => {
                if self.debug {
                    println!("JR C n: {:#X}", n);
                }
                self.pc = self.pc + 2;

                if self.get_c_flag() {
                    self.pc = self.pc.wrapping_add(*n as u16);
                    self.t += 12;
                    self.m += 3; //TODO: Creo que teniendo los t states es suficiente
                } else {
                    self.t += 8;
                    self.m += 2; //TODO: Creo que teniendo los t states es suficiente
                }
            }

            Instruction::Jr(n) => {
                if self.debug { println!("JR n: {:#X}", n); }
                self.pc = self.pc + 2; //TODO: ERROR debe cambiar el PC


                //self.pc = self.pc.wrapping_add(*n as u16);
                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::IncA => {
                if self.debug { println!("INC A"); }
                self.a = self.do_inc_n(self.a);
            }

            Instruction::IncB => {
                if self.debug { println!("INC B"); }
                self.b = self.do_inc_n(self.b);
            }

            Instruction::IncC => {
                if self.debug { println!("INC C"); }
                self.c = self.do_inc_n(self.c);
            }
            Instruction::IncD => {
                if self.debug { println!("INC D"); }
                self.d = self.do_inc_n(self.d);
            }

            Instruction::IncE => {
                if self.debug { println!("INC E"); }
                self.e = self.do_inc_n(self.e);
            }
            Instruction::IncH => {
                if self.debug { println!("INC H"); }
                self.h = self.do_inc_n(self.h);
            }

            Instruction::IncL => {
                if self.debug { println!("INC L"); }
                self.l = self.do_inc_n(self.l);
            }

            Instruction::IncHl => { // TODO: ERROR direccion indirecta
                if self.debug { println!("INC (HL)"); }
                let h16 = (self.h as u16) << 8;
                let mut hl: u16 = h16 | (self.l as u16);
                hl = self.do_inc_d16(hl);
                self.h = ((hl & 0xFF00) >> 8) as u8;
                self.l = (hl & 0x00FF) as u8;
            }

            Instruction::IncHlNoflags => {
                if self.debug { println!("INC HL"); }
                let h16 = (self.h as u16) << 8;
                let mut hl: u16 = h16 | (self.l as u16);
                hl = hl.wrapping_add(1);
                self.h = ((hl & 0xFF00) >> 8) as u8;
                self.l = (hl & 0x00FF) as u8;

                self.pc += 1;
                self.t += 8;
                self.m += 2;
            }

            Instruction::IncBc => {
                if self.debug { println!("INC BC"); }
                let b16 = (self.b as u16) << 8;
                let mut bc: u16 = b16 | (self.c as u16);
                bc = bc.wrapping_add(1);
                self.b = ((bc & 0xFF00) >> 8) as u8;
                self.c = (bc & 0x00FF) as u8;

                self.pc += 1;
                self.t += 8;
                self.m += 2;
            }

            Instruction::IncDe => {
                if self.debug { println!("INC DE"); }
                let d16 = (self.d as u16) << 8;
                let mut de: u16 = d16 | (self.e as u16);
                de = de.wrapping_add(1);
                self.d = ((de & 0xFF00) >> 8) as u8;
                self.e = (de & 0x00FF) as u8;

                self.pc += 1;
                self.t += 8;
                self.m += 2;
            }

            Instruction::DecA => {
                if self.debug { println!("DEC A"); }
                self.a = self.do_dec_n(self.a);
            }
            Instruction::DecB => {
                if self.debug { println!("DEC B"); }
                self.b = self.do_dec_n(self.b);
            }
            Instruction::DecC => {
                if self.debug { println!("DEC C"); }
                self.c = self.do_dec_n(self.c);
            }
            Instruction::DecD => {
                if self.debug { println!("DEC D"); }
                self.d = self.do_dec_n(self.d);
            }
            Instruction::DecE => {
                if self.debug { println!("DEC E"); }
                self.e = self.do_dec_n(self.e);
            }
            Instruction::DecH => {
                if self.debug { println!("DEC H"); }
                self.h = self.do_dec_n(self.h);
            }
            Instruction::DecL => {
                if self.debug { println!("DEC L"); }
                self.l = self.do_dec_n(self.l);
            }
            Instruction::Call(d16) => {
                if self.debug { println!("CALL d16: {:#X}", d16); }
                self.pc += 3;
                self.push_to_stack(mmu, self.pc);
                self.pc = *d16;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::Ret => {
                if self.debug { println!("RET"); }
                self.pc = self.pop_from_stack(mmu);


                self.t += 16;
                self.m += 4; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::PushAf => {
                if self.debug { println!("PUSH AF"); }
                let a16 = (self.a as u16) << 8;
                let af: u16 = a16 | (self.f as u16);
                self.push_to_stack(mmu, af);
                self.pc += 1;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::PushBc => {
                if self.debug { println!("PUSH BC  B:{:#X}  c:{:#X}", self.b, self.c); }
                let b16 = (self.b as u16) << 8;
                let bc: u16 = b16 | (self.c as u16);
                self.push_to_stack(mmu, bc);
                self.pc += 1;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::PushDe => {
                if self.debug { println!("PUSH DE"); }
                let d16 = (self.d as u16) << 8;
                let de: u16 = d16 | (self.e as u16);
                self.push_to_stack(mmu, de);
                self.pc += 1;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::PushHl => {
                if self.debug { println!("PUSH HL"); }
                let h16 = (self.h as u16) << 8;
                let hl: u16 = h16 | (self.l as u16);
                self.push_to_stack(mmu, hl);
                self.pc += 1;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::PopAf => {
                if self.debug { println!("POP AF"); }
                let addr: u16 = self.pop_from_stack(mmu);
                let a = ((addr & 0xFF00) >> 8) as u8;
                self.f = (addr & 0x00FF) as u8;


                self.pc += 1;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::PopDe => {
                if self.debug { println!("POP DE"); }
                let addr: u16 = self.pop_from_stack(mmu);
                let d = ((addr & 0xFF00) >> 8) as u8;
                self.e = (addr & 0x00FF) as u8;


                self.pc += 1;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }
            Instruction::PopHl => {
                if self.debug { println!("POP HL"); }
                let addr: u16 = self.pop_from_stack(mmu);
                let h = ((addr & 0xFF00) >> 8) as u8;
                self.l = (addr & 0x00FF) as u8;


                self.pc += 1;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::PopBc => {
                if self.debug { println!("POP BC"); }
                let addr: u16 = self.pop_from_stack(mmu);
                let b = ((addr & 0xFF00) >> 8) as u8;
                self.c = (addr & 0x00FF) as u8;


                self.pc += 1;

                self.t += 12;
                self.m += 3; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::RlA => {
                if self.debug { println!("RL A"); }
                self.a = self.do_rl_n(self.a);
            }
            Instruction::RlB => {
                if self.debug { println!("RL B"); }
                self.b = self.do_rl_n(self.b);
            }
            Instruction::RlC => {
                if self.debug { println!("RL C"); }
                self.c = self.do_rl_n(self.c);
            }
            Instruction::RlD => {
                if self.debug { println!("RL D"); }
                self.d = self.do_rl_n(self.d);
            }
            Instruction::RlE => {
                if self.debug { println!("RL E"); }
                self.e = self.do_rl_n(self.e);
            }
            Instruction::RlH => {
                if self.debug { println!("RL H"); }
                self.h = self.do_rl_n(self.h);
            }
            Instruction::RlL => {
                if self.debug { println!("RL L"); }
                self.l = self.do_rl_n(self.l);
            }
            Instruction::RLA => {
                if self.debug { println!("RLA"); }
                self.a = self.do_rl_n(self.a);
                self.pc -= 1;  // Solucion poco natural, decrementar despues de incrementar 2
                self.t -= 4;   // Solucion poco natural, decrementar despues de incrementar
                self.m -= 1;   // Solucion poco natural, decrementar despues de incrementar
            }
            Instruction::CpA => {
                if self.debug { println!("CP A"); }
                let _ = self.do_sub(self.a, self.a);
            }
            Instruction::CpB => {
                if self.debug { println!("CP B"); }
                let _ = self.do_sub(self.a, self.b);
            }
            Instruction::CpC => {
                if self.debug { println!("CP C"); }
                let _ = self.do_sub(self.a, self.c);
            }
            Instruction::CpD => {
                if self.debug { println!("CP D"); }
                let _ = self.do_sub(self.a, self.d);
            }
            Instruction::CpE => {
                if self.debug { println!("CP E"); }
                let _ = self.do_sub(self.a, self.e);
            }
            Instruction::CpH => {
                if self.debug { println!("CP H"); }
                let _ = self.do_sub(self.a, self.h);
            }
            Instruction::CpL => {
                if self.debug { println!("CP L"); }
                let _ = self.do_sub(self.a, self.l);
            }
            Instruction::CpHL => {
                if self.debug {
                    println!("CP (HL)");
                }
                let _ = self.do_sub(self.a, self.l);

                let h16 = (self.h as u16) << 8;
                let hl: u16 = h16 | (self.l as u16);
                let _ = self.do_sub(self.a, mmu.read_byte(hl));
                self.pc += 1;

                self.t += 24;
                self.m += 6; //TODO: Creo que teniendo los t states es suficiente
            }

            Instruction::Cp(n) => {
                if self.debug { println!("CP n:  {:#X}", n); }
                println!("MMU State: {:?}", mmu);
                println!("Register A: {:b}", self.a);
                // TODO: ERROR se debe comparar con el valor, no con lo que hay en la direccion
                let _ = self.do_sub(self.a, mmu.read_byte(*n as u16));

                self.pc += 1; // (Reincrementos)
                self.t += 4;
                self.m += 1;
            }

            _ => panic!(
                "\nESTADO DE MMU: {:?}\nESTADO CPU: {:?}\nEjecución: Instrucción no reconocida {:?} en PC {:#X}",
                mmu, self, instruction, self.pc,
            ),
        }
    }

    pub fn run_instruction(&mut self, mmu: &mut MMU) {
        // Obtener instrucción:
        let byte = mmu.read_byte(self.pc);
        // Decodificar instrucción
        let instruction = self.decode(byte, mmu);
        // Ejecutar instrucción
        self.execute(&instruction, mmu);
    }
}