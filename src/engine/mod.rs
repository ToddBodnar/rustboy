mod registers;
mod gpu;
pub mod engine;

pub fn make_engine(rom: Vec::<u8>) -> engine::Engine {
    let mut memory = engine::Memory{
        ram:  vec![0; 0xFFFF + 1],
        rom: rom,
        bank_n: 1,
        ram_bank_n: 1,
        ram_banks: vec![vec![0; 0x1000]; 16],
        memory_model_is_4_32: false,
        ram_bank_ops_disabled: false
    };

    //unsafe {memory.ram.set_len(0xFFFF+1);}

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
