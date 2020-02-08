use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

mod engine;

fn main() {
    let rom_file = env::args().nth(1).expect("Need rom file!");

    println!("Using file {}", rom_file);

    let mut rom = Vec::<u8>::new();

    let mut rom_file_ptr = File::open(rom_file).expect("Bad file name!");

    rom_file_ptr.read_to_end(&mut rom).expect("Couldn't read file");

    let mut memory = engine::Memory{
        ram:  rom
    };

    unsafe {memory.ram.set_len(0xFFFF);}


    let mut eng = engine::Engine{
        memory: memory,
        registers: engine::Registers{
            pc: 0x100,
            sp: 0xFFFE,

            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,

            h: 0,
            l: 0,
        },
        enable_interrupt: false
    };

    eng.run();
}
