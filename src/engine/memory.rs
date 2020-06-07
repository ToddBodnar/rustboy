use crate::engine::registers::Registers;
use crate::engine::registers::RegisterNames;
use crate::engine::engine::KeyNames;

pub trait Memory {
    fn set(&mut self, loc: u16, val: u8);

    fn get(&self, loc: u16) -> u8 ;

    fn setInterruptFlag(&mut self, flag: u8) {
        let interrupts = self.get(0xFF0F);
        if (interrupts & (1 << (flag))) == 0 {
            self.set(0xFF0F,  interrupts + (1 << (flag)));
        }
    }

    fn setLong(&mut self, loc: u16, val: u16) {
        let low_byte = (val & 0xFF) as u8;
        let high_byte = (val / 0x100) as u8;

        self.set(loc, low_byte);
        self.set(loc + 1, high_byte);
    }

    fn push_stack(&mut self, reg: &mut Registers, val: u16) {
        let low_byte = (val & 0xFF) as u8;
        let high_byte = (val / 0x100) as u8;

        let sp = reg.get_register(&RegisterNames::SP);

        self.set(sp - 2, low_byte);
        self.set(sp - 1, high_byte);

        reg.set_register(&RegisterNames::SP, sp - 2);
    }

    fn pop_stack(&self, reg: &mut Registers) -> u16 {
        let sp = reg.get_register(&RegisterNames::SP);

        let high_byte = self.get(sp + 1) as u16;
        let low_byte = self.get(sp + 0) as u16;

        reg.set_register(&RegisterNames::SP, sp + 2);

        return high_byte * 0x100 + low_byte;
    }
}

pub fn make_memory(rom: Vec::<u8>) -> Box<dyn Memory> {
    return match(rom[0x0147]) {
        0x00 => Box::new(ROMOnlyMemory::make_memory(rom)),
        0x01 | 0x02 | 0x03 => Box::new(MBC1Memory::make_memory(rom)),
        _ => panic!("Don't understand cartridge type {:x?}", rom[0x0147])
    };
}

#[derive(Debug)]
pub struct ROMOnlyMemory {
    rom: Vec<u8>,
    ram: Vec<u8>
}

impl ROMOnlyMemory {
    fn make_memory(rom: Vec<u8>) -> impl Memory {
        println!("Making ROM only Memory");
        ROMOnlyMemory {
            ram: vec![0; 0xFFFF + 1],
            rom: rom
        }
    }
}

impl Memory for ROMOnlyMemory {
    fn set(&mut self, loc: u16, val: u8) {
        if loc >= 0xE000 && loc < 0xF000{
            self.set(loc - 0x2000, val);
        } else if loc >= 0x8000 {
            self.ram[loc as usize] = val;
        }
    }

    fn get(&self, loc: u16) -> u8 {
        if loc >= 0xE000 && loc < 0xF000{
            return self.get(loc - 0x2000);
        } else if loc >= 0x8000 {
            return self.ram[loc as usize];
        }
        return self.rom[loc as usize];
    }
}




#[derive(Debug)]
pub struct MBC1Memory {
    rom: Vec<u8>,
    ram: Vec<u8>,
    bank_n: u32,
    ram_bank_n: u32,
    ram_banks: Vec<Vec<u8>>,
    memory_model_is_4_32: bool,
    ram_bank_ops_disabled: bool
}

impl MBC1Memory {
    fn make_memory(rom: Vec<u8>) -> impl Memory {
        println!("Making MBC1 Memory");
        MBC1Memory {
            ram:  vec![0; 0xFFFF + 1],
            rom: rom,
            bank_n: 1,
            ram_bank_n: 1,
            ram_banks: vec![vec![0; 0x1000]; 16],
            memory_model_is_4_32: false,
            ram_bank_ops_disabled: false
        }
    }
}

impl Memory for MBC1Memory {
    fn set(&mut self, loc: u16, val: u8) {
        if loc < 0x8000 {
            match loc {
                0x0000..=0x1FFF => {
                    self.ram_bank_ops_disabled = val % 16 != 10;
                },
                0x2000..=0x3FFF => {
                    if !self.ram_bank_ops_disabled {
                        if val == 0 {
                            self.bank_n = 1;
                        } else {
                            self.bank_n = (val % 16) as u32;
                        }
                    }
                },
                0x4000..=0x5FFF => {
                    if !self.ram_bank_ops_disabled {
                        self.ram_bank_n = (val % 4)  as u32;
                    }
                },
                0x6000..=0x7FFF => {
                    self.memory_model_is_4_32 = val % 2 == 1;
                },
                _ => {
                    println!("How did we get to {}", loc);
                }
            }
        } else if loc >= 0xD000 && loc < 0xE000 {
            self.ram_banks[self.ram_bank_n as usize - 1][loc as usize - 0xD000] = val;
        } else if loc >= 0xE000 && loc < 0xF000{
            return self.set(loc - 0x2000, val);
        } else {
            self.ram[loc as usize] = val;
        }
    }

    fn get(&self, loc: u16) -> u8 {
        if loc < 0x4000 {
            return self.rom[loc as usize];
        } else if loc < 0x8000 {
            let resolved_loc = (0x4000 * self.bank_n + (loc as u32 - 0x4000)) as usize;

            return self.rom[resolved_loc];
        } else if loc >= 0xD000 && loc < 0xE000 {
            return self.ram_banks[self.ram_bank_n as usize - 1][loc as usize - 0xD000];
        } else if loc >= 0xE000 && loc < 0xF000{
            return self.get(loc - 0x2000);
        } else {
            return self.ram[loc as usize];
        }
    }
}
