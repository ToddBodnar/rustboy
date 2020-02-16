pub struct Memory {
    pub ram: Vec<u8>
}

impl Memory {
    pub fn set(&mut self, loc: u16, val: u8) {
        self.ram[loc as usize] = val;
    }

    pub fn get(&self, loc: u16) -> u8 {
        return self.ram[loc as usize];
    }

    pub fn push_stack(&mut self, reg: &mut Registers, val: u16) {
        let low_byte = (val & 0xFF) as u8;
        let high_byte = (val / 0x100) as u8;

        let sp = reg.get_register(&RegisterNames::SP);

        self.set(sp, low_byte);
        self.set(sp - 1, high_byte);

        reg.set_register(&RegisterNames::SP, sp - 2);
    }

    pub fn pop_stack(&self, reg: &mut Registers) -> u16 {
        let sp = reg.get_register(&RegisterNames::SP);

        let high_byte = self.get(sp + 1) as u16;
        let low_byte = self.get(sp + 2) as u16;

        reg.set_register(&RegisterNames::SP, sp + 2);

        return high_byte * 0x100 + low_byte;
    }
}

pub struct Engine {
    pub memory: Memory,
    pub registers: Registers,
    pub enable_interrupt: bool
}

impl Engine {
    pub fn run(&mut self){

        for _ in 1..1000000000 {
            let wait_time = self.execute_next_instruction();
        }
    }

    fn get_d8(&self, start: u16) -> u8 {
        return self.memory.get(start);
    }

    fn get_d16(&self, start: u16) -> u16 {
        return ((self.memory.get(start+1) as u16) << 8)
              + (self.memory.get(start) as u16);
    }

    fn get_r8(&self, start: u16) -> i8 {
        return self.memory.get(start) as i8;
    }

    fn get_a16(&self, start: u16) -> u16 {
        return ((self.memory.get(start+1) as u16) << 8)
              + (self.memory.get(start) as u16);
    }

    fn execute_next_instruction(&mut self) -> u8 {
        let first_byte = self.memory.get(self.registers.pc);
        let first_nibble = first_byte & 0x0F;
        let second_nibble = first_byte >> 4;

        let resolved_second_register = match second_nibble {
            0x0 => RegisterNames::B,
            0x1 => RegisterNames::C,
            0x2 => RegisterNames::D,
            0x3 => RegisterNames::E,
            0x4 => RegisterNames::H,
            0x5 => RegisterNames::L,
            0x6 => RegisterNames::HL,
            0x7 => RegisterNames::A,
            0x8 => RegisterNames::B,
            0x9 => RegisterNames::C,
            0xA => RegisterNames::D,
            0xB => RegisterNames::E,
            0xC => RegisterNames::H,
            0xD => RegisterNames::L,
            0xE => RegisterNames::HL,
            0xF => RegisterNames::A,
            _ => {
                panic!("Bad nibble size {:?} from {:?}", second_nibble, first_byte);
            }
        };

        //println!("{:?} = {:?}, {:?}", first_byte, second_nibble, first_nibble);
        match first_byte {
            0x00 => {
                // no op
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0x01 | 0x11 | 0x21 | 0x31 => { //2 byte load
                let resolved_register = match first_byte {
                    0x01 => RegisterNames::BC,
                    0x11 => RegisterNames::DE,
                    0x21 => RegisterNames::HL,
                    0x31 => RegisterNames::SP,
                    _ => panic!("how did we get here?")
                };

                let d16 = self.get_d16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.registers.set_register(&resolved_register, d16);

                self.registers.incr_pc(3);
                return 12 as u8;
            },

            0x04 => self.inc(RegisterNames::B),
            0x05 => self.dec(RegisterNames::B),
            0x0C => self.inc(RegisterNames::C),
            0x0D => self.dec(RegisterNames::C),

            0x14 => self.inc(RegisterNames::D),
            0x15 => self.dec(RegisterNames::D),
            0x1C => self.inc(RegisterNames::E),
            0x1D => self.dec(RegisterNames::E),

            0x24 => self.inc(RegisterNames::H),
            0x25 => self.dec(RegisterNames::H),
            0x2C => self.inc(RegisterNames::L),
            0x2D => self.dec(RegisterNames::L),

            0x34 => self.inc(RegisterNames::HL),
            0x35 => self.dec(RegisterNames::HL),
            0x3C => self.inc(RegisterNames::A),
            0x3D => self.dec(RegisterNames::A),

            0x02 | 0x12 | 0x22 | 0x32 | 0x0A | 0x1A | 0x2A | 0x3A => {
                let memory_loc = match (first_byte & 0xF0) {
                    0x00 => self.registers.get_register(&RegisterNames::BC),
                    0x10 => self.registers.get_register(&RegisterNames::DE),
                    0x20 | 0x30 => self.registers.get_register(&RegisterNames::HL),
                    _ => panic!("how did we get here?")
                };

                if first_byte % 16 == 2 {
                    self.memory.set(memory_loc, self.registers.get_register(&RegisterNames::A) as u8);
                } else {
                    let memory_val = self.memory.get(memory_loc);
                    self.registers.set_register(&RegisterNames::A, memory_val as u16);
                }

                 match (first_byte & 0xF0) {
                     0x20 => self.registers.change_register(RegisterNames::HL, 1),
                     0x30 => self.registers.change_register(RegisterNames::HL, -1),
                     _ => {}
                 };

                self.registers.incr_pc(1);
                return 1 as u8;
            },
            0x06 | 0x16 | 0x26 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let target_register = match first_byte {
                    0x06 => RegisterNames::B,
                    0x16 => RegisterNames::D,
                    0x26 => RegisterNames::H,
                    0x0E => RegisterNames::C,
                    0x1E => RegisterNames::E,
                    0x2E => RegisterNames::L,
                    0x3E => RegisterNames::A,
                    _ => panic!("how did we get here?")
                };
                let source_value
                      = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1);
                self.registers.set_register(&RegisterNames::B, source_value as u16);
                self.registers.incr_pc(2);
                return 8 as u8;
            },

            // jump relative
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                let decide_to_jump = match first_byte {
                    0x18 => true,
                    0x20 => !self.registers.is_zero_flag(),//nz
                    0x28 => self.registers.is_zero_flag(),//z
                    0x30 => !self.registers.is_cary_flag(),//nc
                    0x38 => self.registers.is_cary_flag(),//c
                    _ => panic!("how did we get here?")
                };

                if !decide_to_jump {
                    self.registers.incr_pc(2);
                    return 8 as u8;
                } else {
                    let to_increase = self.get_r8(self.registers.get_register(&RegisterNames::PC) + 1);
                    self.registers.incr_pc(to_increase as i32);
                    return 12 as u8;
                }
            }

            // load block
            0x40..=0x7F => {
                if first_byte == 0x76 {
                    println!("Halt!!!");
                    self.registers.incr_pc(1);
                    return 4 as u8;
                } else {
                    let resolved_first_register = match first_byte {
                        0x40..=0x47 => RegisterNames::B,
                        0x48..=0x4F => RegisterNames::C,
                        0x50..=0x57 => RegisterNames::D,
                        0x58..=0x5F => RegisterNames::E,
                        0x60..=0x67 => RegisterNames::H,
                        0x68..=0x6F => RegisterNames::L,
                        0x70..=0x77 => RegisterNames::HL,
                        0x78..=0x7F => RegisterNames::A,
                        _ => {
                            panic!("first byte was modified");
                        }
                    };


                    let initial = self.registers.get_register(&resolved_second_register);
                    self.registers.set_register(&resolved_first_register, initial);
                    self.registers.incr_pc(1);

                    if resolved_first_register == RegisterNames::HL || resolved_second_register == RegisterNames::HL {
                        return 8 as u8;
                    }
                    return 4 as u8;
                }
            },
            0x80..=0x87 => {
                self.math_to_a(MathNames::ADD, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0x88..=0x8F => {
                self.math_to_a(MathNames::ADDC, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0x90..=0x97 => {
                self.math_to_a(MathNames::SUB, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0x98..=0x9F => {
                self.math_to_a(MathNames::SUBC, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0xA0..=0xA7 => {
                self.math_to_a(MathNames::AND, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0xA8..=0xAF => {
                self.math_to_a(MathNames::XOR, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0xB0..=0xB7 => {
                self.math_to_a(MathNames::OR, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0xB8..=0xBF => {
                self.math_to_a(MathNames::CP, self.registers.get_register(&resolved_second_register));
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            0xC2 => { // jump nz
                if self.registers.is_zero_flag() {
                    self.registers.incr_pc(3);
                    return 12 as u8;
                } else {
                    let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);
                    self.registers.set_register(&RegisterNames::PC, target);
                    return 16 as u8;
                }
            },

            0xC3 => { // jump
                //println!("jump {:?} , {:?}", self.rom[(self.registers.pc + 1) as usize], self.rom[(self.registers.pc + 2) as usize]);
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);
                //println!("target is {:?}", target);
                self.registers.set_register(&RegisterNames::PC, target);
                return 16 as u8;
            },

            // returns
            0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 => {
                let decide_to_jump = match first_byte {
                    0xC9 => true,
                    0xC0 => !self.registers.is_zero_flag(),//nz
                    0xC8 => self.registers.is_zero_flag(),//z
                    0xD0 => !self.registers.is_cary_flag(),//nc
                    0xD8 => self.registers.is_cary_flag(),//c
                    _ => panic!("how did we get here?")
                };

                self.registers.incr_pc(1);
                if !decide_to_jump {
                    return 8 as u8;
                } else {
                    let new_pc = self.memory.pop_stack(&mut self.registers);
                    self.registers.set_register(&RegisterNames::PC, new_pc);

                    if first_byte == 0xC9 {
                        return 16 as u8;
                    } else {
                        return 20 as u8;
                    }
                }
            },

            0xC1 | 0xC5 | 0xD1 | 0xD5 | 0xE1 | 0xE5 | 0xF1 | 0xF5 => {
                let reg = match first_byte {
                    0xC1 | 0xC5 => RegisterNames::BC,
                    0xD1 | 0xD5 => RegisterNames::DE,
                    0xE1 | 0xE5 => RegisterNames::HL,
                    0xF1 | 0xF5 => RegisterNames::AF,
                    _ => panic!("how did we get here")
                };

                self.registers.incr_pc(1);

                if first_byte % 0xF0 == 1 {
                    let new_reg = self.memory.pop_stack(&mut self.registers);
                    self.registers.set_register(&reg, new_reg);
                    return 12 as u8;
                } else {
                    let register_val = self.registers.get_register(&reg);
                    self.memory.push_stack(&mut self.registers, register_val);
                    return 16 as u8;
                }
            },

            // call
            0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC => {
                let decide_to_jump = match first_byte {
                    0xCD => true,
                    0xC4 => !self.registers.is_zero_flag(),//nz
                    0xCC => self.registers.is_zero_flag(),//z
                    0xD4 => !self.registers.is_cary_flag(),//nc
                    0xDC => self.registers.is_cary_flag(),//c
                    _ => panic!("how did we get here?")
                };

                let a16 = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.registers.incr_pc(3);
                if !decide_to_jump {
                    return 12 as u8;
                } else {
                    let old_pc = self.registers.get_register(&RegisterNames::PC);

                    self.memory.push_stack(&mut self.registers, old_pc);

                    self.registers.set_register(&RegisterNames::PC, a16);

                    return 24 as u8;
                }
            },

            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => { //d8 math
                let math_type = match first_byte {
                    0xC6 => MathNames::ADD,
                    0xCE => MathNames::ADDC,
                    0xD6 => MathNames::SUB,
                    0xDE => MathNames::SUBC,
                    0xE6 => MathNames::AND,
                    0xEE => MathNames::XOR,
                    0xF6 => MathNames::OR,
                    0xFE => MathNames::CP,
                    _ => panic!("how did we get here")
                };

                self.math_to_a(math_type, self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1) as u16);
                self.registers.incr_pc(2);
                return 8 as u8;
            },

            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let old_pc = self.registers.get_register(&RegisterNames::PC) + 1;
                self.memory.push_stack(&mut self.registers, old_pc);

                let new_pc = match first_byte {
                    0xC7 => 0x0000,
                    0xCF => 0x0008,
                    0xD7 => 0x0010,
                    0xDF => 0x0018,
                    0xE7 => 0x0020,
                    0xEF => 0x0028,
                    0xF7 => 0x0030,
                    0xFF => 0x0038,
                    _ => panic!("how did we get here")
                };

                self.registers.set_register(&RegisterNames::SP, new_pc);

                return 16 as u8;
            },

            0xE0 => { // load a into a8
                let target = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1) as u16 + 0xFF00;

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(2);

                return 12 as u8;
            },

            0xEA => { // load a into a16
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(3);

                return 16 as u8;
            },

            //interrupts
            0xF3 => {
                self.enable_interrupt = false;
                self.registers.incr_pc(1);
                return 4 as u8;
            },
            0xFB => {
                self.enable_interrupt = true;
                self.registers.incr_pc(1);
                return 4 as u8;
            },

            _ => {
                println!("Don't understand instr {:?}", self.memory.get(self.registers.get_register(&RegisterNames::PC)));
                self.registers.incr_pc(1);
                return 0 as u8;
            }
        }
    }

    fn inc(&mut self, register: RegisterNames) -> u8 {
        self.registers.change_register(register, 1);
        self.registers.incr_pc(1);
        return 4;
    }

    fn dec(&mut self, register: RegisterNames) -> u8 {
        self.registers.change_register(register, -1);
        self.registers.incr_pc(1);
        return 4;
    }

    fn math_to_a(&mut self, math_type: MathNames, other_value: u16) {
        let initial_a = self.registers.get_register(&RegisterNames::A);
        let mut result = 0;

        match math_type {
            ADD => {
                result = initial_a + other_value;

                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_a + other_value) as u8 == 0,
                                false,
                                // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                (initial_a + other_value) & 0x10 == 0x10,
                                initial_a + other_value > 0xF
                ));
            },
            ADDC => {
                result = initial_a + other_value;
                if self.registers.is_cary_flag() {
                    result += 1;
                }

                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_a + other_value) as u8 == 0,
                                false,
                                // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                (initial_a + other_value) & 0x10 == 0x10,
                                initial_a + other_value > 0xF
                ));
            },

            SUB => {
                result = initial_a - other_value;

                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_a + other_value) as u8 == 0,
                                false,
                                // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                (initial_a - other_value) & 0x10 == 0x10,
                                initial_a - other_value > 0xF
                ));
            },
            SUBC => {
                result = initial_a - other_value;
                if self.registers.is_cary_flag() {
                    result += 1;
                }

                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_a + other_value) as u8 == 0,
                                false,
                                // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                (initial_a + other_value) & 0x10 == 0x10,
                                initial_a + other_value > 0xF
                ));
            },

            AND => {
                result = initial_a & other_value;


                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags(result == 0,
                                false,
                                true,
                                false
                ));
            },

            OR => {
                result = initial_a | other_value;


                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags(result == 0,
                                false,
                                false,
                                false
                ));
            },

            XOR => {
                result = initial_a ^ other_value;


                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags(result == 0,
                                false,
                                true,
                                false
                ));
            },

            CP => {
                result = initial_a;


                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_a - other_value) as u8 == 0,
                                false,
                                (initial_a - other_value) & 0x10 == 0x10,
                                initial_a - other_value > 0xF
                ));
            }
        };

        self.registers.set_register(&RegisterNames::A, result);
    }
}

#[derive(PartialEq, Eq)]
enum RegisterNames {
    A,B,C,D,E,F,H,L,
    AF,BC,DE,HL,
    PC,SP
}

#[derive(PartialEq, Eq)]
enum MathNames {
    ADD, ADDC,
    SUB, SUBC,
    AND, XOR, OR, CP
}

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

impl Registers{
    fn make_flags(zero: bool, subtract: bool, half_cary: bool, cary: bool) -> u16 {
        return 0 as u16; //todo
    }

    fn incr_pc(& mut self, amount: i32){
        self.pc = (self.pc as i32 + amount) as u16;
    }

    fn change_register(&mut self, register: RegisterNames, delta: i32){
        let initial = self.get_register(&register);
        self.set_register(&register, (initial as i32 + delta) as u16 )
    }

    fn is_zero_flag(&mut self) -> bool {
        return self.f & 128 != 0;
    }


    fn is_cary_flag(&mut self) -> bool {
        return self.f & 16 != 0;
    }

    fn set_register(&mut self, register: &RegisterNames, value: u16){
        match register {
            RegisterNames::A => self.a = value as u8,
            RegisterNames::B => self.b = value as u8,
            RegisterNames::C => self.c = value as u8,
            RegisterNames::D => self.d = value as u8,
            RegisterNames::E => self.e = value as u8,
            RegisterNames::F => self.f = value as u8,

            RegisterNames::H => self.h = value as u8,
            RegisterNames::L => self.l = value as u8,


            RegisterNames::PC => self.pc = value,
            RegisterNames::SP => self.sp = value,

            RegisterNames::AF => {
                self.a = (value >> 8) as u8;
                self.f = (value & 0xFF) as u8;
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

    fn get_register(&self, register: &RegisterNames) -> u16 {
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
