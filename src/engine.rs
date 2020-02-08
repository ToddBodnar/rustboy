pub struct Memory {
    pub ram: Vec<u8>
}

impl Memory {
    pub fn set(&mut self, loc: u16, val: u8){
        self.ram[loc as usize] = val;
    }

    pub fn get(&self, loc: u16) -> u8{
        return self.ram[loc as usize];
    }
}

pub struct Engine {
    pub memory: Memory,
    pub registers: Registers
}

impl Engine {
    pub fn run(&mut self){

        for _ in 1..100 {
            let wait_time = self.execute_next_instruction();
        }
    }

    fn get_d8(&self, start: u16) -> u8 {
        return self.memory.get(start);
    }

    fn get_a16(&self, start: u16) -> u16 {
        return ((self.memory.get(start) as u16) << 8)
              + (self.memory.get(start+1) as u16);
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

        println!("{:?} = {:?}, {:?}", first_byte, second_nibble, first_nibble);
        match first_byte {
            0x00 => {
                // no op
                self.registers.incr_pc(1);
                return 4 as u8;
            },
            0x04 => self.inc(RegisterNames::B),
            0x05 => self.dec(RegisterNames::B),
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
            0x80..=0x8F => { //add + addc
                let initial_first = self.registers.get_register(&RegisterNames::A);
                let mut initial_second = self.registers.get_register(&resolved_second_register);

                if (first_byte > 0x87) & self.registers.is_cary_flag() {
                    initial_second += 1;
                }

                self.registers.set_register(&RegisterNames::A, initial_first + initial_second);
                self.registers.incr_pc(1);

                self.registers.set_register(&RegisterNames::F,
                    Registers::make_flags((initial_first + initial_second) as u8 == 0,
                                false,
                                // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                (initial_first + initial_second) & 0x10 == 0x10,
                                initial_first + initial_second > 0xF
                ));

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

            0xEA => { // load a into a16
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(3);

                println!("{:?}", self.memory.get(target));

                return 16 as u8;
            }

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
}

#[derive(PartialEq, Eq)]
enum RegisterNames {
    A,B,C,D,E,F,H,L,
    AF,BC,DE,HL,
    PC,SP
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

    fn incr_pc(& mut self, amount: u16){
        self.pc += amount;
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

            RegisterNames::AF => return self.a as u16 + ((self.f as u16) << 8),
            RegisterNames::BC => return self.b as u16 + ((self.c as u16) << 8),
            RegisterNames::DE => return self.d as u16 + ((self.e as u16) << 8),
            RegisterNames::HL => return self.h as u16 + ((self.l as u16) << 8),

            RegisterNames::SP => return self.sp,
            RegisterNames::PC => return self.pc,

            _ => {
                println!("TODO: IMPLEMENT REMAINING REGISTERS");
                return 0;
            }
        }
    }
}
