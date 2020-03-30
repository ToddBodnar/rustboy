use std::fmt;

use crate::engine::gpu::GPU;
use crate::engine::registers::Registers;
use crate::engine::registers::RegisterNames;


extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

#[derive(Debug)]
pub struct Memory {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub bank_n: u16
}

impl Memory {
    //todo: rest of these
    fn is_mbc1(&self) -> bool {
        println!("memory type {}", self.rom[0x0147]);
        return self.rom[0x0147] == 0x01;
    }

    pub fn set(&mut self, loc: u16, val: u8) {
        if loc < 0x8000 {
            if self.is_mbc1() {
                match loc {
                    0x0000..=0x1FFF => {
                        panic!("todo: set external ram {}", val);
                    },
                    0x2000..=0x4FFF => {
                        if val == 0 {
                            self.bank_n = 1;
                        } else {
                            self.bank_n = val as u16;
                        }
                        println!("Setting memory to {}", self.bank_n);
                    },
                    0x4000..=0x5FFF => {
                        panic!("todo: set high rom bank {}", val);
                    },
                    0x6000..=0x7FFF => {
                        panic!("todo: set mode {}", val);
                    },
                    _ => {
                        println!("How did we get to {}", loc);
                    }
                }
            } else {
                println!("writing to rom loc {:x?}->{}", loc, val);
            }
        } else {
            self.ram[loc as usize] = val;
        }
    }

    pub fn setLong(&mut self, loc: u16, val: u16){
        let low_byte = (val & 0xFF) as u8;
        let high_byte = (val / 0x100) as u8;

        self.set(loc, low_byte);
        self.set(loc + 1, high_byte);
    }

    pub fn get(&self, loc: u16) -> u8 {
        if loc < 0x4000 {
            return self.rom[loc as usize];
        } else if loc < 0x8000 {
            return self.rom[(0x4000 * self.bank_n + (loc - 0x4000)) as usize]
        } else {
            return self.ram[loc as usize];
        }
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
    pub enable_interrupt: bool,
    pub gpu: GPU
}

impl Engine {
    pub fn run(&mut self, headless: bool){
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let width = 800;
        let height = 600;
        let window = match headless {

            false => video_subsystem.window("Rust Boy", 800, 600).build().unwrap(),
            true => video_subsystem.window("Rust Boy", 800, 600).hidden().build().unwrap()
        };

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut total_steps = 0;
        self.gpu.draw(&mut canvas, width, height);

        'running: loop {

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },//todo: rest of these
                    _ => {}
                }
            }
            if(headless){
                self.run_limited(100);
                break 'running
            }

            total_steps += self.run_limited(1);

            if total_steps > 1_000 {
                total_steps -= 1_000;
                self.gpu.draw(&mut canvas, width, height);
            }
        }
    }

    fn run_limited(&mut self, itrs: u64) -> u64{
        let mut total_steps = 0 as u64;
        for i in 0..itrs {
            let wait_time = self.execute_next_instruction();

            self.gpu.tick(&mut self.memory, wait_time);
            total_steps += wait_time as u64;
        }
        return total_steps;
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

    fn execute_next_instruction(&mut self) -> u32 {
        let first_byte = self.memory.get(self.registers.pc);

        println!("{}", self.registers);
        println!("{:x?} -> {:x?}", self.registers.pc, first_byte);
        let first_nibble = first_byte >> 4;
        let second_nibble = first_byte & 0x0F;
        //println!("Second nibble {}", second_nibble);

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
                return 4;
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
                return 12;
            },

            0x03 | 0x0B | 0x13 | 0x1B | 0x23 | 0x2B | 0x33 | 0x3B => {
                let resolved_register = match first_byte {
                    0x03 | 0x0B => RegisterNames::BC,
                    0x13 | 0x1B => RegisterNames::DE,
                    0x23 | 0x2B => RegisterNames::HL,
                    0x33 | 0x3B => RegisterNames::SP,
                    _ => panic!("how did we get here?")
                };

                if first_byte & 0x0F == 3 {
                    self.registers.change_register(resolved_register, 1);
                } else {
                    self.registers.change_register(resolved_register, -1);
                }
                self.registers.incr_pc(1);

                return 8;
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

            0x34 => self.inc(RegisterNames::HL) + 4,
            0x35 => self.dec(RegisterNames::HL) + 4,
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
                    //println!("loading {:x} -> ({:x})", memory_loc, memory_val);
                    self.registers.set_register(&RegisterNames::A, memory_val as u16);
                }

                 match (first_byte) {
                     0x22 | 0x2A => self.registers.change_register(RegisterNames::HL, 1),
                     0x32 | 0x3A => self.registers.change_register(RegisterNames::HL, -1),
                     _ => {}
                 };

                self.registers.incr_pc(1);
                return 8;
            },

            0x08 => {
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.memory.setLong(target, self.registers.get_register(&RegisterNames::SP));

                self.registers.incr_pc(3);
                return 20;
            },

            0x09 | 0x19 | 0x29 | 0x39 => {
                let other_val = match first_byte {
                    0x09 => self.registers.get_register(&RegisterNames::BC),
                    0x19 => self.registers.get_register(&RegisterNames::DE),
                    0x29 => self.registers.get_register(&RegisterNames::HL),
                    0x39 => self.registers.get_register(&RegisterNames::SP),
                    _ => panic!("How did we get here?")
                };

                // zero flag doesn't get changed
                let old_z = self.registers.is_zero_flag();

                self.math_to_reg_reshl(&RegisterNames::HL, MathNames::ADD, other_val, false);

                //println!("A: {}, {}, {}", self.registers.is_zero_flag(), self.registers.is_cary_flag(), self.registers.is_half_cary_flag());
                self.registers.set_zero_flag(old_z);

                self.registers.incr_pc(1);
                return 8;
            },

            0x2F | 0x3F => {
                let oldZ = self.registers.is_zero_flag();

                let carry =  if first_byte == 0x2F {self.registers.is_cary_flag()}
                                            else{ !self.registers.is_cary_flag()};

                let oldA = self.registers.get_register(&RegisterNames::A);

                self.registers.set_register(&RegisterNames::A, !oldA);
                self.registers.set_flags(oldZ, true, true, carry);

                self.registers.incr_pc(1);
                return 4;
            },

            0x36 => {
                let memory_loc = self.registers.get_register(&RegisterNames::HL);

                let d8 = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1);

                self.memory.set(memory_loc, d8);

                self.registers.incr_pc(2);
                return 12;
            }

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

                self.registers.set_register(&target_register, source_value as u16);
                self.registers.incr_pc(2);
                return 8;
            },

            0x0F | 0x1F => {
                //todo: better understanding of RRCA vs RRA

                let initial = self.registers.get_register(&RegisterNames::A);

                let mut res = initial >> 1;
                if first_byte == 0x1F {
                    res += (self.registers.get_register(&RegisterNames::F) / 0x10 % 2 * 0x80);
                }

                self.registers.set_register(&RegisterNames::A, res);
                self.registers.set_flags(res == 0, false, false, initial % 2 == 1);


                self.registers.incr_pc(1);
                return 4;
            }

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
                    return 8;
                } else {
                    let to_increase = self.get_r8(self.registers.get_register(&RegisterNames::PC) + 1);
                    self.registers.incr_pc(to_increase as i32 + 2); //+2 to handle current instr
                    return 12;
                }
            }

            // load block
            0x40..=0x7F => {
                if first_byte == 0x76 {
                    println!("Halt!!!");
                    self.registers.incr_pc(1);
                    return 4;
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
                    if resolved_first_register == RegisterNames::HL {
                        let hl_val = self.registers.get_register(&RegisterNames::HL);
                        self.memory.set(hl_val, initial as u8);
                    } else {
                        self.registers.set_register(&resolved_first_register, initial);
                    }

                    self.registers.incr_pc(1);

                    if resolved_first_register == RegisterNames::HL || resolved_second_register == RegisterNames::HL {
                        return 8;
                    }
                    return 4;
                }
            },
            0x80..=0x87 => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::ADD, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0x88..=0x8F => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::ADDC, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0x90..=0x97 => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::SUB, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0x98..=0x9F => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::SUBC, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0xA0..=0xA7 => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::AND, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0xA8..=0xAF => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::XOR, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0xB0..=0xB7 => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::OR, other_value);
                self.registers.incr_pc(1);

                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0xB8..=0xBF => {
                let other_value = self.registers.get_register(&resolved_second_register);
                self.math_to_a(MathNames::CP, other_value);
                self.registers.incr_pc(1);
                if resolved_second_register == RegisterNames::HL {
                    return 8;
                } else {
                    return 4;
                }
            },

            0xC2 => { // jump nz
                if self.registers.is_zero_flag() {
                    self.registers.incr_pc(3);

                    return 12;
                } else {
                    let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);
                    self.registers.set_register(&RegisterNames::PC, target);

                    return 16;
                }
            },

            0xC3 => { // jump
                //println!("jump {:?} , {:?}", self.rom[(self.registers.pc + 1) as usize], self.rom[(self.registers.pc + 2) as usize]);
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);
                //println!("target is {:?}", target);
                self.registers.set_register(&RegisterNames::PC, target);
                return 16;
            },
            0xE9 => { // jump hp
                //println!("jump {:?} , {:?}", self.rom[(self.registers.pc + 1) as usize], self.rom[(self.registers.pc + 2) as usize]);
                let target = self.registers.get_register(&RegisterNames::HL);
                //println!("target is {:?}", target);
                self.registers.set_register(&RegisterNames::PC, self.get_a16(target));
                return 4;
            },

            0xCB => self.run_cb(),

            // returns
            0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 | 0xD9=> {
                let decide_to_jump = match first_byte {
                    0xC9 | 0xD9=> true,
                    0xC0 => !self.registers.is_zero_flag(),//nz
                    0xC8 => self.registers.is_zero_flag(),//z
                    0xD0 => !self.registers.is_cary_flag(),//nc
                    0xD8 => self.registers.is_cary_flag(),//c
                    _ => panic!("how did we get here?")
                };

                if first_byte == 0xD9 {
                    self.enable_interrupt = true;
                }

                self.registers.incr_pc(1);
                if !decide_to_jump {
                    return 8;
                } else {
                    let new_pc = self.memory.pop_stack(&mut self.registers);
                    self.registers.set_register(&RegisterNames::PC, new_pc);

                    if first_byte == 0xC9 || first_byte == 0xD9 {
                        return 16;
                    } else {
                        return 20;
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

                if second_nibble == 1 {
                    let new_reg = self.memory.pop_stack(&mut self.registers);
                    self.registers.set_register(&reg, new_reg);
                    return 12;
                } else {
                    let register_val = self.registers.get_register(&reg);
                    self.memory.push_stack(&mut self.registers, register_val);
                    return 16;
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
                    return 12;
                } else {
                    let old_pc = self.registers.get_register(&RegisterNames::PC);

                    self.memory.push_stack(&mut self.registers, old_pc);

                    self.registers.set_register(&RegisterNames::PC, a16);

                    return 24;
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
                let other_value = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1) as u16;
                self.math_to_a(math_type, other_value);
                self.registers.incr_pc(2);
                return 8;
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
                    0xFF => panic!("time to die"),//0x0038,
                    _ => panic!("how did we get here")
                };



                self.registers.set_register(&RegisterNames::PC, new_pc);

                return 16;
            },

            0xE0 => { // load a into a8
                let target = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1) as u16 + 0xFF00;

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(2);

                return 12;
            },

            0xE2 => { // load a into a8
                let target =self.registers.get_register(&RegisterNames::C) as u16 + 0xFF00;

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(1);

                return 8;
            },

            0xE8 => {
                let target = self.get_r8(self.registers.get_register(&RegisterNames::PC) + 1);
                self.math_to_reg(&RegisterNames::SP, MathNames::ADD, target as u16);

                self.registers.incr_pc(2);

                return 16;
            }

            0xEA => { // load a into a16
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                self.memory.set(target, self.registers.get_register(&RegisterNames::A) as u8);

                self.registers.incr_pc(3);

                println!("Setting {} to a", target);

                return 16;
            },

            0xF0 => { // load a8 into a
                let target = self.get_d8(self.registers.get_register(&RegisterNames::PC) + 1) as u16 + 0xFF00;

                let res = self.memory.get(target);
                println!("{:x} -> ({:x})", target, res);
                self.registers.set_register(&RegisterNames::A, res as u16);

                self.registers.incr_pc(2);

                return 12;
            },

            //interrupts
            0xF3 => {
                self.enable_interrupt = false;
                self.registers.incr_pc(1);
                return 4;
            },
            0xFB => {
                self.enable_interrupt = true;
                self.registers.incr_pc(1);
                return 4;
            },

            0xFA => { // load a8 into a
                let target = self.get_a16(self.registers.get_register(&RegisterNames::PC) + 1);

                let res = self.memory.get(target);
                self.registers.set_register(&RegisterNames::A, res as u16);

                self.registers.incr_pc(3);

                return 16;
            },

            _ => {
                print!("Don't understand instr {:x?}", self.memory.get(self.registers.get_register(&RegisterNames::PC)));

                self.registers.incr_pc(1);

                return 16;
            }
        }
    }

    fn run_cb(&mut self) -> u32 {
        self.registers.incr_pc(1);

        let first_byte = self.memory.get(self.registers.pc);

        //println!("{}", self.registers);
        //println!("{:x?} -> {:x?}", self.registers.pc, first_byte);
        let first_nibble = first_byte >> 4;
        let second_nibble = first_byte & 0x0F;

        let resolved_register = match second_nibble {
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

        let reg_val = self.registers.get_register(&resolved_register);
        let initial_val;
        if resolved_register == RegisterNames::HL {
            initial_val = self.memory.get(reg_val);
        } else {
            initial_val = reg_val as u8;
        }

        //todo: make this not a hack
        let res;
        self.registers.incr_pc(1);
        let steps = match first_byte {
            0x18..=0x1F => {
                res = initial_val / 2 + if self.registers.is_cary_flag() {0x80} else {0};

                self.registers.set_flags( res == 0, false, false, initial_val & 1 == 1);

                8
            },
            0x38..=0x3F => {
                res = initial_val / 2 + if self.registers.is_cary_flag() {1} else {0};

                self.registers.set_flags( res == 0, false, false, initial_val & 1 == 1);

                8
            },
            _ => {
                println!("unknown cb {:x?}", first_byte);
                res = 0;
                8
            }
        };

        if resolved_register == RegisterNames::HL {
            self.memory.set(reg_val, res);
            return 16;
        } else {
            self.registers.set_register(&resolved_register, res as u16);
        }

        return steps;
    }

    fn inc(&mut self, register: RegisterNames) -> u32 {
        let currentFlags = self.registers.get_register(&RegisterNames::F);

        let currentCary = self.registers.is_cary_flag();

        self.math_to_reg(&register, MathNames::ADD, 1);

        self.registers.incr_pc(1);

        if register == RegisterNames::BC || register == RegisterNames::DE || register == RegisterNames::HL || register == RegisterNames::SP {
            self.registers.set_register(&RegisterNames::F, currentFlags);
            return 8;
        } else {
            // carry isn't changed
            if self.registers.is_cary_flag() != currentCary {
                let flags = self.registers.get_register(&RegisterNames::F);
                self.registers.set_register(&RegisterNames::F, flags ^ 16);
            }
        }

        return 4;
    }

    fn dec(&mut self, register: RegisterNames) -> u32 {
        let currentFlags = self.registers.get_register(&RegisterNames::F);

        let currentCary = self.registers.is_cary_flag();

        self.math_to_reg(&register, MathNames::SUB, 1);

        self.registers.incr_pc(1);

        if register == RegisterNames::BC || register == RegisterNames::DE || register == RegisterNames::HL || register == RegisterNames::SP {
            self.registers.set_register(&RegisterNames::F, currentFlags);
            return 8;
        } else {
            // carry isn't changed
            if self.registers.is_cary_flag() != currentCary {
                let flags = self.registers.get_register(&RegisterNames::F);
                self.registers.set_register(&RegisterNames::F, flags ^ 16);
            }
        }

        return 4;
    }

    fn math_to_a(&mut self, math_type: MathNames, other_value: u16) {
        self.math_to_reg(&RegisterNames::A, math_type, other_value);
    }

    fn math_to_reg(&mut self, register: &RegisterNames, math_type: MathNames, other_value: u16) {
        self.math_to_reg_reshl(register, math_type, other_value, true)
    }

    fn math_to_reg_reshl(&mut self, register: &RegisterNames, math_type: MathNames, other_value: u16, resolve_hl: bool) {
        let reg_val = self.registers.get_register(register);
        let initial_a;

        let double_wide = match register {
            RegisterNames::HL => true,
            RegisterNames::SP => true,
            RegisterNames::PC => true,
            RegisterNames::AF => true,
            RegisterNames::BC => true,
            RegisterNames::DE => true,

            _ => false
        };

        if register == &RegisterNames::HL && resolve_hl {
            initial_a = self.memory.get(reg_val) as u16;
        } else {
            initial_a = reg_val;
        }

        let mut result;

        match math_type {
            MathNames::ADD => {
                let t_result = initial_a as u32 + other_value as u32;
                //println!("A: {:x?} + {:x?} = {:x?}", initial_a, other_value, t_result);

                if double_wide {
                    self.registers.set_flags((t_result % 0xFFFF) as u8 == 0,
                                    false,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    ((initial_a & 0xF00) + (other_value & 0xF00)) & 0x1000 == 0x1000,
                                    (t_result) > 0xFFFF
                    );
                } else {
                    self.registers.set_flags((t_result % 0xFFFF) as u8 == 0,
                                    false,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    ((initial_a & 0xF) + (other_value & 0xF)) & 0x10 == 0x10,
                                    (t_result) > 0xFF
                    );
                }

                result = (t_result % 0x10000) as u16;
            },

            MathNames::ADDC => {
                result = initial_a + other_value;
                let mut other_val_mut = other_value;
                if self.registers.is_cary_flag() {
                    result += 1;
                    other_val_mut += 1;
                }

                if double_wide {
                    self.registers.set_flags((initial_a + other_value) as u8 == 0,
                                    false,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    ((initial_a & 0xF00) + (other_val_mut & 0xF00)) & 0x1000 == 0x1000,
                                    (result) > 0xFFFF
                    );
                } else {
                    self.registers.set_flags((initial_a + other_value) as u8 == 0,
                                    false,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    ((initial_a & 0xF) + (other_val_mut & 0xF)) & 0x10 == 0x10,
                                    (result) > 0xFF
                    );
                }
            },

            MathNames::SUB => {
                if initial_a >= other_value {
                    result = initial_a - other_value;
                    if double_wide {
                        self.registers.set_flags(result & 0xFF00 == 0,
                                        true,
                                        // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                        ((initial_a & 0xF00) < (result & 0xF00)),
                                        false
                        );
                    } else {
                        self.registers.set_flags(result == 0,
                                        true,
                                        // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                        ((initial_a & 0xF) < (result & 0xF)),
                                        false
                        );
                    }
                } else {
                    if double_wide {
                        result = ((0x10000 as u32 + initial_a as u32) - other_value as u32) as u16;
                        self.registers.set_flags(result == 0,
                                        true,
                                        // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                        ((initial_a & 0xF00) < (result & 0xF00)),
                                        true
                        );
                    } else {

                        result = (0x100 + initial_a) as u16 - other_value;
                        self.registers.set_flags(result == 0,
                                        true,
                                        // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                        ((initial_a & 0xF) < (result & 0xF)),
                                        true
                        );
                    }
                }
            },
            MathNames::SUBC => {
                let to_sub = other_value + if self.registers.is_cary_flag() {1} else {0};

                if initial_a >= to_sub {
                    result = initial_a - to_sub;
                    self.registers.set_flags(result == 0,
                                    true,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    (initial_a - to_sub) & 0x10 == 0x10,
                                    false
                    );
                } else {
                    result = (256 + initial_a) as u16 - to_sub;
                    self.registers.set_flags(result == 0,
                                    true,
                                    // see https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
                                    (result) & 0x10 == 0x10,
                                    true
                    );
                }
            },

            MathNames::AND => {
                result = initial_a & other_value;


                self.registers.set_flags(result == 0,
                                false,
                                true,
                                false
                );
            },

            MathNames::OR => {
                result = initial_a | other_value;


                self.registers.set_flags(result == 0,
                                false,
                                false,
                                false
                );
            },

            MathNames::XOR => {
                result = initial_a ^ other_value;


                self.registers.set_flags(result == 0,
                                false,
                                false,
                                false
                );
            },

            MathNames::CP => {
                result = initial_a;

                let inner_result = ((0xFF100 + initial_a as u64 - other_value as u64) % 0x00100 )as u16;

                self.registers.set_flags(initial_a == other_value,
                                true,
                                ((initial_a & 0xF) < (inner_result & 0xF)),
                                initial_a < other_value
                );
            }
        };

        if register == &RegisterNames::HL && resolve_hl {
            self.memory.set(reg_val, result as u8);
        } else {
            self.registers.set_register(register, result);
        }
    }
}

#[derive(Debug)]
enum MathNames {
    ADD, ADDC,
    SUB, SUBC,
    AND, XOR, OR, CP
}

#[cfg(test)]
mod tests {
    use crate::engine::registers::Registers;
    use crate::engine::registers::RegisterNames;
    use crate::engine::engine::Engine;
    use crate::engine::gpu::GPU;
    use crate::engine::engine::Memory;
    use crate::engine::engine::MathNames;
    use crate::engine::make_engine;

    #[test]
    fn test_math_sub(){
        let mut reg = Registers::make_registers();

        let mut eng = Engine{
            memory: Memory{
                rom: vec![0,0],
                ram: vec![0,0],
                bank_n: 0
            },
            registers: reg,
            enable_interrupt: false,
            gpu: GPU::make_gpu()
        };

        eng.registers.set_register(&RegisterNames::A, 0);
        eng.math_to_a(MathNames::SUB, 1);

        assert_eq!(0xFF, eng.registers.get_register(&RegisterNames::A));
        assert!(eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::SUB, 1);

        assert_eq!(19, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::SUB, 20);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());
    }

    #[test]
    fn test_math_subc(){
        let mut reg = Registers::make_registers();

        let mut eng = Engine{
            memory: Memory{
                rom: vec![0,0],
                ram: vec![0,0],
                bank_n: 0
            },
            registers: reg,
            enable_interrupt: false,
            gpu: GPU::make_gpu()
        };

        eng.registers.set_register(&RegisterNames::A, 0);
        eng.registers.set_register(&RegisterNames::F, 0);

        eng.math_to_a(MathNames::SUBC, 1);

        assert_eq!(0xFF, eng.registers.get_register(&RegisterNames::A));
        assert!(eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::SUBC, 1);

        assert_eq!(18, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::SUBC, 20);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());
    }

    #[test]
    fn test_dec_c(){

        let mut rom = vec![0; 0xFFFF];

        rom[0x0100] = 0x0D;
        rom[0x0101] = 0x0D;

        let mut eng = make_engine(rom);
        eng.registers.set_register(&RegisterNames::C, 0x10);
        eng.registers.set_register(&RegisterNames::F, 0x00);

        eng.run_limited(1);

        println!("{:?}", eng.registers);

        assert_eq!(0x0F, eng.registers.c);
        assert_eq!(0x60, eng.registers.f);


        eng.registers.set_register(&RegisterNames::C, 0xF0);
        eng.registers.set_register(&RegisterNames::F, 0x00);

        eng.run_limited(1);

        println!("{:?}", eng.registers);

        assert_eq!(0xEF, eng.registers.c);
        assert_eq!(0x60, eng.registers.f);
    }

    #[test]
    fn test_xor_a(){

        let mut rom = vec![0; 0xFFFF];

        rom[0x0100] = 0xAF;

        let mut eng = make_engine(rom);
        eng.registers.set_register(&RegisterNames::A, 0x01);

        eng.run_limited(1);

        println!("{:?}", eng.registers);

        assert_eq!(0x00, eng.registers.a);
        assert_eq!(0x80, eng.registers.f);
    }

    #[test]
    fn test_addc_a(){

        let mut rom = vec![0; 0xFFFF];

        rom[0x0100] = 0xCE;
        rom[0x0101] = 0x01;

        let mut eng = make_engine(rom);
        eng.registers.set_register(&RegisterNames::A, 0xFF);
        eng.registers.set_register(&RegisterNames::F, 0x00);

        eng.run_limited(1);

        println!("{:?}", eng.registers);

        assert_eq!(0x00, eng.registers.a);
        assert_eq!(0xB0, eng.registers.f);
    }

    #[test]
    fn test_rr_a(){

        let mut rom = vec![0; 0xFFFF];

        rom[0x0100] = 0x1F;

        let mut eng = make_engine(rom);
        eng.registers.set_register(&RegisterNames::A, 0xFE);
        eng.registers.set_register(&RegisterNames::F, 0x70);

        eng.run_limited(1);

        println!("{:?}", eng.registers);

        assert_eq!(0xFF, eng.registers.a);
        assert_eq!(0x00, eng.registers.f);
    }

    #[test]
    fn test_math_add(){
        let mut reg = Registers::make_registers();

        let mut eng = Engine{
            memory: Memory{
                rom: vec![0,0],
                ram: vec![0,0],
                bank_n: 0
            },
            registers: reg,
            enable_interrupt: false,
            gpu: GPU::make_gpu()
        };

        eng.registers.set_register(&RegisterNames::A, 0xFF);
        eng.math_to_a(MathNames::ADD, 1);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::ADD, 1);

        assert_eq!(21, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 0);
        eng.math_to_a(MathNames::ADD, 0);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());
    }

    #[test]
    fn test_math_add_c(){
        let mut reg = Registers::make_registers();

        let mut eng = Engine{
            memory: Memory{
                rom: vec![0,0],
                ram: vec![0,0],
                bank_n: 0
            },
            registers: reg,
            enable_interrupt: false,
            gpu: GPU::make_gpu()
        };

        eng.registers.set_register(&RegisterNames::A, 0xFF);
        eng.registers.set_register(&RegisterNames::F, 0);
        eng.math_to_a(MathNames::ADDC, 1);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 20);
        eng.math_to_a(MathNames::ADDC, 1);

        assert_eq!(22, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(!eng.registers.is_zero_flag());

        eng.registers.set_register(&RegisterNames::A, 0);
        eng.math_to_a(MathNames::ADDC, 0);

        assert_eq!(0, eng.registers.get_register(&RegisterNames::A));
        assert!(!eng.registers.is_cary_flag());
        assert!(eng.registers.is_zero_flag());
    }

    #[test]
    fn test_jump_and_return(){
        //todo:
        // idea is to start at some point, jump, do some simple math, then execute a return nz, then loop at that point until the end,
        // then we check if we actually jumped to the right places

        let mut rom = vec![0; 0xFFFF];

        rom[0x0100] = 0xCD; rom[0x0101] = 0x00; rom[0x0102] = 0x11; // call the sub routine
        rom[0x0103] = 0x0E; rom[0x0104] = 42; // load 42 into C (after return)

        rom[0x1100] = 0x3E; rom[0x1101] = 0x01; // load 1 into A
        rom[0x1102] = 0x06; rom[0x1103] = 0x05; // loat 5 into B
        rom[0x1104] = 0; //no op LABEL START
        rom[0x1105] = 0xC6; rom[0x1106] = 10; // add 10 into A
        rom[0x1107] = 0x05; // dec b
        rom[0x1108] = 0xC2; rom[0x1109] = 0x04; rom[0x110A] = 0x11; // if not zero, jump to START
        rom[0x110B] = 0xC8; // return if zero

        let mut eng = make_engine(rom);
        eng.run_limited(100);

        println!("{:?}", eng.registers);

        assert_eq!(0, eng.registers.b);
        assert_eq!(51, eng.registers.a);
        assert_eq!(42, eng.registers.c);
    }
}
