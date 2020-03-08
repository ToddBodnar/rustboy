use std::fmt;

use crate::engine::engine::Memory;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Debug)]
pub struct GPU {
    time: u32,
    line: u8,
    mode: GpuState,
    pub lcd: Vec<Vec<u8>>
}

impl GPU {
    pub fn make_gpu() -> GPU {
        return GPU {
            time: 0,
            line: 0,
            mode: GpuState::H_BLANK,
            lcd: vec![vec![0; 160]; 144]
        };
    }

    pub fn draw<C: sdl2::render::RenderTarget>(&mut self,
        canvas : &mut sdl2::render::Canvas<C>,
        width: u32,
        height: u32){
        //todo

        let mut scale = 1.0 as f64;

        if (160.0 / 144.0 < (width as f64 / height as f64)) {
            scale = height as f64 / (144 as f64);
        }else{
            scale = width as f64 / (160 as f64);
        }

        canvas.set_draw_color(Color::RGB(0,0,255));
        canvas.clear();
        println!("scale {}", scale);
        for y in 0..144 {
            for x in 0..160 {
                let mut col = self.lcd[y][x];

                canvas.set_draw_color(Color::RGB(col, col, col));
                canvas.fill_rect(Rect::new(
                    (x as f64 * scale).round() as i32,
                    (y as f64 * scale).round() as i32,
                    scale.ceil() as u32,
                    scale.ceil() as u32));
            }
        }

        canvas.present();
    }

    pub fn tick(&mut self, memory: &mut Memory, ticks: u32){
        self.time += ticks;
        //println!("Gpu time is {} ({:?})", self.time, self.mode);
        //timing based on http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings, not sure of
        match self.mode {
            GpuState::H_BLANK => {
                if self.time >= 204 {
                    self.time -= 204;
                    self.line += 1;
                    self.set_lcdc_y(memory, self.line);

                    if self.line == 143 {
                        self.mode = GpuState::V_BLANK;
                        self.line = 0;
                    } else {
                        self.mode = GpuState::SCAN_OAM;
                        self.draw_line(memory, self.line - 1);
                    }
                }
            },
            GpuState::V_BLANK => {
                if self.time >= 456 {
                    self.time -= 456;
                    self.line += 1;

                    self.set_lcdc_y(memory, self.line + 143);

                    if self.line >= 10 {
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
                    self.set_lcdc_y(memory, self.line);
                    //todo: draw
                }
            }
        };
    }

    fn set_lcdc_y(&mut self, memory: &mut Memory, amt: u8){
        println!("---Y frame is now {} ({:?})", amt, self.mode);
        memory.set(0xFF44, amt);
    }

    fn draw_line(&mut self, memory: &mut Memory, line: u8){
        let map_loc = self.get_map_loc(memory);
        let inner_line = line as i16 + self.get_y_offset(memory);
        let x_offset = self.get_x_offset(memory);
        let tile_loc = 0x8000 as usize;
        let palet = memory.get(0xFF47);

        for x in 0..(160 / 8) {
            println!("{},{} -> {}",x,line, ((x as i16 * 8 + x_offset) / 8 + (inner_line / 8 as i16 * 32)+ map_loc as i16) as u16);
            let tile_id = memory.get(((x as i16 * 8 + x_offset) / 8 + (inner_line / 8 as i16 * 32)+ map_loc as i16) as u16) as usize;
            let tile_low = memory.get((tile_id * 2 * 8 + (inner_line as usize % 8) + tile_loc) as u16);
            let tile_high = memory.get((tile_id * 2 * 8 + 1 + (inner_line as usize % 8) + tile_loc) as u16);

            println!("Tile is {} -> {:x?},{:x?}", tile_id, tile_low, tile_high);

            for xi in 0..8 {
                let t_low = (tile_low >> (8-xi - 1)) & 0x1;
                let t_high = (tile_high >> (8-xi - 1)) & 0x1;

                println!("{},{}", t_low, t_high);

                let t_res = t_low + t_high * 2;
                println!("{:x?}", palet);
                let palet_loc = (palet >> (t_res * 2)) % 0x04;

                let col = match palet_loc {
                    3 => 0,
                    2 => 82,
                    1 => 173,
                    0 => 255,
                    _ => panic!("Bad pallet value {}", palet_loc)
                };
                self.lcd[line as usize][(x * 8 + xi) as usize] = col;
            }

            println!("Resolved to ");
            for xi in 0..8{
                println!("{}", self.lcd[line as usize][(x * 8 + xi) as usize]);
            }
        }
    }

    //todo
    fn get_map_loc(&mut self, memory: &mut Memory) -> u16 {
        0x9800
    }
    fn get_y_offset(&mut self, memory: &mut Memory) -> i16 {
        0
    }
    fn get_x_offset(&mut self, memory: &mut Memory) -> i16 {
        0
    }
}

#[derive(Debug)]
enum GpuState{
   SCAN_OAM,
   SCAN_VRAM,
   H_BLANK,
   V_BLANK
}
