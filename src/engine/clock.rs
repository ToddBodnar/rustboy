use std::fmt;

use crate::engine::memory::Memory;

#[derive(Debug)]
pub struct Clock {
    pub time: u32
}

impl Clock {
    pub fn make_clock() -> Clock{
        return Clock {time: 0}
    }

    pub fn tick(&mut self, memory: &mut Box<dyn Memory>, ticks: u32) {

        //println!("{}", memory.get(0xFF07));
        if (memory.get(0xFF07) & 0x4) == 0 {
            //self.time = 0;
            return;
        }
        self.time += ticks;

        let tma = memory.get(0xFF06);

        let clock_rate = match memory.get(0xFF07) % 4 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("how did we get here {}", memory.get(0xFF07))
        } as u32;

        let mut tima = memory.get(0xFF05);

        if tima == 0 {
            memory.set(0xFF05, tma);
            tima = tma;
        }

        if self.time > clock_rate {
            self.time -= clock_rate;

            if tima == 0xFF {
                let flags = memory.get(0xFF0F);
                if flags & 0x04 == 0 {
                    memory.set(0xFF0F, flags + 0x04);
                }
                memory.set(0xFF05, 0);
            } else {
                memory.set(0xFF05, tima + 1);
            }
        }
    }
}
