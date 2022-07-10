use std::fmt;
use std::cmp;

use crate::engine::memory::Memory;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Debug)]
pub struct GPU {
    pub time: f32,
    pub line: u8,
    pub mode: GpuState,
    pub lcd: Vec<Vec<u8>>,
    pub time_to_draw: bool
}

impl GPU {
    pub fn make_gpu() -> GPU {
        return GPU {
            time: 0.0,
            line: 0,
            mode: GpuState::H_BLANK,
            lcd: vec![vec![0; 160]; 144],
            time_to_draw: true
        };
    }

    pub fn draw<C: sdl2::render::RenderTarget>(&mut self,
        canvas : &mut sdl2::render::Canvas<C>,
        width: u32,
        height: u32){

        if ! self.time_to_draw {
            return;
        }

        self.time_to_draw = false;

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
                        self.time_to_draw = true;
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
                }
            }
        };
    }

    fn set_lcdc_y(&mut self, memory: &mut Box<dyn Memory>, amt: u8){
        //println!("---Y frame is now {} ({:?})", amt, self.mode);
        memory.set(0xFF44, amt);
    }

    fn get_lcdc_bit(memory: &mut Box<dyn Memory>, loc: u8) -> bool {
        return (memory.get(0xFF40) & (1 << (loc))) > 0;
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

    fn get_lcdc_bg_on(memory: &mut Box<dyn Memory>) -> bool{
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

        let inner_line = (line as i32 + self.get_y_offset(memory)) % 0xFF;
        let x_offset = self.get_x_offset(memory);

        let map_loc = match GPU::get_lcdc_tile_map(memory) {
            true  => 0x9C00 as usize,
            false => 0x9800 as usize
        };
        let tile_loc = match GPU::get_lcdc_tile_data(memory) {
            true  => 0x8000 as i32,
            false => 0x9000 as i32
        };

        if GPU::get_lcdc_bg_on(memory) {
            self.draw_line_tiles(memory, 0, 160, line, tile_loc, x_offset, inner_line, map_loc);
        }

        if GPU::get_lcdc_window_on(memory) {
            let window_map_loc = match GPU::get_lcdc_window_tile_select(memory) {
                true  => 0x9C00 as usize,
                false => 0x9800 as usize
            };
            let window_tile_loc = match GPU::get_lcdc_tile_data(memory) {
                true  => 0x8000 as i32,
                false => 0x9000 as i32
            };

            if line >= memory.get(0xFF4A) {
                self.draw_line_tiles(memory, memory.get(0xFF4A) as i32 - 7, 160, line, window_tile_loc, 0, line as i32, window_map_loc);
            }
        }


        for sprite in 0..40 {
            self.draw_line_sprite(memory, line, sprite, 0, line as i32);
        }
    }

    fn draw_line_tiles(&mut self,
                        memory: &mut Box<dyn Memory>,
                        start_x: i32,
                        end_x: i32,
                        line: u8,
                        tile_loc: i32,
                        x_offset: i32,
                        inner_line: i32,
                        map_loc: usize) {
        let palet = memory.get(0xFF47);

        for x in cmp::max(start_x, 0)..((end_x / 8)+1){
            let mut tile_id = memory.get((((x as i32 * 8 + x_offset) & 0xFF) / 8 + (inner_line / 8 as i32 * 32) + map_loc as i32) as u16) as i32;

                if !GPU::get_lcdc_tile_data(memory) {
                if tile_id > 127 {
                    tile_id = tile_id - 256;
                }
            }

            let tile_low = memory.get((tile_id * 2 * 8 + (inner_line as i32 * 2 % 16) + tile_loc) as u16);
            let tile_high = memory.get((tile_id * 2 * 8 + 1 + (inner_line as i32 * 2 % 16) + tile_loc) as u16);

            for xi in 0..8 {
                let t_low = (tile_low >> (8-xi - 1)) & 0x1;
                let t_high = (tile_high >> (8-xi - 1)) & 0x1;
                let t_res = t_low + t_high * 2;
                let palet_loc = (palet >> (t_res * 2)) % 0x04;

                let mut col = match palet_loc {
                    3 => 0,
                    2 => 82,
                    1 => 173,
                    0 => 255,
                    _ => panic!("Bad pallet value {}", palet_loc)
                };
                if x * 8 + xi < x_offset % 8 {
                    continue;
                }
                let final_x = (x * 8 + xi - (x_offset % 8)) as usize;

                if final_x < 0 || final_x >=160 {
                    continue;
                }

                self.lcd[line as usize][final_x] = col;
            }
        }
    }

    fn draw_line_sprite(&mut self, memory: &mut Box<dyn Memory>, line: u8, sprite: u16, x:i32, y:i32) {
        let oam_loc = (0xFE00 + sprite * 4) as u16;

        let sprite_y_coord = memory.get(oam_loc) as i32 - 16 + 8;
        let sprite_x_coord = memory.get(oam_loc + 1) as i32 - 8;

        if sprite_y_coord < y || sprite_y_coord >= y + 8 {
            //not at a line to draw this
            return;
        }

        if sprite_y_coord == -8 && sprite_x_coord == -8 {
            //disabled
            return;
        }

        let pattern = memory.get(oam_loc + 2);
        let flags = memory.get(oam_loc + 3);

        let priority = flags & 0b10000000 == 0;
        let flip_y = (flags & 0b01000000) > 0;
        let flip_x = (flags & 0b00100000) > 0;
        let pallet_num = (flags & 0b00010000) > 0;

        //println!("{} -> {},{}", sprite, sprite_x_coord, sprite_y_coord);
        //println!("{},{}", x,y);

        let mut tile_id = pattern as u16; // memory.get((((x as i32 * 8 + x_offset) & 0xFF) / 8 + (inner_line / 8 as i32 * 32) + map_loc as i32) as u16) as i32;

        let mut sprite_line = 7 - (sprite_y_coord - line as i32) as u16;

        if flip_y {
            sprite_line = 7 - sprite_line;
        }

        let tile_low = memory.get((tile_id * 8 * 2 + sprite_line * 2 + 0x8000) as u16);
        let tile_high = memory.get((tile_id * 8 * 2 + sprite_line * 2 + 1 + 0x8000) as u16);

        let palet = if !pallet_num {memory.get(0xFF48)} else {memory.get(0xFF49)};
        for xi in 0..8 {
            let target = ((sprite_x_coord + if flip_x {7 - xi} else {xi}) as usize);
            if target < 0 || target >= 160 {
                continue;
            }

            println!("{}, {}", priority, self.lcd[line as usize][target]);
            if !priority && self.lcd[line as usize][target] != 255 {
                continue;
            }

            let true_xi = 8-xi - 1;

            let t_low = (tile_low >> (true_xi)) & 0x1;
            let t_high = (tile_high >> (true_xi)) & 0x1;
            let t_res = t_low + t_high * 2;
            let palet_loc = (palet >> (t_res * 2)) % 0x04;

            if t_res == 0 {
                continue;
            }

            let mut col = match palet_loc {
                3 => 0,
                2 => 82,
                1 => 173,
                0 => 255,
                _ => panic!("Bad pallet value {}", palet_loc)
            };

            self.lcd[line as usize][target] = col;
        }
    }

    fn get_y_offset(&mut self, memory: &mut Box<dyn Memory>) -> i32 {
        memory.get(0xFF42) as i32 & 0xFF
    }
    fn get_x_offset(&mut self, memory: &mut Box<dyn Memory>) -> i32 {
        memory.get(0xFF43) as i32 & 0xFF
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GpuState {
   SCAN_OAM,
   SCAN_VRAM,
   H_BLANK,
   V_BLANK
}
