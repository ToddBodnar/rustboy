use std::fmt;

#[derive(Debug)]
pub struct Registers {
    pub pc: u16, //todo: how to initilaze this without it being public
    pub sp: u16,

    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,

    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn make_registers() -> Registers {
        return Registers { //todo: understanding of these default values
            pc: 0x100,
            sp: 0xFFFE,

            a: 0x11,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x08,
            f: 0x80,

            h: 0x00,
            l: 0x7c
        };
    }

    pub fn make_flags(zero: bool, subtract: bool, half_cary: bool, cary: bool) -> u16 {
        let mut res = 0;
        if zero {
            res += 128;
        }

        if subtract {
            res += 64
        }

        if half_cary {
            res += 32;
        }

        if cary {
            res += 16;
        }

        return res as u16; //todo
    }

    pub fn incr_pc(& mut self, amount: i32){
        self.pc = (self.pc as i32 + amount) as u16;
    }

    pub fn change_register(&mut self, register: RegisterNames, delta: i32){
        let initial = self.get_register(&register);
        self.set_register(&register, (initial as i32 + delta) as u16 )
    }

    pub fn is_zero_flag(&mut self) -> bool {
        return self.f & 128 != 0;
    }

    pub fn set_zero_flag(&mut self, val: bool) {
        if self.is_zero_flag() == val {
            return;
        } else{
            self.f ^= 128;
        }
    }

    pub fn is_subtract_flag(&mut self) -> bool {
        return self.f & 64 != 0;
    }

    pub fn is_half_cary_flag(&mut self) -> bool {
        return self.f & 32 != 0;
    }

    pub fn is_cary_flag(&mut self) -> bool {
        return self.f & 16 != 0;
    }

    pub fn set_flags(&mut self, zero: bool, subtract: bool, half_cary: bool, cary: bool) {
        self.set_register(&RegisterNames::F,
            Registers::make_flags(zero, subtract, half_cary, cary));
    }

    pub fn set_register(&mut self, register: &RegisterNames, value: u16){
        match register {
            RegisterNames::A => self.a = value as u8,
            RegisterNames::B => self.b = value as u8,
            RegisterNames::C => self.c = value as u8,
            RegisterNames::D => self.d = value as u8,
            RegisterNames::E => self.e = value as u8,
            RegisterNames::F => self.f = (value & 0xF0) as u8,

            RegisterNames::H => self.h = value as u8,
            RegisterNames::L => self.l = value as u8,


            RegisterNames::PC => self.pc = value,
            RegisterNames::SP => self.sp = value,

            RegisterNames::AF => {
                self.a = (value >> 8) as u8;
                self.f = (value & 0xF0) as u8;
            },

            RegisterNames::BC => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0xFF) as u8;
            },

            RegisterNames::DE => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0xFF) as u8;
            },

            RegisterNames::HL => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0xFF) as u8;
            },

            _ => {
                println!("Setting undefined register")
            } //TODO
        }
    }

    pub fn get_register(&self, register: &RegisterNames) -> u16 {
        match register {
            RegisterNames::A => return self.a as u16,
            RegisterNames::B => return self.b as u16,
            RegisterNames::C => return self.c as u16,
            RegisterNames::D => return self.d as u16,
            RegisterNames::E => return self.e as u16,
            RegisterNames::F => return self.f as u16,

            RegisterNames::H => return self.h as u16,
            RegisterNames::L => return self.l as u16,

            RegisterNames::AF => return self.f as u16 + ((self.a as u16) << 8),
            RegisterNames::BC => return self.c as u16 + ((self.b as u16) << 8),
            RegisterNames::DE => return self.e as u16 + ((self.d as u16) << 8),
            RegisterNames::HL => return self.l as u16 + ((self.h as u16) << 8),

            RegisterNames::SP => return self.sp,
            RegisterNames::PC => return self.pc,

            _ => {
                println!("TODO: IMPLEMENT REMAINING REGISTERS");
                return 0;
            }
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // mgba's main output
        //return write!(f, "A: {:x?}\tF: {:x?}\nB: {:x?}\tC: {:x?}\nD: {:x?}\tE: {:x?}\nH: {:x?}\tL: {:x?}\nPC: {:x?} SP: {:x?}", self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp);
        // mgba's minified output
        return write!(f, "A: {:02X?} F: {:02X?} B: {:02X?} C: {:02X?} D: {:02X?} E: {:02X?} H: {:02X?} L: {:02X?} SP: {:04X?} PC: 00:{:04X?}", self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc);
    }
}

#[derive(PartialEq, Eq)]
#[derive(Debug)]
pub enum RegisterNames {
    A,B,C,D,E,F,H,L,
    AF,BC,DE,HL,
    PC,SP
}

#[cfg(test)]
mod tests {
    use crate::engine::registers::Registers;
    use crate::engine::registers::RegisterNames;
    #[test]
    fn test_flags() {
        let mut reg = Registers::make_registers();

        reg.set_flags(true, false, false, false);
        assert!(!reg.is_cary_flag());
        assert!(reg.is_zero_flag());

        reg.set_flags(false, false, false, true);
        assert!(reg.is_cary_flag());
        assert!(!reg.is_zero_flag());

        reg.set_flags(true, false, true, true);
        assert!(reg.is_cary_flag());
        assert!(reg.is_zero_flag());

        println!("{:?}", reg);
    }

    #[test]
    fn test_af(){
        let mut reg = Registers::make_registers();

        reg.set_register(&RegisterNames::AF, 8512);

        assert_eq!(8512, reg.get_register(&RegisterNames::AF));
        assert_eq!(33, reg.get_register(&RegisterNames::A));
        assert_eq!(64, reg.get_register(&RegisterNames::F));


        reg.set_register(&RegisterNames::AF, 0);

        assert_eq!(0, reg.get_register(&RegisterNames::AF));
        assert_eq!(0, reg.get_register(&RegisterNames::A));
        assert_eq!(0, reg.get_register(&RegisterNames::F));
    }

    #[test]
    fn test_bc(){
        let mut reg = Registers::make_registers();

        reg.set_register(&RegisterNames::BC, 8512);

        assert_eq!(8512, reg.get_register(&RegisterNames::BC));
        assert_eq!(64, reg.get_register(&RegisterNames::C));


        reg.set_register(&RegisterNames::BC, 0);

        assert_eq!(0, reg.get_register(&RegisterNames::BC));
        assert_eq!(0, reg.get_register(&RegisterNames::C));
    }

    #[test]
    fn test_de(){
        let mut reg = Registers::make_registers();

        reg.set_register(&RegisterNames::DE, 8512);

        assert_eq!(8512, reg.get_register(&RegisterNames::DE));
        assert_eq!(64, reg.get_register(&RegisterNames::E));


        reg.set_register(&RegisterNames::DE, 0);

        assert_eq!(0, reg.get_register(&RegisterNames::DE));
        assert_eq!(0, reg.get_register(&RegisterNames::E));
    }

    #[test]
    fn test_hl(){
        let mut reg = Registers::make_registers();

        reg.set_register(&RegisterNames::HL, 8512);

        assert_eq!(8512, reg.get_register(&RegisterNames::HL));
        assert_eq!(64, reg.get_register(&RegisterNames::L));


        reg.set_register(&RegisterNames::HL, 0);

        assert_eq!(0, reg.get_register(&RegisterNames::HL));
        assert_eq!(0, reg.get_register(&RegisterNames::L));
    }
}
