use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod engine;

fn main() {
    let rom_file = env::args().nth(1).expect("Need rom file!");

    println!("Using file {}", rom_file);

    let mut rom = Vec::<u8>::new();

    let mut rom_file_ptr = File::open(&rom_file).expect("Bad file name!");

    rom_file_ptr.read_to_end(&mut rom).expect("Couldn't read file");

    let mut eng = engine::make_engine(rom);

    let save_file_name = rom_file + ".sav";
    if Path::new(&save_file_name).exists() {
        println!("Loading save from {}", save_file_name);
        let mut sav_ram = Vec::<u8>::new();

        let mut sav_ram_file_ptr = File::open(&save_file_name).expect("Bad save file name!");

        sav_ram_file_ptr.read_to_end(&mut sav_ram).expect("Couldn't read save file");
        eng.memory.load(sav_ram);
    }

    eng.run(false);

    print!("\nLCD Control\n{:#010b}", eng.memory.get(0xFF40));

    print!("\nLCD Stat\n{:#010b}", eng.memory.get(0xFF41));
    print!("\nLCD Control\n{:#010b}", eng.memory.get(0xFF40));
    print!("\nLCD Scroll Y\n{}", eng.memory.get(0xFF42));
    print!("\nLCD Scroll X\n{}", eng.memory.get(0xFF43));
    print!("\nLCD Current Y\n{}", eng.memory.get(0xFF44));
    print!("\nLCD OAM DMA Xfer\n{:#010b}", eng.memory.get(0xFF46));

    print!("\nCharacters\n");
    for tile in 0x8000..0x97FF {
        print!("{:x?},", eng.memory.get(tile));
    }

    print!("\nBG 1\n");
    for tile in 0x9800..0x9BFF {
        print!("{:x?},", eng.memory.get(tile));
    }

    print!("\nBG 2\n");
    for tile in 0x9C00..0x9FFF {
        print!("{}", eng.memory.get(tile));
    }

    print!("\nOAM (sprites)\n");
    for tile in 0xFE00..0xFE9F {
        print!("{}", eng.memory.get(tile));
    }

    print!("\nRegisters\n");
    print!("{:?}", eng.registers);


    print!("\nImage:\n");
    for i in 0..144 {
        //println!("{:?}", eng.gpu.lcd[i]);
    }

    println!("\nTimer\n");
    println!("{:x?}, {:x?}", eng.memory.get(0xFF06), eng.memory.get(0xFF07));


    println!("\nInterrupts\n");
    println!("{:x?} x {:x?}", eng.memory.get(0xFF0F), eng.memory.get(0xFFFF));
    println!("{:?}", eng.enable_interrupt);

    println!("\nKeys\n{:#010b}", eng.memory.get(0xFF00));

    let to_save = eng.memory.save();

    if (to_save.len() == 0){
        println!("Nothing to save");
    }else {
        println!("Saving state");
        let mut out_file = File::create(save_file_name).expect("Couldn't save!");
        out_file.write_all(&to_save).expect("Write failed");
        println!("Done");
    }
}
