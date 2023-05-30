#[cfg(test)]
#[path = "./draw_test.rs"]
mod draw_test;

use crate::play::ProjectionConfig;
use crate::def::{GameState, Assets, Level, LevelState, ObjType, MIN_DIST, MAP_SIZE, TILEGLOBAL, TILESHIFT, ANGLES, FOCAL_LENGTH, FINE_ANGLES};
use crate::vga_render::Renderer;
use crate::vga_render;
use crate::fixed::{Fixed, fixed_mul, fixed_by_frac, new_fixed_i32};

const DEG90 : usize = 900;
const DEG180 : usize = 1800;
const DEG270 : usize = 2700;
const DEG360 : usize = 3600;

static MAP_MASKS_1 : [u8; 4*8] = [
    1 ,3 ,7 ,15,15,15,15,15,
    2 ,6 ,14,14,14,14,14,14,
    4 ,12,12,12,12,12,12,12,
    8 ,8 ,8 ,8 ,8 ,8 ,8 ,8
];

static VGA_CEILING : [u8; 60] = [	
	0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0x1d,0xbf,
	0x4e,0x4e,0x4e,0x1d,0x8d,0x4e,0x1d,0x2d,0x1d,0x8d,
	0x1d,0x1d,0x1d,0x1d,0x1d,0x2d,0xdd,0x1d,0x1d,0x98,
   
	0x1d,0x9d,0x2d,0xdd,0xdd,0x9d,0x2d,0x4d,0x1d,0xdd,
	0x7d,0x1d,0x2d,0x2d,0xdd,0xd7,0x1d,0x1d,0x1d,0x2d,
	0x1d,0x1d,0x1d,0x1d,0xdd,0xdd,0x7d,0xdd,0xdd,0xdd
];

pub struct ScalerState {
    last_side: bool,
    post_x: usize,
    post_width: usize,
    post_source: usize,
    texture_ix: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Hit {
    VerticalBorder,
    HorizontalBorder,
    VerticalWall,
    HorizontalWall,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    JLE,
    JGE,
}

// const throughout the core cast loop
pub struct RayCastConsts {
    pub view_angle : i32,
    pub mid_angle: i32,
    pub view_sin: Fixed,
    pub view_cos: Fixed,
    pub view_x: i32,
    pub view_y: i32,
    pub focal_tx: i32,
    pub focal_ty: i32,
    pub x_partialdown: i32,
    pub x_partialup: i32,
    pub y_partialdown: i32,
    pub y_partialup: i32,
}
pub struct RayCast {
    pub tile_hit: u16,
    pub hit: Hit, 

    pub x_tile: i32,
    pub y_tile: i32,
    pub x_tilestep: i32,
    pub y_tilestep: i32,
    pub x_step: i32,
    pub y_step: i32,
    pub y_intercept: i32,
    pub x_intercept: i32,
    pub x_partial: i32,
    pub y_partial: i32,
    
    pub wall_height: Vec<i32>,

    // register names from the assembler port (rename after port is complete)
    pub si: i32, // xspot
    pub di: i32, // yspot
    pub cx: i32, // high word of xintercept 
    pub dx: i32, // high word of yintercept
    pub bx: i32, // xtile
    pub bp: i32, // ytile
    pub horizop: Op,
    pub vertop: Op,
}

enum DirJmp {
    HorizEntry,
    VertEntry,
}

enum DirCheck {
    VertCheck,
    HorizCheck,
}

pub fn init_scaler_state() -> ScalerState {
    ScalerState { 
        last_side: false,
        post_x: 0,
        post_width: 1,
        post_source: 0,
        texture_ix: 0,
     }
}

fn hit(level: &Level, linear_offset: usize) -> (bool, u16) {
    let x = linear_offset/MAP_SIZE;
    let y = linear_offset - x*MAP_SIZE;
    let tile = level.tile_map[x][y];
    ((tile & 0xFF) != 0, tile)
}

fn init_ray_cast_consts(prj: &ProjectionConfig, player: &ObjType) -> RayCastConsts {
    let view_sin = prj.sin(player.angle as usize);
    let view_cos = prj.cos(player.angle as usize);
    let view_x = player.x - fixed_by_frac(new_fixed_i32(FOCAL_LENGTH), view_cos).to_i32();
    let view_y = player.y + fixed_by_frac(new_fixed_i32(FOCAL_LENGTH), view_sin).to_i32();

    let x_partialdown = view_x&(TILEGLOBAL-1);
    let x_partialup = TILEGLOBAL-x_partialdown;
    let y_partialdown = view_y&(TILEGLOBAL-1);
    let y_partialup = TILEGLOBAL-y_partialdown;

    RayCastConsts { 
        view_angle: player.angle,
        mid_angle: player.angle * (FINE_ANGLES as i32/ANGLES as i32),
        view_sin,
        view_cos, 
        view_x, 
        view_y, 
        focal_tx: view_x >> TILESHIFT,
        focal_ty: view_y >> TILESHIFT,
        x_partialdown,
        x_partialup,
        y_partialdown,
        y_partialup, 
    }
}

pub fn init_ray_cast(view_width: usize) -> RayCast {
    RayCast{
        tile_hit: 0, hit: Hit::VerticalBorder, 
        x_intercept:0, y_intercept:0, 
        x_tile: 0, y_tile:0,
        x_tilestep: 0, y_tilestep: 0,
        x_step: 0, y_step: 0, 
        cx: 0, dx: 0,
        si: 0, di: 0,
        bx: 0, bp: 0,
        horizop: Op::JLE, vertop: Op::JLE,
        x_partial: 0, y_partial: 0,
        wall_height: vec![0; view_width],
    } 
}

impl RayCast {
    fn init_cast(&mut self, prj: &ProjectionConfig, pixx: usize, consts: &RayCastConsts) {
        let mut angl=consts.mid_angle + prj.pixelangle[pixx];
        if angl<0 {
            angl+=FINE_ANGLES as i32;
        }
        if angl>=DEG360 as i32 {
            angl-=FINE_ANGLES as i32;
        }
        if angl<DEG90 as i32 {
            self.x_tilestep = 1;
            self.y_tilestep = -1;
            self.horizop = Op::JGE;
            self.vertop = Op::JLE;
            self.x_step = prj.fine_tangents[DEG90-1-angl as usize];
            self.y_step = -prj.fine_tangents[angl as usize];
            self.x_partial = consts.x_partialup;
            self.y_partial = consts.y_partialdown;
        } else if angl<DEG180 as i32 {
            self.x_tilestep = -1;
            self.y_tilestep = -1;
            self.horizop = Op::JLE;
            self.vertop = Op::JLE;
            self.x_step = -prj.fine_tangents[angl as usize -DEG90];
            self.y_step = -prj.fine_tangents[DEG180-1-angl as usize];
            self.x_partial=consts.x_partialdown;
            self.y_partial=consts.y_partialdown;
        } else if angl<DEG270 as i32 {
            self.x_tilestep = -1;
            self.y_tilestep = 1;
            self.horizop = Op::JLE;
            self.vertop = Op::JGE;
            self.x_step = -prj.fine_tangents[DEG270-1-angl as usize];
            self.y_step = prj.fine_tangents[angl as usize - DEG180 as usize];
            self.x_partial=consts.x_partialup;
            self.y_partial=consts.y_partialup; 
        } else if angl<DEG360 as i32 {
            self.x_tilestep = 1;
            self.y_tilestep = 1;
            self.horizop = Op::JGE;
            self.vertop = Op::JGE;
            self.x_step = prj.fine_tangents[angl as usize - DEG270];
            self.y_step = prj.fine_tangents[DEG360-1-angl as usize];
            self.x_partial=consts.x_partialup;
            self.y_partial=consts.y_partialup;
        }
        //initvars:
        self.y_intercept = fixed_by_frac(new_fixed_i32(self.y_step), new_fixed_i32(self.x_partial)).to_i32();
        self.y_intercept += consts.view_y;
        self.dx = self.y_intercept >> 16;

        self.si = consts.focal_tx+self.x_tilestep;
        self.x_tile = self.si;
        self.si <<= 6;
        self.si += self.dx;

        self.y_tile = 0;

        self.x_intercept = fixed_by_frac(new_fixed_i32(self.x_step), new_fixed_i32(self.y_partial)).to_i32();
        self.x_intercept += consts.view_x;
        self.dx = self.x_intercept >> 16;
        self.cx = self.dx;

        self.bx = consts.focal_ty;
        self.bx += self.y_tilestep;
        self.bp = self.bx;
        self.di = self.dx;
        self.di <<= 6;
        self.di += self.bx;

        self.bx = self.x_tile;
        self.dx = self.y_intercept >> 16;
    }

    fn cast(&mut self, level: &Level) {
        let mut check = DirCheck::VertCheck;
        let mut dir : DirJmp;
        
        'checkLoop:
        loop {
            match check {
                DirCheck::VertCheck => {
                    dir = self.vertcheck();
                },
                DirCheck::HorizCheck => {
                    dir = self.horizcheck();
                }
            }

            match dir {
                DirJmp::VertEntry => {
                    let (hit, tile) = hit(level, self.si.try_into().unwrap());
                    if hit {
                        self.tile_hit = tile;
                        // TODO check for door
                        self.x_intercept = self.bx << 16;
                        self.x_tile = self.bx;
                        self.y_intercept &= 0xFFFF;
                        self.y_intercept |= self.dx << 16;
                        self.y_tile = self.dx;
                        self.hit = Hit::VerticalWall;
                        break 'checkLoop;
                    }
                    //passvert:
                    //TODO Update spotvis
                    self.bx += self.x_tilestep;
                    let y_intercept_low = (self.y_intercept & 0xFFFF) as u16;
                    let (y_intercept_low, carry) = y_intercept_low.overflowing_add(self.y_step as u16);
                    self.y_intercept = (self.y_intercept & 0x7FFF0000) | y_intercept_low as i32;
                    self.dx += (self.y_step >> 16) + if carry {1} else {0};
                    self.si = self.bx;
                    self.si <<= 6;
                    self.si += self.dx;
                    check = DirCheck::VertCheck;
                },
                DirJmp::HorizEntry => {
                    let (hit, tile) = hit(level, self.di.try_into().unwrap());
                    if hit {
                        //hithoriz:
                        self.tile_hit = tile;
                        //TODO check for door
                        self.x_intercept &= 0xFFFF;
                        self.x_intercept |= self.cx << 16;
                        self.x_tile = self.cx;
                        self.y_intercept = self.bp << 16;
                        self.y_tile = self.bp;
                        self.hit = Hit::HorizontalWall;
                        break 'checkLoop;
                    }
                    //passhoriz:
                    // TODO update spotvis
                    self.bp += self.y_tilestep;
                    let x_intercept_low = (self.x_intercept & 0xFFFF) as u16;
                    let (x_intercept_low, carry) = x_intercept_low.overflowing_add(self.x_step as u16);
                    self.x_intercept = (self.x_intercept & 0x7FFF0000) | x_intercept_low as i32;
                    self.cx += (self.x_step >> 16) + if carry {1} else {0};
                    self.di = self.cx;
                    self.di <<= 6;
                    self.di += self.bp;                    
                    check = DirCheck::HorizCheck;
                },
            }
        }
    }

    fn vertcheck(&self) -> DirJmp {
        match self.vertop {
            Op::JLE => {
                return if self.dx <= self.bp {
                    DirJmp::HorizEntry
                } else {
                    DirJmp::VertEntry
                }
            },
            Op::JGE => {
                return if self.dx >= self.bp {
                    DirJmp::HorizEntry
                } else {
                    DirJmp::VertEntry
                }
            },
        }
    }
    
    fn horizcheck(&self) -> DirJmp {
        match self.horizop {
            Op::JLE => {
                return if self.cx <= self.bx {
                    DirJmp::VertEntry
                } else {
                    DirJmp::HorizEntry
                }
            },
            Op::JGE => {
                return if self.cx >= self.bx {
                    DirJmp::VertEntry
                } else {
                    DirJmp::HorizEntry
                }
            },
        }
    }
}

fn wall_refresh(level_state: &LevelState, rdr: &dyn Renderer, prj: &ProjectionConfig, assets: &Assets) {
    let player = level_state.player();
    // TODO allocate memory for RayCast + RayCastConsts only once, not on each wall_refresh (benchmark it)!
    let consts = init_ray_cast_consts(prj, player);
    let mut rc = init_ray_cast(prj.view_width);
    let mut scaler_state = init_scaler_state();

    //asm_refresh / ray casting core loop
    for pixx in 0..prj.view_width {
        rc.init_cast(prj, pixx, &consts);
        rc.cast(&level_state.level);

        match rc.hit {
            Hit::VerticalWall|Hit::VerticalBorder => hit_vert_wall(&mut scaler_state, &mut rc, &consts, pixx, prj, rdr, assets),
            Hit::HorizontalWall|Hit::HorizontalBorder => hit_horiz_wall(&mut scaler_state, &mut rc, &consts, pixx, prj, rdr, assets),
            // TODO hit other things (door, pwall)
        }
    }
}

pub fn three_d_refresh(game_state: &GameState, level_state: &LevelState, rdr: &dyn Renderer, prj: &ProjectionConfig, assets: &Assets) {
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

// Clears the screen and already draws the bottom and ceiling
fn clear_screen(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	let ceil_color = VGA_CEILING[state.episode*10+state.map_on];

	let half = prj.view_height/2;
	rdr.bar(0, 0, prj.view_width, half, ceil_color); 
	rdr.bar(0, half, prj.view_width, half, 0x19);
}

//Helper functions

pub fn calc_height(height_numerator: i32, x_intercept: i32, y_intercept: i32, consts: &RayCastConsts) -> i32 {
    let gx = new_fixed_i32(x_intercept - consts.view_x);
    let gxt = fixed_by_frac(gx, consts.view_cos);

    let gy = new_fixed_i32(y_intercept - consts.view_y);
    let gyt = fixed_by_frac(gy, consts.view_sin);

    let mut nx = gxt.to_i32() - gyt.to_i32();

    if nx < MIN_DIST {
         nx = MIN_DIST;
    }

    height_numerator/(nx >> 8)
}

pub fn scale_post(scaler_state: &ScalerState, height: i32, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let texture = &assets.textures[scaler_state.texture_ix];
    
    //shr additionally by 2, the original offset is a offset into a DWORD pointer array.
    //We have to correct here for that in jump table.
    let mut h = ((height & 0xFFF8)>>3) as usize;
    if h >= prj.scaler.scale_call.len() {
        h = prj.scaler.scale_call.len()-1;
    }


    let ix = prj.scaler.scale_call[h];
    let scaler = &prj.scaler.scalers[ix];
    let offset = (scaler_state.post_x >> 2) + rdr.buffer_offset();
    let mask = ((scaler_state.post_x & 3) << 3)+1;
    rdr.set_mask(MAP_MASKS_1[mask-1]);
    for pix_scaler in &scaler.pixel_scalers {
        let pix = texture.bytes[scaler_state.post_source + pix_scaler.texture_src];
        for mem_dest in &pix_scaler.mem_dests {
            rdr.write_mem(offset + *mem_dest as usize, pix);
        }
    }
}

pub fn hit_vert_wall(scaler_state : &mut ScalerState, rc : &mut RayCast, consts: &RayCastConsts, pixx: usize, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let mut post_source = 0xFC0 - ((rc.y_intercept>>4) & 0xFC0);
    if rc.x_tilestep == -1 {
        post_source = 0xFC0-post_source;
        rc.x_intercept += TILEGLOBAL;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        vert_wall(rc.tile_hit as usize & !0x40)
    } else {
        vert_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

pub fn hit_horiz_wall(scaler_state : &mut ScalerState, rc : &mut RayCast, consts: &RayCastConsts, pixx: usize, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let mut post_source = 0xFC0 - ((rc.x_intercept>>4) & 0xFC0);
    if rc.y_tilestep == -1 {
        rc.y_intercept += TILEGLOBAL;
    } else {
        post_source = 0xFC0-post_source;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        horiz_wall(rc.tile_hit as usize & !0x40)
    } else {
        horiz_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

fn horiz_wall(i: usize) -> usize {
    if i == 0 { 0 } else { (i-1)*2 }
}

fn vert_wall(i: usize) -> usize {
    if i == 0 { 0 } else { (i-1)*2+1 }
}

pub fn hit_horiz_door() {
}

pub fn hit_vert_door() {
}

pub fn hit_horiz_pwall() {
}

pub fn hit_vert_pwall() {
}