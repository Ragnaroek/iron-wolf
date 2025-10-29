#[cfg(feature = "tracing")]
use tracing::instrument;

use std::mem;
use std::sync::atomic::AtomicUsize;

use vga::input::MouseButton;
use vga::{CRTReg, SCReg, VGA, input::NumCode};

use crate::assets::{Font, GAMEPAL, Graphic, GraphicNum, Music, SoundName, TileData, WolfVariant};
use crate::config::WolfConfig;
use crate::def::{Assets, Button, NUM_BUTTONS, NUM_MOUSE_BUTTONS, ObjType};
use crate::draw::RayCast;
use crate::loader::Loader;
use crate::play::ProjectionConfig;
use crate::sd::Sound;
use crate::start::quit;
use crate::time::{self, Ticker, get_count};
use crate::vl;

pub const SCREENBWIDE: usize = 80;
pub const SCREEN_SIZE: usize = SCREENBWIDE * 208;

pub const PAGE_1_START: usize = 0;
pub const PAGE_2_START: usize = SCREEN_SIZE;
pub const PAGE_3_START: usize = SCREEN_SIZE * 2;
pub const FREE_START: usize = SCREEN_SIZE * 3;

static PIXMASKS: [u8; 4] = [1, 2, 4, 8];
static LEFTMASKS: [u8; 4] = [15, 14, 12, 8];
static RIGHTMASKS: [u8; 4] = [1, 3, 7, 15];

// Indexes into the Input.dir_scan array for the up, down, left, right buttons
pub const DIR_SCAN_NORTH: usize = 0;
pub const DIR_SCAN_EAST: usize = 1;
pub const DIR_SCAN_SOUTH: usize = 2;
pub const DIR_SCAN_WEST: usize = 3;

#[derive(PartialEq)]
pub enum InputMode {
    Player,
    DemoPlayback,
}

#[derive(PartialEq)]
pub enum ControlDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    None,
}

pub struct ControlInfo {
    pub button_0: bool,
    pub button_1: bool,
    pub button_2: bool,
    pub button_3: bool,
    pub dir: ControlDirection,
}

pub struct Input {
    pub demo_buffer: Option<Vec<u8>>,
    pub demo_ptr: usize,
    pub mode: InputMode,
    pub mouse_enabled: bool,
    pub joystick_enabled: bool,
    pub button_scan: [NumCode; NUM_BUTTONS],
    pub button_mouse: [Button; NUM_MOUSE_BUTTONS],
    pub dir_scan: [NumCode; 4],
}

pub struct RenderContext {
    pub vga: VGA,
    pub ticker: time::Ticker,
    linewidth: usize,
    bufferofs: AtomicUsize,
    displayofs: AtomicUsize,
    pub graphics: Vec<Graphic>,
    pub fonts: Vec<Font>,
    pub tiles: TileData,
    pub texts: Vec<String>,
    pub assets: Assets,
    pub variant: &'static WolfVariant,
    pub input: Input,
    pub projection: ProjectionConfig,
    pub cast: RayCast,
    pub sound: Sound,

    prev_input: Option<Input>,
}

pub enum FizzleFadeAbortable<'a> {
    Yes(&'a Input),
    No,
}

impl Input {
    pub fn init_player(wolf_config: &WolfConfig) -> Input {
        Input {
            mouse_enabled: true,
            joystick_enabled: false,
            demo_buffer: None,
            demo_ptr: 0,
            mode: InputMode::Player,
            button_scan: wolf_config.button_scan.clone(),
            button_mouse: wolf_config.button_mouse.clone(),
            dir_scan: wolf_config.dir_scan.clone(),
        }
    }

    pub fn init_demo_playback(demo_buffer: Vec<u8>) -> Input {
        Input {
            mouse_enabled: false,
            joystick_enabled: false,
            demo_buffer: Some(demo_buffer),
            demo_ptr: 0,
            mode: InputMode::DemoPlayback,
            button_scan: [NumCode::None; NUM_BUTTONS],
            button_mouse: [Button::NoButton; NUM_MOUSE_BUTTONS],
            dir_scan: [NumCode::None; 4],
        }
    }

    pub fn wait_user_input(&mut self, vga: &mut VGA, ticker: &Ticker, delay: u64) -> bool {
        let last_count = get_count(&ticker.time_count);
        {
            vga.input_monitoring().clear_keyboard();
        }
        loop {
            if vga.input_monitoring().any_key_pressed() {
                return true;
            }

            if get_count(&ticker.time_count) - last_count > delay {
                break;
            }
            if vga.draw_frame() {
                break;
            }
        }
        false
    }

    pub fn clear_keys_down(&self, vga: &mut VGA) {
        if self.mode == InputMode::Player {
            let mut input = vga.input_monitoring();
            input.clear_keyboard();
            input.keyboard.last_scan = NumCode::None;
            input.keyboard.last_ascii = '\0';
        }
    }
}

impl RenderContext {
    pub fn init(
        vga: VGA,
        ticker: time::Ticker,
        graphics: Vec<Graphic>,
        fonts: Vec<Font>,
        tiles: TileData,
        texts: Vec<String>,
        assets: Assets,
        variant: &'static WolfVariant,
        input: Input,
        projection: ProjectionConfig,
        cast: RayCast,
        sound: Sound,
    ) -> RenderContext {
        RenderContext {
            vga,
            ticker,
            linewidth: 80,
            bufferofs: AtomicUsize::new(PAGE_1_START),
            displayofs: AtomicUsize::new(PAGE_1_START),
            graphics,
            fonts,
            tiles,
            texts,
            assets,
            variant,
            input,
            projection,
            cast,
            sound,
            prev_input: None,
        }
    }

    pub fn display(&mut self) {
        if self.vga.draw_frame() {
            quit(None);
        }
    }

    pub fn use_demo_input(&mut self, input: Input) {
        let prev_input = mem::replace(&mut self.input, input);
        self.prev_input = Some(prev_input);
    }

    pub fn restore_player_input(&mut self) -> Result<(), String> {
        if self.prev_input.is_none() {
            return Err("on input to restore".to_string());
        }

        let prev = mem::replace(&mut self.prev_input, None);
        self.input = prev.expect("previous input");
        Ok(())
    }

    pub fn tile_to_screen(&mut self, tile: usize, width: usize, height: usize, x: usize, y: usize) {
        let width_bytes = width >> 2;
        let mut mask = 1 << (x & 3);
        let mut src_ix = 0;
        let dst_offset = self.y_offset(y) + (x >> 2);
        let mut dst_ix;
        for _ in 0..4 {
            self.vga.set_sc_data(SCReg::MapMask, mask);
            mask <<= 1;
            if mask == 16 {
                mask = 1;
            }
            dst_ix = dst_offset;
            for _ in 0..height {
                self.vga.write_mem_chunk(
                    dst_ix,
                    &self.tiles.tile8[tile][src_ix..(src_ix + width_bytes)],
                );
                dst_ix += self.linewidth;
                src_ix += width_bytes;
            }
        }
    }

    pub fn graphic_to_screen(&mut self, pic_num: usize, x: usize, y: usize) {
        let width_bytes = self.graphics[pic_num].width >> 2;
        let mut mask = 1 << (x & 3);
        let mut src_ix = 0;
        let dst_offset = self.y_offset(y) + (x >> 2);
        let mut dst_ix;
        for _ in 0..4 {
            self.vga.set_sc_data(SCReg::MapMask, mask);
            mask <<= 1;
            if mask == 16 {
                mask = 1;
            }
            dst_ix = dst_offset;
            for _ in 0..self.graphics[pic_num].height {
                self.vga.write_mem_chunk(
                    dst_ix,
                    &self.graphics[pic_num].data[src_ix..(src_ix + width_bytes)],
                );
                dst_ix += self.linewidth;
                src_ix += width_bytes;
            }
        }
    }

    pub fn y_offset(&self, y: usize) -> usize {
        self.bufferofs.load(std::sync::atomic::Ordering::Relaxed) + y * self.linewidth
    }

    pub fn set_buffer_offset(&self, offset: usize) {
        self.bufferofs
            .store(offset, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn buffer_offset(&self) -> usize {
        self.bufferofs.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg_attr(feature = "tracing", instrument(skip_all))]
    pub async fn activate_buffer(&self, offset: usize) {
        let addr_parts = offset.to_le_bytes();
        self.vga.set_crt_data(CRTReg::StartAdressLow, addr_parts[0]);
        self.vga
            .set_crt_data(CRTReg::StartAdressHigh, addr_parts[1]);

        self.displayofs
            .store(offset, std::sync::atomic::Ordering::Relaxed);
    }

    // displayofs in the orginal
    pub fn active_buffer(&self) -> usize {
        self.displayofs.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn write_mem(&self, offset: usize, v_in: u8) {
        self.vga.write_mem(offset, v_in)
    }

    pub fn set_mask(&mut self, m: u8) {
        self.vga.set_sc_data(SCReg::MapMask, m)
    }

    pub fn bar(&mut self, x: usize, y: usize, width: usize, height: usize, color: u8) {
        let leftmask = LEFTMASKS[x & 3];
        let rightmask = RIGHTMASKS[(x + width - 1) & 3];
        let midbytes = ((x as i32 + (width as i32) + 3) >> 2) - (x as i32 >> 2) - 2;

        let mut dest = self.y_offset(y) + (x >> 2);

        if midbytes < 0 {
            self.vga.set_sc_data(SCReg::MapMask, leftmask & rightmask);
            for _ in 0..height {
                self.vga.write_mem(dest, color);
                dest += self.linewidth;
            }
        } else {
            for _ in 0..height {
                let linedelta = self.linewidth - (midbytes as usize + 1);
                self.vga.set_sc_data(SCReg::MapMask, leftmask);
                self.vga.write_mem(dest, color);
                dest += 1;

                self.vga.set_sc_data(SCReg::MapMask, 0xFF);
                for _ in 0..midbytes {
                    self.vga.write_mem(dest, color);
                    dest += 1;
                }
                self.vga.set_sc_data(SCReg::MapMask, rightmask);
                self.vga.write_mem(dest, color);

                dest += linedelta;
            }
        }

        self.vga.set_sc_data(SCReg::MapMask, 0xFF);
    }

    pub fn hlin(&mut self, x: usize, y: usize, width: usize, color: u8) {
        let xbyte = x >> 2;
        let leftmask = LEFTMASKS[x & 3];
        let rightmask = RIGHTMASKS[(x + width - 1) & 3];
        let midbytes: i32 = ((x + width + 3) >> 2) as i32 - xbyte as i32 - 2;

        let mut dest = self.y_offset(y) + xbyte;
        if midbytes < 0 {
            self.vga.set_sc_data(SCReg::MapMask, leftmask & rightmask);
            self.vga.write_mem(dest, color);
        } else {
            self.vga.set_sc_data(SCReg::MapMask, leftmask);
            self.vga.write_mem(dest, color);
            dest += 1;

            self.vga.set_sc_data(SCReg::MapMask, 0xFF);
            for _ in 0..midbytes {
                self.vga.write_mem(dest, color);
                dest += 1;
            }

            self.vga.set_sc_data(SCReg::MapMask, rightmask);
            self.vga.write_mem(dest, color);
        }

        self.vga.set_sc_data(SCReg::MapMask, 0xFF);
    }

    pub fn vlin(&mut self, x: usize, y: usize, height: usize, color: u8) {
        let mask = PIXMASKS[x & 3];
        self.vga.set_sc_data(SCReg::MapMask, mask);

        let mut dest = self.y_offset(y) + (x >> 2);
        let mut h = height;
        while h > 0 {
            self.vga.write_mem(dest, color);
            dest += self.linewidth;
            h -= 1;
        }

        self.vga.set_sc_data(SCReg::MapMask, 0xFF);
    }

    pub fn plot(&mut self, x: usize, y: usize, color: u8) {
        let mask = PIXMASKS[x & 3];
        self.vga.set_sc_data(SCReg::MapMask, mask);
        let dest = self.y_offset(y) + (x >> 2);
        self.vga.write_mem(dest, color);
    }

    pub fn pic_lump(&mut self, x: usize, y: usize, lump: usize) {
        let pic_num = lump - self.variant.start_pics;
        self.graphic_to_screen(pic_num, x & !7, y);
    }

    pub fn pic(&mut self, x: usize, y: usize, graph_num: GraphicNum) {
        self.pic_lump(x, y, self.variant.graphic_lump_map[graph_num as usize]);
    }

    pub fn debug_pic(&mut self, data: &Vec<u8>) {
        for tex_y in 0..64 {
            for tex_x in 0..64 {
                self.plot(tex_x, tex_y, data[tex_x * 64 + tex_y]);
            }
        }
    }

    pub async fn fade_out(&self) {
        vl::fade_out(&self.vga, 0, 255, 0, 0, 0, 30).await;
    }

    pub async fn fade_in(&self) {
        vl::fade_in(&self.vga, 0, 255, GAMEPAL, 30).await;
    }

    #[cfg_attr(feature = "tracing", instrument(skip_all))]
    pub fn fizzle_fade<'a>(
        &mut self,
        source: usize,
        dest: usize,
        width: usize,
        height: usize,
        frames: usize,
        abortable: FizzleFadeAbortable<'a>,
    ) -> bool {
        let (page_delta, _) = dest.overflowing_sub(source);
        let mut rnd_val: u32 = 1;
        let pix_per_frame = 64000 / frames;

        self.ticker.clear_count();
        let mut frame = 0;
        loop {
            self.vga.draw_frame();

            if let FizzleFadeAbortable::Yes(_) = abortable {
                if self.check_ack() {
                    return true;
                }
            }

            for _ in 0..pix_per_frame {
                let mut ax: u32 = rnd_val & 0xFFFF;
                let mut dx: u32 = (rnd_val >> 16) & 0xFFFF;

                let (bx, _) = ax.overflowing_sub(1);
                let y = (bx & 0xFF) as usize; // low 8 bits - 1 = y coordinate
                let x = (((ax & 0xFF00) >> 8) | (dx << 8)) as usize; // next 9 bits = x coordinate

                // advance to next random element
                let carry_dx = dx & 0x01;
                dx >>= 1;
                let carry_ax = ax & 0x1;
                ax = (ax >> 1) | (carry_dx << 15); // rcr on ax
                if carry_ax == 1 {
                    dx ^= 0x0001;
                    ax ^= 0x2000;
                }

                rnd_val = ax | (dx << 16);

                if x > width || y > height {
                    continue;
                }

                let draw_ofs = self.y_offset(y) + (x >> 2);
                self.vga
                    .set_gc_data(vga::GCReg::ReadMapSelect, (x & 3) as u8);
                let src_pix = self.vga.read_mem(draw_ofs);

                let mask = PIXMASKS[x & 3];
                self.vga.set_sc_data(SCReg::MapMask, mask);
                let (dst, _) = draw_ofs.overflowing_add(page_delta);
                self.vga.write_mem(dst, src_pix);

                if rnd_val == 1 {
                    return false;
                }
            }

            frame += 1;
            // TODO don't do busy wait. wait for next count
            while self.ticker.get_count() < frame {} // don't go too fast
        }
    }

    // input handling

    pub fn wait_user_input(&mut self, delay: u64) -> bool {
        self.input
            .wait_user_input(&mut self.vga, &self.ticker, delay)
    }

    pub fn start_ack(&mut self) {
        let mut input = self.vga.input_monitoring();
        input.clear_keyboard();
        input.clear_mouse();
        // TODO clear joystick buttons
    }

    pub fn ack(&mut self) -> bool {
        self.wait_user_input(u64::MAX)
    }

    pub fn check_ack(&mut self) -> bool {
        self.vga.input_monitoring().any_key_pressed()
    }

    pub fn clear_keys_down(&mut self) {
        self.input.clear_keys_down(&mut self.vga);
    }

    pub fn key_pressed(&mut self, code: NumCode) -> bool {
        self.vga.input_monitoring().key_pressed(code)
    }

    pub fn last_scan(&mut self) -> NumCode {
        self.vga.input_monitoring().keyboard.last_scan
    }

    // Returns the 0 char if nothing is set
    pub fn last_ascii(&mut self) -> char {
        self.vga.input_monitoring().keyboard.last_ascii
    }

    pub fn clear_last_scan(&mut self) {
        self.vga.input_monitoring().keyboard.last_scan = NumCode::None;
    }

    pub fn clear_last_ascii(&mut self) {
        self.vga.input_monitoring().keyboard.last_ascii = '\0';
    }

    pub fn mouse_button_pressed(&mut self, button: MouseButton) -> bool {
        self.vga.input_monitoring().mouse_button_pressed(button)
    }

    pub fn read_control(&mut self, ci: &mut ControlInfo) {
        if self.key_pressed(NumCode::UpArrow) {
            ci.dir = ControlDirection::North;
        } else if self.key_pressed(NumCode::DownArrow) {
            ci.dir = ControlDirection::South;
        } else if self.key_pressed(NumCode::LeftArrow) {
            ci.dir = ControlDirection::West;
        } else if self.key_pressed(NumCode::RightArrow) {
            ci.dir = ControlDirection::East;
        } else {
            ci.dir = ControlDirection::None;
        }
    }

    // sound helpers

    pub fn play_sound(&mut self, sound: SoundName) -> bool {
        self.sound.play_sound(sound, &self.assets) // TODO rename back to play_sound
    }

    pub fn force_play_sound(&mut self, sound: SoundName) -> bool {
        self.sound.force_play_sound(sound, &self.assets)
    }

    pub fn play_music(&mut self, track: Music, loader: &dyn Loader) {
        self.sound.play_music(track, &self.assets, loader);
    }

    pub fn play_sound_loc_tile(&mut self, sound: SoundName, tile_x: usize, tile_y: usize) {
        self.sound
            .play_sound_loc_tile(sound, &self.assets, &self.cast, tile_x, tile_y)
    }

    pub fn play_sound_loc_actor(&mut self, sound: SoundName, obj: &ObjType) {
        self.sound
            .play_sound_loc_actor(sound, &self.assets, &self.cast, obj);
    }
}
