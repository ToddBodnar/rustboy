use std::fmt;

use crate::engine::engine::Memory;

#[derive(Debug)]
pub struct GPU {
    time: u32,
    line: u8,
    mode: GpuState
}

impl GPU {
    pub fn make_gpu() -> GPU {
        return GPU {
            time: 0,
            line: 0,
            mode: GpuState::H_BLANK
        };
    }

    pub fn tick(&mut self, memory: &mut Memory, ticks: u32){
        self.time += ticks;

        //timing based on http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings, not sure of
        match self.mode {
            GpuState::H_BLANK => {
                if self.time > 204 {
                    self.time -= 204;
                    self.line += 1;
                    self.set_lcdc_y(memory, self.line);
                }

                if self.line == 143 {
                    self.mode = GpuState::V_BLANK;
                    self.line = 0;
                    //todo: draw
                } else {
                    self.mode = GpuState::SCAN_OAM;
                }
            },
            GpuState::V_BLANK => {
                if self.time >= 456 {
                    self.time -= 456;
                    self.line += 1;

                    self.set_lcdc_y(memory, self.line + 143);

                    if self.line > 154 {
                        self.line = 0;

                        self.set_lcdc_y(memory, 0);
                        self.mode = GpuState::SCAN_OAM;
                    }
                }
            },
            GpuState::SCAN_OAM => {
                if self.time >= 80 {
                    self.time -= 80;
                    self.mode = GpuState::SCAN_VRAM;
                }
            },
            GpuState::SCAN_VRAM => {
                if self.time >= 172 {
                    self.time -= 172;
                    self.mode = GpuState::H_BLANK;
                    //todo: draw
                }
            }
        };
    }

    fn set_lcdc_y(&mut self, memory: &mut Memory, amt: u8){
        memory.set(0xFF44, amt);
    }
}

#[derive(Debug)]
enum GpuState{
   SCAN_OAM,
   SCAN_VRAM,
   H_BLANK,
   V_BLANK
}
