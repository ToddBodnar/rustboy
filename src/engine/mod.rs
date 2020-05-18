mod registers;
mod gpu;
mod clock;
mod memory;
pub mod engine;

pub fn make_engine(rom: Vec::<u8>) -> engine::Engine {
    let mut memory = memory::make_memory(rom);

    /*engine::Memory{
        ram:  vec![0; 0xFFFF + 1],
        rom: rom,
        bank_n: 1,
        ram_bank_n: 1,
        ram_banks: vec![vec![0; 0x1000]; 16],
        memory_model_is_4_32: false,
        ram_bank_ops_disabled: false
    };*/

    //unsafe {memory.ram.set_len(0xFFFF+1);}

    memory.set(0xFF40, 0x91); // set LCDC


    let mut gpu = gpu::GPU::make_gpu();

    //gpu.tick(&mut memory, 800);

    return engine::Engine{
        memory: memory,
        registers: registers::Registers::make_registers(),
        enable_interrupt: engine::InterruptState::DISABLED,
        gpu: gpu,
        clock: clock::Clock::make_clock()
    };
}
