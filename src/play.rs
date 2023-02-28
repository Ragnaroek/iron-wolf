#[cfg(test)]
#[path = "./play_test.rs"]
mod play_test;

use crate::def::{new_fixed, new_fixed_u32, new_fixed_i32};

use super::vga_render::Renderer;
use super::def::{GameState, WeaponType, Assets, Level, ObjKey, LevelState, Control, Fixed, GLOBAL1, TILEGLOBAL, MAP_SIZE, ANGLES, ANGLE_QUAD};
use super::assets::{GraphicNum, face_pic, num_pic, weapon_pic};
use libiw::gamedata::Texture;
use vgaemu::input::NumCode;
use super::input;
use super::time;
use super::vga_render;
use super::game::{setup_game_level, TILESHIFT, ANGLE_45, ANGLE_180};
use super::wolf_hack::fixed_mul;

//TODO separate draw.c stuff from play.c stuff in here

const MIN_DIST : i32 = 0x5800;

const STATUS_LINES : usize = 40;
const HEIGHT_RATIO : f32 = 0.5;
const SCREEN_WIDTH : usize = 80;

const SCREEN_WIDTH_PIXEL : usize = 640;
const SCREEN_HEIGHT_PIXEL : usize = 480;

const FRAC_BITS : usize = 16;
const FRAC_UNIT : usize = 1<<FRAC_BITS;

const PI : f32 = 3.141592657;

const ANGLE_TO_FINE_SHIFT : u32 = 19;
const FINE_ANGLES : usize = 8192;
const ANG90 : usize = FINE_ANGLES/4;
const ANG180 : usize = ANG90*2;
const ANG270 : usize = ANG90*3;
const ANG360 : usize = ANG90*4;

const NUM_FINE_TANGENTS : usize = FINE_ANGLES/2 + ANG180;
const NUM_FINE_SINES : usize = FINE_ANGLES+ANG90;

const VIEW_GLOBAL : usize = 0x10000;
const FOCAL_LENGTH : i32 = 0x5700;

const TEXTURE_WIDTH : usize = 64;
const TEXTURE_HEIGHT : usize = 64;

const RAD_TO_INT : f64 = FINE_ANGLES as f64 / 2.0 / std::f64::consts::PI;

const RUN_MOVE : i32 = 70;
const BASE_MOVE: i32 = 35;

static SCREENLOC : [usize; 3] = [vga_render::PAGE_1_START, vga_render::PAGE_2_START, vga_render::PAGE_3_START];

static VGA_CEILING : [u8; 60] = [	
	0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0xbf,
	0x4e,0x4e,0x4e,0x1d,0x8d,0x4e,0x1d,0x2d,0x1d,0x8d,
	0x1d,0x1d,0x1d,0x1d,0x1d,0x2d,0xdd,0x1d,0x1d,0x98,
   
	0x1d,0x9d,0x2d,0xdd,0xdd,0x9d,0x2d,0x4d,0x1d,0xdd,
	0x7d,0x1d,0x2d,0x2d,0xdd,0xd7,0x1d,0x1d,0x1d,0x2d,
	0x1d,0x1d,0x1d,0x1d,0xdd,0xdd,0x7d,0xdd,0xdd,0xdd
];

pub struct ProjectionConfig {
	view_width: usize,
	view_height: usize,
	screenofs: usize,
    height_numerator: i32,
	pixelangle: Vec<i32>,
    sines: Vec<Fixed>,
    fine_tangents: [i32; NUM_FINE_TANGENTS],
    focal_length_y: i32,
}

impl ProjectionConfig {
    pub fn sin(&self, ix: usize) -> Fixed {
        self.sines[ix]
    }

    pub fn cos(&self, ix: usize) -> Fixed {
        self.sines[ix+ANGLE_QUAD as usize]
    }
}

pub fn new_game_state() -> GameState {
	GameState {
		map_on: 0,
		score: 0,
		lives: 3,
		health: 100,
		ammo: 8,
		keys: 0,
		weapon: WeaponType::Pistol,
		face_frame: 0,
		episode: 0
	}
}

pub fn new_projection_config(view_size: usize) -> ProjectionConfig {
	let view_width = (view_size * 16) & !15;
	let view_height = ((((view_size * 16) as f32 * HEIGHT_RATIO) as u16) & !1) as usize;
	let screenofs = (200-STATUS_LINES-view_height)/2*SCREEN_WIDTH+(320-view_width)/8;
    let half_view = view_width/2;
    let projection_fov = VIEW_GLOBAL as f64;

    let face_dist = FOCAL_LENGTH + MIN_DIST;

	let pixelangle = calc_pixelangle(view_width, projection_fov, face_dist as f64);
    let sines = calc_sines();
    let fine_tangents = calc_fine_tangents();

    let center_x = view_width/2-1;
    let y_aspect = fixed_mul(new_fixed_u32((320<<FRAC_BITS)/200), new_fixed_u32(((SCREEN_HEIGHT_PIXEL<<FRAC_BITS)/SCREEN_WIDTH_PIXEL) as u32));
    let focal_length_y = center_x as i32 * y_aspect.to_i32()/fine_tangents[FINE_ANGLES/2+(ANGLE_45 >> ANGLE_TO_FINE_SHIFT) as usize];
    let scale = half_view as i32 * face_dist/(VIEW_GLOBAL as i32/2);

    let height_numerator = (TILEGLOBAL*scale)>>6;

	ProjectionConfig {
		view_width,
		view_height,
		screenofs,
        height_numerator,
		pixelangle,
        sines,
        fine_tangents,
        focal_length_y,
	}
}

fn calc_fine_tangents() -> [i32; NUM_FINE_TANGENTS] {
    let mut tangents = [0; NUM_FINE_TANGENTS];

    for i in 0..FINE_ANGLES/8 {
        let tang = ((i as f64 +0.5)/RAD_TO_INT).tan();
        let t = (tang * FRAC_UNIT as f64) as i32;
        tangents[i] = t;
        tangents[i+FINE_ANGLES/2] = t;
        tangents[FINE_ANGLES/4-1-i] = ((1.0/tang)*FRAC_UNIT as f64) as i32;
        tangents[FINE_ANGLES/4+i]=-tangents[FINE_ANGLES/4-1-i];
        tangents[FINE_ANGLES/2-1-i]=-tangents[i];
    }
    let mut src = 0;
    for i in FINE_ANGLES/2..FINE_ANGLES {
        tangents[i] = tangents[src];
        src += 1;
    }

    tangents
}

fn calc_pixelangle(view_width: usize, projection_fov: f64, face_dist: f64) -> Vec<i32> {
	let half_view = view_width/2;

	let mut pixelangles = vec![0; view_width as usize]; 

	for i in 0..(half_view+1) {
		let tang = (((i as f64 + 0.5) * projection_fov) / view_width as f64) / face_dist;
		let angle = (tang.atan() * RAD_TO_INT) as i32;
        pixelangles[half_view-i] = angle;
		pixelangles[half_view-1+i] = -angle;
	}

	pixelangles
}

fn calc_sines() -> Vec<Fixed> {
    //TODO_VANILLA +1?? Bug in the original? does it write outside they array there in the original?
    let mut sines: Vec<Fixed> = vec![new_fixed(0, 0); ANGLES+ANGLE_QUAD+1]; 

    let mut angle : f32 = 0.0;
    let angle_step = PI/2.0/ANGLE_QUAD as f32;
    for i in 0..=ANGLE_QUAD {
        let value : u32 = (GLOBAL1 as f32 * angle.sin()) as u32;
        let v_fixed = new_fixed_u32(value.min(65535));
        let v_fixed_neg = new_fixed_u32(value | 0x80000000u32);
        sines[i] = v_fixed;
        sines[i+ANGLES] = v_fixed;
        sines[ANGLES/2-i] = v_fixed;
        sines[ANGLES-i] = v_fixed_neg;
        sines[ANGLES/2+i] = v_fixed_neg;
        angle += angle_step;   
    }
    sines
}

pub fn game_loop(ticker: &time::Ticker, rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig, assets: &Assets) {
	
    let game_state = new_game_state();
    
    draw_play_screen(&game_state, rdr, prj);
	
	let mut level = setup_game_level(prj, game_state.map_on, assets).unwrap();

	//TODO StartMusic
	//TODO PreloadGraphics

	draw_level(&game_state, rdr);
    
    rdr.fade_in();

	play_loop(ticker, &mut level, &game_state, rdr, input, prj, assets);

	//TODO Go to next level (gamestate.map_on+=1)

	input.wait_user_input(time::TICK_BASE*1000);
}

fn play_loop(ticker: &time::Ticker, level_state: &mut LevelState, game_state: &GameState, rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig, assets: &Assets) {
	//TODO A lot to do here (clear palette, poll controls, prepare world)
    loop {
        level_state.control = poll_controls(ticker, input);

        for i in 0..level_state.actors.len() {
            do_actor(ObjKey(i), level_state, prj);
        }
        
	    three_d_refresh(game_state, level_state, rdr, prj, assets);

        let offset_prev = rdr.buffer_offset();
        for i in 0..3 {
            rdr.set_buffer_offset(SCREENLOC[i]);
        } 
        rdr.set_buffer_offset(offset_prev);
    }
}

fn do_actor(k: ObjKey, level_state: &mut LevelState, prj: &ProjectionConfig) {
    //TODO do ob->ticcount part from DoActor here
    let may_think = level_state.obj(k).state.think;
    if let Some(think) = may_think {
        think(k, level_state, prj)
    }

    //TODO remove obj if state becomes None
    //TODO return if flag = FL_NEVERMARK (the player obj always has this flag)
    //TODO Impl think for player = T_Player function and supply the corret (mutable) args
}

fn three_d_refresh(game_state: &GameState, level_state: &LevelState, rdr: &dyn Renderer, prj: &ProjectionConfig, assets: &Assets) {

    rdr.set_buffer_offset(rdr.buffer_offset() + prj.screenofs);

	clear_screen(game_state, rdr, prj);
    wall_refresh(level_state, rdr, prj, assets);

	rdr.set_buffer_offset(rdr.buffer_offset() - prj.screenofs);
    rdr.activate_buffer(rdr.buffer_offset());

    //set offset to buffer for next frame
    let mut next_offset = rdr.buffer_offset() + vga_render::SCREEN_SIZE;
    if next_offset > vga_render::PAGE_3_START {
        next_offset = vga_render::PAGE_1_START;
    }
    rdr.set_buffer_offset(next_offset);
}

#[derive(Debug)]
enum Hit {
    VerticalBorder,
    HorizontalBorder,
    VerticalWall,
    HorizontalWall,
}

struct RayCast {
    tile_hit: u16,
    hit: Hit, 
    y_tile: i32,
    x_tile: i32,
    y_intercept: i32,
    x_intercept: i32,
    x_spot: [i32; 2],
    y_spot: [i32; 2],
    x_tilestep: i32,
    y_tilestep: i32,
    x_step: i32,
    y_step: i32,
}
enum Dir {
    Vertical,
    Horizontal,
}

impl RayCast {
    fn cast(&mut self, level: &Level) {
        let mut dir = Dir::Vertical;
        loop {
            match dir {
                Dir::Vertical => {
                    if self.y_tilestep == -1 && (self.y_intercept>>16)<=self.y_tile || self.y_tilestep == 1 && (self.y_intercept>>16)>=self.y_tile {
                        dir = Dir::Horizontal
                    }
                }
                Dir::Horizontal => {
                    if self.x_tilestep == -1 && (self.x_intercept>>16)<=self.x_tile || self.x_tilestep == 1 && (self.x_intercept>>16)>=self.x_tile {
                        dir = Dir::Vertical
                    }
                } 
            }

            if match dir {
                Dir::Vertical => self.vert_entry(level),
                Dir::Horizontal => self.horiz_entry(level),
            } {
                break;
            }
        }
    }

    fn vert_entry(&mut self, level: &Level) -> bool {
        if self.y_intercept > (MAP_SIZE*65536-1) as i32 || self.x_tile >= MAP_SIZE as i32 {
            if self.x_tile < 0 {
                self.x_intercept = 0;
                self.x_tile = 0;
            } else if self.x_tile >= MAP_SIZE as i32 {
                self.x_intercept = (MAP_SIZE as i32) << TILESHIFT;
                self.x_tile = MAP_SIZE as i32 - 1;
            } else {
                self.x_tile = self.x_intercept >> TILESHIFT;
            }

            if self.y_intercept < 0 {
                self.y_intercept = 0;
                self.y_tile = 0;
            } else if self.y_intercept >= (MAP_SIZE as i32) << TILESHIFT {
                self.y_intercept = (MAP_SIZE as i32) << TILESHIFT;
                self.y_tile = MAP_SIZE as i32 - 1;
            }
            self.y_spot[0] = 0xffff;
            self.tile_hit = 0;
            self.hit = Hit::HorizontalBorder;
            return true;
        }
        if self.x_spot[0] >= MAP_SIZE as i32 || self.x_spot[1] >= MAP_SIZE as i32 {
            return true;
        }
        self.tile_hit = level.tile_map[self.x_spot[0] as usize][self.x_spot[1] as usize];
        if self.tile_hit != 0 {
            //TODO Ignored tile.offsetVertical and pushWall handling here!
            self.x_intercept = self.x_tile << TILESHIFT;
            self.y_tile = self.y_intercept >> TILESHIFT;
            self.hit = Hit::VerticalWall;
            return true;
        }
        self.x_tile += self.x_tilestep;
        self.y_intercept += self.y_step;
        self.x_spot[0] = self.x_tile;
        self.x_spot[1] = self.y_intercept >> 16;
        false
    }

    fn horiz_entry(&mut self, level: &Level) -> bool {
        if self.x_intercept > (MAP_SIZE*65536-1) as i32 || self.y_tile >= MAP_SIZE as i32  {
            if self.y_tile < 0 {
                self.y_intercept=0;
                self.y_tile=0;
            } else if self.y_tile >= MAP_SIZE as i32 {
                self.y_intercept = (MAP_SIZE as i32) << TILESHIFT;
                self.y_tile=MAP_SIZE as i32 - 1;
            } else {
                self.y_tile = self.y_intercept >> TILESHIFT;
            }

            if self.x_intercept < 0 {
                self.x_intercept = 0;
                self.x_tile = 0;
            } else if self.x_intercept >= (MAP_SIZE as i32) << TILESHIFT {
                self.x_intercept = (MAP_SIZE as i32) << TILESHIFT;
                self.x_tile = MAP_SIZE as i32 - 1;
            }

            self.x_spot[0] = 0xffff;
            self.tile_hit = 0;
            self.hit = Hit::VerticalBorder;
            return true;
        }
        if self.y_spot[0]>=MAP_SIZE as i32 || self.y_spot[1]>=MAP_SIZE as i32 {
            return true;
        }
        self.tile_hit = level.tile_map[self.y_spot[0] as usize][self.y_spot[1] as usize];
        if self.tile_hit != 0 {
            //TODO Ignored tile.offsetHorizontal and pushWall handling here!
            self.y_intercept = self.y_tile<<TILESHIFT;
            self.x_tile = self.x_intercept >> TILESHIFT;
            self.hit = Hit::HorizontalWall;
            return true;
        }
        //passhoriz
        self.y_tile += self.y_tilestep;
        self.x_intercept += self.x_step;
        self.y_spot[0] = self.x_intercept >> 16;
        self.y_spot[1] = self.y_tile;
        false
    }
}

fn wall_refresh(level_state: &LevelState, rdr: &dyn Renderer, prj: &ProjectionConfig, assets: &Assets) {
    let player = level_state.player();
    let view_angle = player.angle;
    let mid_angle = view_angle * (FINE_ANGLES as i32/ANGLES as i32);
    let view_sin = prj.sin((view_angle >> ANGLE_TO_FINE_SHIFT) as usize);
    let view_cos = prj.cos((view_angle >> ANGLE_TO_FINE_SHIFT) as usize);
    let view_x = player.x - fixed_mul(new_fixed_i32(FOCAL_LENGTH), view_cos).to_i32();
    let view_y = player.y + fixed_mul(new_fixed_i32(FOCAL_LENGTH), view_sin).to_i32();
    
    let focal_tx = view_x >> TILESHIFT;
    let focal_ty = view_y >> TILESHIFT;

    let view_tx = player.x >> TILESHIFT;
    let view_ty = player.y >> TILESHIFT;

    let x_partialdown = view_x&(TILEGLOBAL-1);
    let x_partialup = TILEGLOBAL-x_partialdown;
    let y_partialdown = view_y&(TILEGLOBAL-1);
    let y_partialup = TILEGLOBAL-y_partialdown;

    let mid_wallheight = prj.view_height;
    let lastside = -1;
    let view_shift = fixed_mul(new_fixed_i32(prj.focal_length_y), new_fixed_i32(prj.fine_tangents[(ANGLE_180 + player.pitch >> ANGLE_TO_FINE_SHIFT) as usize]));

    let mut x_partial = 0;
    let mut y_partial = 0;

    //asm_refresh / ray casting core loop
    let mut rc = RayCast{tile_hit: 0, hit: Hit::VerticalBorder, 
        x_intercept:0, y_intercept:0, 
        x_tile: 0, y_tile:0,
        x_tilestep: 0, y_tilestep: 0,
        x_step: 0, y_step: 0, 
        x_spot: [0, 0], y_spot: [0, 0]};

    for pixx in 0..prj.view_width {
        let mut angl=mid_angle as i32 + prj.pixelangle[pixx];
        if angl<0 {
            angl+=FINE_ANGLES as i32;
        }
        if angl>=ANG360 as i32 {
            angl-=FINE_ANGLES as i32;
        }
        if angl<ANG90 as i32 {
            rc.x_tilestep = 1;
            rc.y_tilestep = -1;
            rc.x_step = prj.fine_tangents[ANG90-1-angl as usize];
            rc.y_step = -prj.fine_tangents[angl as usize];
            x_partial = x_partialup;
            y_partial = y_partialdown;
        } else if angl<ANG180 as i32 {
            rc.x_tilestep = -1;
            rc.y_tilestep = -1;
            rc.x_step = -prj.fine_tangents[angl as usize -ANG90];
            rc.y_step = -prj.fine_tangents[ANG180-1-angl as usize];
            x_partial=x_partialdown;
            y_partial=y_partialdown;
        } else if angl<ANG270 as i32 {
            rc.x_tilestep = -1;
            rc.y_tilestep = 1;
            rc.x_step = -prj.fine_tangents[ANG270-1-angl as usize];
            rc.y_step = prj.fine_tangents[angl as usize - ANG180 as usize];
            x_partial=x_partialup;
            y_partial=y_partialup; 
        } else if angl<ANG360 as i32 {
            rc.x_tilestep = 1;
            rc.y_tilestep = 1;
            rc.x_step = prj.fine_tangents[angl as usize - ANG270];
            rc.y_step = prj.fine_tangents[ANG360-1-angl as usize];
            x_partial=x_partialup;
            y_partial=y_partialup;
        }
        rc.y_intercept = fixed_mul(new_fixed_i32(rc.y_step), new_fixed_i32(x_partial)).to_i32() + view_y;
        rc.x_tile = focal_tx+rc.x_tilestep;
        rc.x_spot[0] = rc.x_tile;
        rc.x_spot[1] = rc.y_intercept >> 16;
        rc.x_intercept = fixed_mul(new_fixed_i32(rc.x_step), new_fixed_i32(y_partial)).to_i32() + view_x;
        rc.y_tile = focal_ty + rc.y_tilestep;
        rc.y_spot[0] = rc.x_intercept>>16;
        rc.y_spot[1] = rc.y_tile;
        let tex_delta = 0;

        rc.cast(&level_state.level);
        
        let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, view_x, view_y, view_cos.to_i32(), view_sin.to_i32());

        let side = match rc.hit {
            Hit::HorizontalBorder|Hit::HorizontalWall => 0,
            Hit::VerticalBorder|Hit::VerticalWall => 1,
        };
       
        let post_src = match rc.hit {
            Hit::HorizontalBorder|Hit::HorizontalWall => (rc.x_intercept>>4)&0xFC0,
            Hit::VerticalBorder|Hit::VerticalWall => (rc.y_intercept>>4)&0xFC0,
        };

        let texture = if rc.tile_hit < 50 && rc.tile_hit > 0 {
            Some(&assets.textures[((rc.tile_hit - 1) * 2 + side) as usize])
        } else {
            //TODO totally only for test
            None
        };

        draw_scaled(pixx, post_src, height, prj.view_height as i32, texture, rdr);
    }
}

fn draw_scaled(x: usize, post_src: i32, height: i32, view_height: i32, texture: Option<&Texture>, rdr: &dyn Renderer) {
    //TODO use the exact copy statements as the compiled scalers do! (compare scaling code in the original with this => step_size and clamping)
    let line_height = if height > 512 {
        view_height
    } else {
        (height as f64 / 512.0 * view_height as f64) as i32
    };
    let step = TEXTURE_HEIGHT as f64 / line_height as f64;
   
    let y = view_height/2 - line_height/2;

    let mut src = post_src as f64;
    for y_draw in y..(y+line_height) {
        let pixel = if let Some(tex) = texture {
            tex.bytes[src as usize]
        } else {
            0x50
        };
        // TODO replace this with a faster? buffered draw
        rdr.plot(x, y_draw as usize, pixel);
        src += step;
    }
}

fn calc_height(height_numerator: i32, x_intercept: i32, y_intercept: i32, view_x: i32, view_y: i32, view_cos: i32, view_sin: i32) -> i32 {
    let mut z = fixed_mul(new_fixed_i32(x_intercept-view_x), new_fixed_i32(view_cos)).to_i32() - fixed_mul(new_fixed_i32(y_intercept - view_y), new_fixed_i32(view_sin)).to_i32();
    if z < MIN_DIST {
         z = MIN_DIST;
    }
    (height_numerator << 8) / z
}

// Clears the screen and already draws the bottom and ceiling
fn clear_screen(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	let ceil_color = VGA_CEILING[state.episode*10+state.map_on];

	let half = prj.view_height/2;
	rdr.bar(0, 0, prj.view_width, half, ceil_color); 
	rdr.bar(0, half, prj.view_width, half, 0x19);
}

fn draw_play_screen(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	rdr.fade_out();

	let offset_prev = rdr.buffer_offset();
	for i in 0..3 {
		rdr.set_buffer_offset(SCREENLOC[i]); 
		draw_play_border(rdr, prj);
		rdr.pic(0, 200-STATUS_LINES, GraphicNum::STATUSBARPIC);
	}
	rdr.set_buffer_offset(offset_prev);

	draw_face(state, rdr);
	draw_health(state, rdr);
	draw_lives(state, rdr);
	draw_level(state, rdr);
	draw_ammo(state, rdr);
	draw_keys(state, rdr);
	draw_weapon(state, rdr);
	draw_score(state, rdr);
}

fn draw_play_border(rdr: &dyn Renderer, prj: &ProjectionConfig) {
	//clear the background:
	rdr.bar(0, 0, 320, 200-STATUS_LINES, 127);

	let xl = 160-prj.view_width/2;
	let yl = (200-STATUS_LINES-prj.view_height)/2;

	//view area
	rdr.bar(xl, yl, prj.view_width, prj.view_height, 127);

	//border around the view area
	hlin(rdr, xl-1, xl+prj.view_width, yl-1, 0);
	hlin(rdr, xl-1, xl+prj.view_width, yl+prj.view_height, 125);
	vlin(rdr, yl-1, yl+prj.view_height, xl-1, 0);
	vlin(rdr, yl-1, yl+prj.view_height, xl+prj.view_width, 125);

	rdr.plot(xl-1, yl+prj.view_height, 124);
}

fn hlin(rdr: &dyn Renderer, x: usize, z: usize, y: usize, c: u8) {
	rdr.hlin(x, y, (z-x)+1, c)
}

fn vlin(rdr: &dyn Renderer, y: usize, z: usize, x: usize, c: u8) {
	rdr.vlin(x, y, (z-y)+1, c)
}

fn draw_face(state: &GameState, rdr: &dyn Renderer) {
	if state.health > 0 {
		status_draw_pic(rdr, 17, 4, face_pic(3*((100-state.health)/16)+state.face_frame));
	} else {
		// TODO draw mutant face if last attack was needleobj
		status_draw_pic(rdr, 17, 4, GraphicNum::FACE8APIC)
	}
}

fn draw_health(state: &GameState, rdr: &dyn Renderer) {
	latch_number(rdr, 21, 16, 3, state.health);
}

fn draw_lives(state: &GameState, rdr: &dyn Renderer) {
	latch_number(rdr, 14, 16, 1, state.lives);
}

fn draw_level(state: &GameState, rdr: &dyn Renderer) {
	latch_number(rdr, 2, 16, 2, state.map_on+1);
}

fn draw_ammo(state: &GameState, rdr: &dyn Renderer) {
	latch_number(rdr, 27, 16, 2, state.ammo);
}

fn draw_keys(state: &GameState, rdr: &dyn Renderer) {
	if state.keys & 1 != 0 {
		status_draw_pic(rdr, 30, 4, GraphicNum::GOLDKEYPIC);
	} else {
		status_draw_pic(rdr, 30, 4, GraphicNum::NOKEYPIC)
	}

	if state.keys & 2 != 0 {
		status_draw_pic(rdr, 30, 20, GraphicNum::SILVERKEYPIC);
	} else {
		status_draw_pic(rdr, 30, 20, GraphicNum::NOKEYPIC);
	}
}

fn draw_weapon(state: &GameState, rdr: &dyn Renderer) {
	status_draw_pic(rdr, 32, 8, weapon_pic(state.weapon))
}

fn draw_score(state: &GameState, rdr: &dyn Renderer) {
	latch_number(rdr, 6, 16, 6, state.score);
}

// x in bytes
fn status_draw_pic(rdr: &dyn Renderer, x: usize, y: usize, pic: GraphicNum) {
    let offset_prev = rdr.buffer_offset();
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        let y_status = (200-STATUS_LINES) + y;
        rdr.pic(x*8, y_status, pic);  
    } 
    rdr.set_buffer_offset(offset_prev);
}

fn latch_number(rdr: &dyn Renderer, x_start: usize, y: usize, width: usize, num: usize) {
	let str = num.to_string();
	let mut w_cnt = width;
	let mut x = x_start;
	while str.len() < w_cnt {
		status_draw_pic(rdr, x, y, GraphicNum::NBLANKPIC);
		x += 1;
		w_cnt -= 1;
	}

	let mut c = if str.len() <= w_cnt {0} else {str.len()-w_cnt};
	let mut chars = str.chars();
	while c<str.len() {
		let ch = chars.next().unwrap();
		status_draw_pic(rdr, x, y, num_pic(ch.to_digit(10).unwrap() as usize));
		x += 1;
		c += 1;
	}
}

// reads input delta since last tic and manipulates the player state
fn poll_controls(ticker: &time::Ticker, input: &input::Input) -> Control {

    let tics = ticker.calc_tics() as i32;

    let mut control = Control{x:0, y:0};

    poll_keyboard_move(&mut control, input, tics);
    //TODO Mouse Move
    //TODO Joystick Move?

    //bound movement to a maximum
    let max = 100*tics;
    let min = -max;

    if control.x > max {
        control.x = max;
    } else if control.x < min {
        control.x = min;
    }

    if control.y > max {
        control.y = max;
    } else if control.y < min {
        control.y  = min;
    }
    if control.x != 0 || control.y != 0 {
        println!("control={:?}", control);
    }
    control
}

fn poll_keyboard_move(control: &mut Control, input: &input::Input, tics: i32) {
    //TODO impl button mapping, uses hardcoded buttons as for now
    let move_factor = if input.key_pressed(NumCode::RShift) {
        RUN_MOVE * tics
    } else {
        BASE_MOVE * tics
    };

    if input.key_pressed(NumCode::UpArrow) {
        println!("tics={}", tics);
        control.y -= move_factor;
    }
    if input.key_pressed(NumCode::DownArrow) {
        control.y += move_factor;
    }
    if input.key_pressed(NumCode::LeftArrow) {
        control.x -= move_factor;
    }
    if input.key_pressed(NumCode::RightArrow) {
        control.x += move_factor;
    }
}


