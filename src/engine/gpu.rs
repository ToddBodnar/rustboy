use std::fmt;

use crate::engine::memory::Memory;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Debug)]
pub struct GPU {
    pub time: f32,
    pub line: u8,
    pub mode: GpuState,
    pub lcd: Vec<Vec<u8>>
}

impl GPU {
    pub fn make_gpu() -> GPU {
        return GPU {
            time: 0.0,
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

    pub fn tick(&mut self, memory: &mut Box<dyn Memory>, ticks: u32){
        self.time += ticks as f32;
        //println!("Gpu time is {} ({:?})", self.time, self.mode);
        //timing based on http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings, not sure of
        match self.mode {
            GpuState::H_BLANK => {
                if self.time >= 204.0 {
                    self.time -= 204.0;
                    self.line += 1;
                    self.set_lcdc_y(memory, self.line - 1);
                    self.update_stat(memory);

                    if self.line == 143 {
                        self.mode = GpuState::V_BLANK;
                        self.line = 0;
                    } else {
                        self.mode = GpuState::SCAN_OAM;
                        self.draw_line(memory, self.line);
                    }
                }
            },
            GpuState::V_BLANK => {
                if self.time >= 456.0 + 23.2{
                    self.time -= 456.0 + 23.2;
                    self.line += 1;

                    self.set_lcdc_y(memory, self.line + 143 - 1);
                    self.update_stat(memory);

                    if self.line >= 10 {
                        self.line = 0;

                        //self.set_lcdc_y(memory, 0);
                        self.mode = GpuState::SCAN_OAM;
                    }
                }
            },
            GpuState::SCAN_OAM => {
                if self.time >= 80.0 {
                    self.time -= 80.0;
                    self.mode = GpuState::SCAN_VRAM;
                    self.update_stat(memory);
                }
            },
            GpuState::SCAN_VRAM => {
                if self.time >= 172.0 {
                    self.time -= 172.0;
                    self.mode = GpuState::H_BLANK;
                    //self.set_lcdc_y(memory, self.line);
                    //todo: draw
                }
            }
        };
    }

    fn set_lcdc_y(&mut self, memory: &mut Box<dyn Memory>, amt: u8){
        //println!("---Y frame is now {} ({:?})", amt, self.mode);
        memory.set(0xFF44, amt);
    }

    fn get_lcdc_bit(memory: &mut Box<dyn Memory>, loc: u8) -> bool {
        return memory.get(0xFF40) & (1 << (loc)) > 0;
    }

    fn get_lcdc_control_operation(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 7)
    }

    fn get_lcdc_window_tile_select(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 6)
    }

    fn get_lcdc_window_on(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 5)
    }

    fn get_lcdc_tile_data(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 4)
    }

    fn get_lcdc_tile_map(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 3)
    }

    fn get_lcdc_big_sprite(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 2)
    }

    fn get_lcdc_sprite_display(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 1)
    }

    fn get_lcdc_bg_window_on(memory: &mut Box<dyn Memory>) -> bool{
        GPU::get_lcdc_bit(memory, 0)
    }

    fn get_lyc_ly_eq_set(memory: &mut Box<dyn Memory>) -> bool {
        memory.get(0xFF41) & (1 << 6) > 0
    }

    fn update_stat(&self, memory: &mut Box<dyn Memory>) {
        let mut val = 0;

        if GPU::get_lyc_ly_eq_set(memory) {
            val += 1 << 6
        }

        match self.mode {
            GpuState::SCAN_OAM => val += 1 << 5,
            GpuState::V_BLANK => val += 1 << 4,
            GpuState::H_BLANK => val += 1 << 3,
            GpuState::SCAN_VRAM => {}
        };

        if memory.get(0xFF44) == self.line {
            val += 1 << 2
        }

        match self.mode {
            GpuState::SCAN_OAM => val += 2,
            GpuState::V_BLANK => val += 1,
            GpuState::H_BLANK => val += 0,
            GpuState::SCAN_VRAM => val += 3
        };

        memory.set(0xFF41, val);

        if self.mode == GpuState::V_BLANK {
            memory.setInterruptFlag(0);
        }
    }

    fn draw_line(&mut self, memory: &mut Box<dyn Memory>, line: u8){
        let map_loc = match GPU::get_lcdc_tile_map(memory) {
            true  => 0x9C00 as usize,
            false => 0x9800 as usize
        };
        let inner_line = (line as i32 + self.get_y_offset(memory)) % 0x1000;
        let x_offset = self.get_x_offset(memory);
        let tile_loc = match GPU::get_lcdc_tile_data(memory) {
            true  => 0x8000 as i32,
            false => 0x9000 as i32
        };

        let palet = memory.get(0xFF47);

        for x in 0..(160 / 8) {
            //println!("{},{} -> {}",x,line, ((x as i16 * 8 + x_offset) / 8 + (inner_line / 8 as i16 * 32)+ map_loc as i16) as u16);
            let mut tile_id = memory.get(((x as i32 * 8 + x_offset) / 8 + (inner_line / 8 as i32 * 32)+ map_loc as i32) as u16) as i32;

            if !GPU::get_lcdc_tile_data(memory) {
                if tile_id > 127 {
                    tile_id = tile_id - 256;
                }
            }


            //tile_id = 0;

            let tile_low = memory.get((tile_id * 2 * 8 + (inner_line as i32 * 2 % 16) + tile_loc) as u16);
            let tile_high = memory.get((tile_id * 2 * 8 + 1 + (inner_line as i32 * 2 % 16) + tile_loc) as u16);

            if x == 0{
            //println!("Tile is {} -> {:x?},{:x?}", tile_id, tile_low, tile_high);
        //    println!("Locations are {}, {}", (tile_id * 2 * 8 + (inner_line as usize % 8) + tile_loc) as u16,
        //(tile_id * 2 * 8 + 1 + (inner_line as usize % 8) + tile_loc) as u16);
        }

            for xi in 0..8 {
                let t_low = (tile_low >> (8-xi - 1)) & 0x1;
                let t_high = (tile_high >> (8-xi - 1)) & 0x1;
                if x == 0{
                    //println!("{},{}", t_low, t_high);
                }

                let t_res = t_low + t_high * 2;
                //println!("{:x?}", palet);
                let palet_loc = (palet >> (t_res * 2)) % 0x04;

                let mut col = match palet_loc {
                    3 => 0,
                    2 => 82,
                    1 => 173,
                    0 => 255,
                    _ => panic!("Bad pallet value {}", palet_loc)
                };

                if tile_id != 0x20 {
                    //col = 173;
                }

                self.lcd[line as usize][(x * 8 + xi) as usize] = col;
            }

            if x == 0 {
                //println!("Resolved to ");
                for xi in 0..8{
                //    println!("{}", self.lcd[line as usize][(x * 8 + xi) as usize]);
                }
            }
        }

        for sprite in 0..40 {
            self.draw_line_sprite(memory, line, sprite, x_offset, inner_line);
        }
    }

    fn draw_line_sprite(&mut self, memory: &mut Box<dyn Memory>, line: u8, sprite: u16, x:i32, y:i32) {
        let oam_loc = (0xFE00 + sprite * 4) as u16;

        let sprite_y_coord = memory.get(oam_loc) as i32 - 16;

        let sprite_x_coord = memory.get(oam_loc + 1) as i32 - 8;

        if sprite_y_coord < y || sprite_y_coord > y + 8 {
            //println!("skip!");
            return;
        }

        //println!("{} -> {},{}", sprite, sprite_x_coord, sprite_y_coord);
        //println!("{},{}", x,y);

        for i in 0..8 {
            let target = ((sprite_x_coord + i - x) as usize);
            if target < 0 || target >= 160 {
                continue;
            }
            self.lcd[line as usize][target] = 25 * i as u8;
        }
    }

    fn get_y_offset(&mut self, memory: &mut Box<dyn Memory>) -> i32 {
        memory.get(0xFF42) as i32 & 0xFF * 0
    }
    fn get_x_offset(&mut self, memory: &mut Box<dyn Memory>) -> i32 {
        memory.get(0xFF43) as i32 & 0xFF
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GpuState{
   SCAN_OAM,
   SCAN_VRAM,
   H_BLANK,
   V_BLANK
}
