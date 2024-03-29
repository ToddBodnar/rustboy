use crate::engine::registers::Registers;
use crate::engine::registers::RegisterNames;
use crate::engine::engine::KeyNames;

/// Common interface for various types of memory mapped RAM supported in different GB cards
pub trait Memory {
    
    /// Set a specific memory mapped point 
    fn set(&mut self, loc: u16, val: u8);

    /// Get a specific memory mapped point 
    fn get(&self, loc: u16) -> u8 ;

    /// load a rom into memory
    fn load(&mut self, data: Vec<u8>);

    /// handle saving for writable cards 
    fn save(&self) -> Vec<u8> ;

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
    return match rom[0x0147] {
        0x00 => Box::new(ROMOnlyMemory::make_memory(rom)),
        0x01 | 0x02 | 0x03 => Box::new(MBC1Memory::make_memory(rom)),
        0x13 => Box::new(MBC3Memory::make_memory(rom)),
        0x1b => Box::new(MBC5Memory::make_memory(rom)),
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

    fn load(&mut self, ram: Vec<u8>) {
        //nothing to load MBC1 doesn't support saving
    }

    fn save(&self) -> Vec<u8> {
        return vec![];
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

    fn load(&mut self, ram: Vec<u8>) {
        for bank in 0..16 {
            for loc in 0..0x1000 {
                self.ram_banks[bank][loc] = ram[bank * 0x1000 + loc];
            }
        }
    }

    fn save(&self) -> Vec<u8> {
        let mut res = vec![0; 0x1000 * 16];
        for bank in 0..16 {
            for loc in 0..0x1000 {
                res[bank * 0x1000 + loc] = self.ram_banks[bank][loc];
            }
        }

        return res;
    }
}




#[derive(Debug)]
pub struct MBC3Memory {
    rom: Vec<u8>,
    ram: Vec<u8>,
    bank_n: u32,
    ram_bank_n: u32,
    ram_banks: Vec<Vec<u8>>,
    memory_model_is_4_32: bool,
    ram_bank_ops_disabled: bool
}

impl MBC3Memory {
    fn make_memory(rom: Vec<u8>) -> impl Memory {
        println!("Making MBC3 Memory");
        MBC3Memory {
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

impl Memory for MBC3Memory {
    fn set(&mut self, loc: u16, val: u8) {
        if loc < 0x8000 {
            match loc {
                0x0000..=0x1FFF => {
                    self.ram_bank_ops_disabled = val % 16 != 10;
                },
                0x2000..=0x3FFF => {
                    if !self.ram_bank_ops_disabled {
                        if (val & 0b01111111) == 0 {
                            self.bank_n = 1;
                        } else {
                            self.bank_n = (val & 0b01111111) as u32;
                        }
                    }
                },
                0x4000..=0x5FFF => {
                    self.ram_bank_n = val as u32;
                },
                0x6000..=0x7FFF => {
                        //todo: clock logic
                },
                _ => {
                    println!("How did we get to {}", loc);
                }
            }
        } else if loc >= 0xA000 && loc < 0xC000 {
            self.ram_banks[self.ram_bank_n as usize][loc as usize - 0xA000] = val;
        } else if loc >= 0xE000 && loc < 0xF000{
            return self.set(loc - 0x2000, val);
        } else {
            self.ram[loc as usize] = val;
        }

        if loc < 0xFF00 {
            //println!("setting {:x} to {:x}", loc, val);
        }
    }

    fn get(&self, loc: u16) -> u8 {
        if loc < 0x4000 {
            return self.rom[loc as usize];
        } else if loc < 0x8000 {
            let resolved_loc = (0x4000 * self.bank_n + (loc as u32 - 0x4000)) as usize;

            return self.rom[resolved_loc];
        } else if loc >= 0xA000 && loc < 0xC000 {
            return self.ram_banks[self.ram_bank_n as usize][loc as usize - 0xA000];
        } else if loc >= 0xE000 && loc < 0xF000{
            return self.get(loc - 0x2000);
        } else {
            return self.ram[loc as usize];
        }
    }

    fn load(&mut self, ram: Vec<u8>) {
        
        for bank in 0..16 {
            for loc in 0..0x1000 {
                self.ram_banks[bank][loc] = ram[bank * 0x1000 + loc];
            }
        }
    }

    fn save(&self) -> Vec<u8> {
        let mut res = vec![0; 0x1000 * 16];
        for bank in 0..16 {
            for loc in 0..0x1000 {
                res[bank * 0x1000 + loc] = self.ram_banks[bank][loc];
            }
        }

        return res;
    }
}


#[derive(Debug)]
pub struct MBC5Memory {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank_n: u32,
    rom_bank_hi: bool,
    ram_bank_n: u32,
    ram_banks: Vec<Vec<u8>>,
    memory_model_is_4_32: bool,
    ram_bank_ops_disabled: bool
}

impl MBC5Memory {
    fn make_memory(rom: Vec<u8>) -> impl Memory {
        println!("Making MBC5 Memory");
        MBC5Memory {
            ram:  vec![0; 0xFFFF + 1],
            rom: rom,
            rom_bank_n: 1,
            rom_bank_hi: false,
            ram_bank_n: 1,
            ram_banks: vec![vec![0; 0x2000]; 16],
            memory_model_is_4_32: false,
            ram_bank_ops_disabled: false
        }
    }
}

impl Memory for MBC5Memory {
    fn set(&mut self, loc: u16, val: u8) {
        if loc < 0x8000 {
            match loc {
                0x0000..=0x1FFF => {
                    self.ram_bank_ops_disabled = val == 10;
                },
                0x2000..=0x2FFF => {
                    self.rom_bank_n = val as u32 + if self.rom_bank_hi {0b100000000} else {0};
                },
                0x3000..=0x3FFF => {
                    self.rom_bank_hi = val % 2 == 1;
                    self.rom_bank_n = (self.rom_bank_n & 0b11111111) + if self.rom_bank_hi {0b100000000} else {0};
                },
                0x4000..=0x5FFF => {
                    self.ram_bank_n = (val & 0xFF) as u32;
                },
                _ => {
                    println!("How did we get to {}", loc);
                }
            }
        } else if loc >= 0xA000 && loc < 0xC000 {
            self.ram_banks[self.ram_bank_n as usize][loc as usize - 0xA000] = val;
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
            let resolved_loc = (0x4000 * self.rom_bank_n + (loc as u32 - 0x4000)) as usize;

            return self.rom[resolved_loc];
        } else if loc >= 0xA000 && loc < 0xC000 {
            return self.ram_banks[self.ram_bank_n as usize][loc as usize - 0xA000];
        } else if loc >= 0xE000 && loc < 0xF000{
            return self.get(loc - 0x2000);
        } else {
            return self.ram[loc as usize];
        }
    }

    fn load(&mut self, ram: Vec<u8>) {
        for bank in 0..16 {
            for loc in 0..0x2000 {
                self.ram_banks[bank][loc] = ram[bank * 0x2000 + loc];
            }
        }
    }

    fn save(&self) -> Vec<u8> {
        let mut res = vec![0; 0x2000 * 16];
        for bank in 0..16 {
            for loc in 0..0x2000 {
                res[bank * 0x2000 + loc] = self.ram_banks[bank][loc];
            }
        }

        return res;
    }
}
