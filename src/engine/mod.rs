mod registers;
mod gpu;
pub mod engine;

pub fn make_engine(rom: Vec::<u8>) -> engine::Engine {
    let mut memory = engine::Memory{
        ram:  rom
    };

    unsafe {memory.ram.set_len(0xFFFF+1);}

    memory.ram[0xFF40] = 0x91; // set LCDC


    let mut gpu = gpu::GPU::make_gpu();

    //gpu.tick(&mut memory, 800);

    return engine::Engine{
        memory: memory,
        registers: registers::Registers::make_registers(),
        enable_interrupt: false,
        gpu: gpu
    };
}
