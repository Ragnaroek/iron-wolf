#[cfg(test)]
#[path = "./draw_test.rs"]
mod draw_test;

#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::act2::S_DEATH_CAM;
use crate::agent::get_bonus;
use crate::def::{
    Assets, ClassType, DoorLock, DoorType, GameState, Level, LevelState, ObjType, Sprite,
    StaticType, VisObj, ANGLES, DIR_ANGLE, FINE_ANGLES, FL_BONUS, FL_VISABLE, FOCAL_LENGTH,
    MAP_SIZE, MIN_DIST, NUM_WEAPONS, TILEGLOBAL, TILESHIFT,
};
use crate::fixed::{fixed_by_frac, new_fixed_i32, Fixed};
use crate::play::ProjectionConfig;
use crate::scale::{scale_shape, simple_scale_shape, MAP_MASKS_1};
use crate::sd::Sound;
use crate::time::{self, Ticker};
use crate::vga_render::{self, VGARenderer};

const DEG90: usize = 900;
const DEG180: usize = 1800;
const DEG270: usize = 2700;
const DEG360: usize = 3600;

const ACTOR_SIZE: i32 = 0x4000;

static VGA_CEILING: [u8; 60] = [
    0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0xbf, 0x4e, 0x4e, 0x4e, 0x1d, 0x8d, 0x4e,
    0x1d, 0x2d, 0x1d, 0x8d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x2d, 0xdd, 0x1d, 0x1d, 0x98, 0x1d, 0x9d,
    0x2d, 0xdd, 0xdd, 0x9d, 0x2d, 0x4d, 0x1d, 0xdd, 0x7d, 0x1d, 0x2d, 0x2d, 0xdd, 0xd7, 0x1d, 0x1d,
    0x1d, 0x2d, 0x1d, 0x1d, 0x1d, 0x1d, 0xdd, 0xdd, 0x7d, 0xdd, 0xdd, 0xdd,
];

static WEAPON_SCALE: [Sprite; NUM_WEAPONS] = [
    Sprite::KnifeReady,
    Sprite::PistolReady,
    Sprite::MachinegunReady,
    Sprite::ChainReady,
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
    VerticalDoor,
    HorizontalDoor,
    VerticalPushWall,
    HorizontalPushWall,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    JLE,
    JGE,
}

// const throughout the core cast loop
// TODO merge with RayCast
pub struct RayCastConsts {
    pub view_angle: i32,
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
    pub push_wall_pos: i32,
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
    pub spotvis: Vec<Vec<bool>>,

    // register names from the assembler port (TODO rename after port is complete)
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

fn xy(linear_offset: usize) -> (usize, usize) {
    let x = linear_offset / MAP_SIZE;
    let y = linear_offset - x * MAP_SIZE;
    return (x, y);
}

fn hit(level: &Level, x: usize, y: usize) -> (bool, u16) {
    let tile = level.tile_map[x][y];
    ((tile & 0xFF) != 0, tile)
}

pub fn init_ray_cast_consts(
    prj: &ProjectionConfig,
    player: &ObjType,
    push_wall_pos: i32,
) -> RayCastConsts {
    let view_sin = prj.sin(player.angle as usize);
    let view_cos = prj.cos(player.angle as usize);
    let view_x = player.x - fixed_by_frac(new_fixed_i32(FOCAL_LENGTH), view_cos).to_i32();
    let view_y = player.y + fixed_by_frac(new_fixed_i32(FOCAL_LENGTH), view_sin).to_i32();

    let x_partialdown = view_x & (TILEGLOBAL - 1);
    let x_partialup = TILEGLOBAL - x_partialdown;
    let y_partialdown = view_y & (TILEGLOBAL - 1);
    let y_partialup = TILEGLOBAL - y_partialdown;

    RayCastConsts {
        view_angle: player.angle,
        mid_angle: player.angle * (FINE_ANGLES as i32 / ANGLES as i32),
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
        push_wall_pos,
    }
}

pub fn init_ray_cast(view_width: usize) -> RayCast {
    RayCast {
        tile_hit: 0,
        hit: Hit::VerticalBorder,
        x_intercept: 0,
        y_intercept: 0,
        x_tile: 0,
        y_tile: 0,
        x_tilestep: 0,
        y_tilestep: 0,
        x_step: 0,
        y_step: 0,
        cx: 0,
        dx: 0,
        si: 0,
        di: 0,
        bx: 0,
        bp: 0,
        horizop: Op::JLE,
        vertop: Op::JLE,
        x_partial: 0,
        y_partial: 0,
        wall_height: vec![0; view_width],
        spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
    }
}

impl RayCast {
    fn init_cast(&mut self, prj: &ProjectionConfig, pixx: usize, consts: &RayCastConsts) {
        let mut angl = consts.mid_angle + prj.pixelangle[pixx];
        if angl < 0 {
            angl += FINE_ANGLES as i32;
        }
        if angl >= DEG360 as i32 {
            angl -= FINE_ANGLES as i32;
        }
        if angl < DEG90 as i32 {
            self.x_tilestep = 1;
            self.y_tilestep = -1;
            self.horizop = Op::JGE;
            self.vertop = Op::JLE;
            self.x_step = prj.fine_tangents[DEG90 - 1 - angl as usize];
            self.y_step = -prj.fine_tangents[angl as usize];
            self.x_partial = consts.x_partialup;
            self.y_partial = consts.y_partialdown;
        } else if angl < DEG180 as i32 {
            self.x_tilestep = -1;
            self.y_tilestep = -1;
            self.horizop = Op::JLE;
            self.vertop = Op::JLE;
            self.x_step = -prj.fine_tangents[angl as usize - DEG90];
            self.y_step = -prj.fine_tangents[DEG180 - 1 - angl as usize];
            self.x_partial = consts.x_partialdown;
            self.y_partial = consts.y_partialdown;
        } else if angl < DEG270 as i32 {
            self.x_tilestep = -1;
            self.y_tilestep = 1;
            self.horizop = Op::JLE;
            self.vertop = Op::JGE;
            self.x_step = -prj.fine_tangents[DEG270 - 1 - angl as usize];
            self.y_step = prj.fine_tangents[angl as usize - DEG180 as usize];
            self.x_partial = consts.x_partialdown;
            self.y_partial = consts.y_partialup;
        } else if angl < DEG360 as i32 {
            self.x_tilestep = 1;
            self.y_tilestep = 1;
            self.horizop = Op::JGE;
            self.vertop = Op::JGE;
            self.x_step = prj.fine_tangents[angl as usize - DEG270];
            self.y_step = prj.fine_tangents[DEG360 - 1 - angl as usize];
            self.x_partial = consts.x_partialup;
            self.y_partial = consts.y_partialup;
        }
        //initvars:
        self.y_intercept =
            fixed_by_frac(new_fixed_i32(self.y_step), new_fixed_i32(self.x_partial)).to_i32();
        self.y_intercept += consts.view_y;
        self.dx = self.y_intercept >> 16;

        self.si = consts.focal_tx + self.x_tilestep;
        self.x_tile = self.si;
        self.si <<= 6;
        self.si += self.dx;

        self.y_tile = 0;

        self.x_intercept =
            fixed_by_frac(new_fixed_i32(self.x_step), new_fixed_i32(self.y_partial)).to_i32();
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

    fn cast(&mut self, consts: &RayCastConsts, level_state: &mut LevelState) {
        let mut check = DirCheck::VertCheck;
        let mut dir: DirJmp;

        'checkLoop: loop {
            match check {
                DirCheck::VertCheck => {
                    dir = self.vertcheck();
                }
                DirCheck::HorizCheck => {
                    dir = self.horizcheck();
                }
            }

            match dir {
                DirJmp::VertEntry => {
                    let (x, y) = xy(self.si.try_into().unwrap());
                    let (hit, tile) = hit(&level_state.level, x, y);
                    if hit {
                        self.tile_hit = tile;
                        let do_break;
                        if tile & 0x80 != 0 {
                            // vertdoor
                            self.x_tile = self.bx; // save off live register variables
                            self.y_intercept = self.y_intercept & 0xFFFF | self.dx << 16;

                            if tile & 0x40 != 0 {
                                let y_pos =
                                    ((self.y_step * consts.push_wall_pos) / 64) + self.y_intercept;
                                if (self.y_intercept >> 16) & 0xFFFF == y_pos >> 16 & 0xFFFF {
                                    // is it still in the same tile?
                                    self.y_intercept = y_pos;
                                    self.bx = self.x_tile;
                                    self.x_intercept = self.x_tile << 16;
                                    do_break = true;
                                    self.hit = Hit::VerticalPushWall;
                                } else {
                                    // no, it hit the side, continuevert
                                    self.bx = self.x_tile;
                                    self.dx = (self.y_intercept >> 16) & 0xFFFF;
                                    do_break = false
                                }
                            } else {
                                let door_num = (self.tile_hit & 0x7f) as usize;
                                let x_0 = self.y_step >> 1;
                                let x = x_0 + self.y_intercept;
                                let dx = (x >> 16) & 0xFFFF;
                                let ic = (self.y_intercept >> 16) & 0xFFFF;
                                if ic == dx {
                                    // is it still in the same tile?
                                    //hitvmid
                                    let door_pos = level_state.doors[door_num].position;
                                    let ax = (x & 0xFFFF) as u16;
                                    if ax < door_pos {
                                        self.bx = self.x_tile;
                                        self.dx = ic;
                                        do_break = false //continue with passvert
                                    } else {
                                        //draw the door
                                        self.y_intercept =
                                            (self.y_intercept & (0xFFFF << 16)) | ax as i32;
                                        self.x_intercept =
                                            (self.x_tile & 0xFFFF as i32) << 16 | 0x8000;
                                        do_break = true;
                                        self.hit = Hit::VerticalDoor
                                    }
                                } else {
                                    //else continue with tracing in passvert
                                    self.bx = self.x_tile;
                                    self.dx = ic;
                                    do_break = false
                                }
                            }
                        } else {
                            do_break = true;
                            self.hit = Hit::VerticalWall;
                            self.x_intercept = self.bx << 16;
                            self.x_tile = self.bx;
                            self.y_intercept &= 0xFFFF;
                            self.y_intercept |= self.dx << 16;
                            self.y_tile = self.dx;
                        }

                        if do_break {
                            break 'checkLoop;
                        }
                    }
                    //passvert:
                    level_state.spotvis[x][y] = true;
                    self.bx += self.x_tilestep;
                    let y_intercept_low = (self.y_intercept & 0xFFFF) as u16;
                    let (y_intercept_low, carry) =
                        y_intercept_low.overflowing_add(self.y_step as u16);
                    self.y_intercept = (self.y_intercept & 0x7FFF0000) | y_intercept_low as i32;
                    self.dx += (self.y_step >> 16) + if carry { 1 } else { 0 };
                    self.si = self.bx;
                    self.si <<= 6;
                    self.si += self.dx;
                    check = DirCheck::VertCheck;
                }
                DirJmp::HorizEntry => {
                    let (x, y) = xy(self.di.try_into().unwrap());
                    let (hit, tile) = hit(&level_state.level, x, y);
                    if hit {
                        //hithoriz:
                        self.tile_hit = tile;
                        let do_break;
                        if tile & 0x80 != 0 {
                            self.x_tile = self.bx; // save off live register variables
                            self.y_intercept = self.y_intercept & 0xFFFF | self.dx << 16;

                            if tile & 0x40 != 0 {
                                let x_pos =
                                    ((self.x_step * consts.push_wall_pos) / 64) + self.x_intercept;
                                if (self.x_intercept >> 16) & 0xFFFF == x_pos >> 16 & 0xFFFF {
                                    // is it still in the same tile?
                                    self.x_intercept = x_pos;
                                    self.y_intercept = self.bp << 16;
                                    do_break = true;
                                    self.hit = Hit::HorizontalPushWall;
                                } else {
                                    // no, it hit the side, continuevert
                                    self.bx = self.x_tile;
                                    self.dx = (self.y_intercept >> 16) & 0xFFFF;
                                    do_break = false
                                }
                            } else {
                                let door_num = (self.tile_hit & 0x7f) as usize;
                                let x_0 = self.x_step >> 1;
                                let x = x_0 + self.x_intercept;
                                self.dx = (x >> 16) & 0xFFFF;
                                let ic = (self.x_intercept >> 16) & 0xFFFF;
                                if ic == self.dx {
                                    // is it still in the same tile?
                                    //hithmid
                                    let door_pos = level_state.doors[door_num].position;
                                    let ax = (x & 0xFFFF) as u16;
                                    if ax < door_pos {
                                        self.bx = self.x_tile;
                                        self.dx = (self.y_intercept >> 16) & 0xFFFF;
                                        do_break = false //continue with passhoriz
                                    } else {
                                        //draw the door
                                        self.x_intercept = (self.cx << 16) | ax as i32;
                                        self.y_intercept = (self.bp & 0xFFFF as i32) << 16 | 0x8000;
                                        do_break = true;
                                        self.hit = Hit::HorizontalDoor
                                    }
                                } else {
                                    //else continue with tracing in passhoriz
                                    self.bx = self.x_tile;
                                    self.dx = (self.y_intercept >> 16) & 0xFFFF;
                                    do_break = false
                                }
                            }
                        } else {
                            do_break = true;
                            self.hit = Hit::HorizontalWall;
                            self.x_intercept &= 0xFFFF;
                            self.x_intercept |= self.cx << 16;
                            self.x_tile = self.cx;
                            self.y_intercept = self.bp << 16;
                            self.y_tile = self.bp;
                        }

                        if do_break {
                            break 'checkLoop;
                        }
                    }
                    //passhoriz:
                    level_state.spotvis[x][y] = true;
                    self.bp += self.y_tilestep;
                    let x_intercept_low = (self.x_intercept & 0xFFFF) as u16;
                    let (x_intercept_low, carry) =
                        x_intercept_low.overflowing_add(self.x_step as u16);
                    self.x_intercept = (self.x_intercept & 0x7FFF0000) | x_intercept_low as i32;
                    self.cx += (self.x_step >> 16) + if carry { 1 } else { 0 };
                    self.di = self.cx;
                    self.di <<= 6;
                    self.di += self.bp;
                    check = DirCheck::HorizCheck;
                }
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
            }
            Op::JGE => {
                return if self.dx >= self.bp {
                    DirJmp::HorizEntry
                } else {
                    DirJmp::VertEntry
                }
            }
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
            }
            Op::JGE => {
                return if self.cx >= self.bx {
                    DirJmp::VertEntry
                } else {
                    DirJmp::HorizEntry
                }
            }
        }
    }
}

#[cfg_attr(feature = "tracing", instrument(skip_all))]
pub fn wall_refresh(
    level_state: &mut LevelState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    // TODO Is there a faster way to do this?
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            level_state.spotvis[x][y] = false;
        }
    }
    let mut scaler_state = init_scaler_state();

    //asm_refresh / ray casting core loop
    for pixx in 0..prj.view_width {
        rc.init_cast(prj, pixx, consts);
        rc.cast(consts, level_state);

        match rc.hit {
            Hit::VerticalWall | Hit::VerticalBorder => hit_vert_wall(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state.level,
                assets,
            ),
            Hit::HorizontalWall | Hit::HorizontalBorder => hit_horiz_wall(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state.level,
                assets,
            ),
            Hit::VerticalDoor => hit_vert_door(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state,
                assets,
            ),
            Hit::HorizontalDoor => hit_horiz_door(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state,
                assets,
            ),
            Hit::VerticalPushWall => hit_vert_push_wall(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state,
                assets,
            ),
            Hit::HorizontalPushWall => hit_horiz_push_wall(
                &mut scaler_state,
                rc,
                &consts,
                pixx,
                prj,
                rdr,
                &level_state,
                assets,
            ),
        }
    }
}

#[cfg_attr(feature = "tracing", instrument(skip_all))]
pub async fn three_d_refresh(
    ticker: &time::Ticker,
    game_state: &mut GameState,
    level_state: &mut LevelState,
    rc: &mut RayCast,
    rdr: &VGARenderer,
    sound: &mut Sound,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    rdr.set_buffer_offset(rdr.buffer_offset() + prj.screenofs);

    let player = level_state.player();
    let consts = init_ray_cast_consts(prj, player, game_state.push_wall_pos);

    clear_screen(game_state, rdr, prj);
    wall_refresh(level_state, rc, &consts, rdr, prj, assets);

    draw_scaleds(
        level_state,
        game_state,
        &rc.wall_height,
        &consts,
        rdr,
        sound,
        prj,
        assets,
    );

    draw_player_weapon(ticker, level_state, game_state, rdr, prj, assets);

    if game_state.fizzle_in {
        rdr.fizzle_fade(
            ticker,
            rdr.buffer_offset(),
            rdr.active_buffer() + prj.screenofs,
            prj.view_width,
            prj.view_height,
            20,
            false,
        )
        .await;
        game_state.fizzle_in = false;
        ticker.clear_count(); // don't make a big tic count
    }

    rdr.set_buffer_offset(rdr.buffer_offset() - prj.screenofs);
    rdr.activate_buffer(rdr.buffer_offset()).await;

    //set offset to buffer for next frame
    let mut next_offset = rdr.buffer_offset() + vga_render::SCREEN_SIZE;
    if next_offset > vga_render::PAGE_3_START {
        next_offset = vga_render::PAGE_1_START;
    }
    rdr.set_buffer_offset(next_offset);
}

// Clears the screen and already draws the bottom and ceiling
#[cfg_attr(feature = "tracing", instrument(skip_all))]
fn clear_screen(state: &GameState, rdr: &VGARenderer, prj: &ProjectionConfig) {
    let ceil_color = VGA_CEILING[state.episode * 10 + state.map_on];

    let half = prj.view_height / 2;
    rdr.bar(0, 0, prj.view_width, half, ceil_color);
    rdr.bar(0, half, prj.view_width, half, 0x19);
}

//Helper functions

pub fn calc_height(
    height_numerator: i32,
    x_intercept: i32,
    y_intercept: i32,
    consts: &RayCastConsts,
) -> i32 {
    let gx = new_fixed_i32(x_intercept - consts.view_x);
    let gxt = fixed_by_frac(gx, consts.view_cos);

    let gy = new_fixed_i32(y_intercept - consts.view_y);
    let gyt = fixed_by_frac(gy, consts.view_sin);

    let mut nx = gxt.to_i32() - gyt.to_i32();

    if nx < MIN_DIST {
        nx = MIN_DIST;
    }

    height_numerator / (nx >> 8)
}

pub fn scale_post(
    scaler_state: &ScalerState,
    height: i32,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    assets: &Assets,
) {
    let texture = &assets.textures[scaler_state.texture_ix];

    //shr additionally by 2, the original offset is a offset into a DWORD pointer array.
    //We have to correct here for that in jump table.
    let mut h = ((height & 0xFFF8) >> 3) as usize;
    if h >= prj.scaler.scale_call.len() {
        h = prj.scaler.scale_call.len() - 1;
    }

    let ix = prj.scaler.scale_call[h];
    let scaler = &prj.scaler.scalers[ix];
    let offset = (scaler_state.post_x >> 2) + rdr.buffer_offset();
    let mask = ((scaler_state.post_x & 3) << 3) + 1;
    rdr.set_mask(MAP_MASKS_1[mask - 1]);
    for pix_scaler_opt in &scaler.pixel_scalers {
        if let Some(pix_scaler) = pix_scaler_opt {
            let pix = texture.bytes[scaler_state.post_source + pix_scaler.texture_src];
            for mem_dest in &pix_scaler.mem_dests {
                rdr.write_mem(offset + *mem_dest as usize, pix);
            }
        }
    }
}

pub fn hit_vert_wall(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level: &Level,
    assets: &Assets,
) {
    let mut post_source = (rc.y_intercept >> 4) & 0xFC0;
    if rc.x_tilestep == -1 {
        post_source = 0xFC0 - post_source;
        rc.x_intercept += TILEGLOBAL;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    //check for adjacent door
    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        rc.y_tile = rc.y_intercept >> TILESHIFT;
        if level.tile_map[(rc.x_tile - rc.x_tilestep) as usize][rc.y_tile as usize] & 0x80 != 0 {
            door_wall(assets) + 3
        } else {
            vert_wall(rc.tile_hit as usize & !0x40)
        }
    } else {
        vert_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

pub fn hit_horiz_wall(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level: &Level,
    assets: &Assets,
) {
    let mut post_source = (rc.x_intercept >> 4) & 0xFC0;
    if rc.y_tilestep == -1 {
        rc.y_intercept += TILEGLOBAL;
    } else {
        post_source = 0xFC0 - post_source;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    //check for adjacent door
    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        rc.x_tile = rc.x_intercept >> TILESHIFT;
        if level.tile_map[rc.x_tile as usize][(rc.y_tile - rc.y_tilestep) as usize] & 0x80 != 0 {
            door_wall(assets) + 2
        } else {
            horiz_wall(rc.tile_hit as usize & !0x40)
        }
    } else {
        horiz_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

pub fn hit_horiz_door(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level_state: &LevelState,
    assets: &Assets,
) {
    let doornum = rc.tile_hit & 0x7F;
    let door = &level_state.doors[doornum as usize];
    let post_source = ((rc.x_intercept - door.position as i32) >> 4) & 0xFC0;
    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;
    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = door_texture(door, assets);
}

pub fn hit_vert_door(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level_state: &LevelState,
    assets: &Assets,
) {
    let doornum = rc.tile_hit & 0x7F;
    let door = &level_state.doors[doornum as usize];
    let post_source = ((rc.y_intercept - door.position as i32) >> 4) & 0xFC0;
    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;
    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = door_texture(door, assets) + 1;
}

fn door_wall(assets: &Assets) -> usize {
    (assets.gamedata_headers.sprite_start - 8) as usize
}

fn door_texture(door: &DoorType, assets: &Assets) -> usize {
    let door_wall = door_wall(assets);
    match door.lock {
        DoorLock::Normal => door_wall,
        DoorLock::Lock1 | DoorLock::Lock2 | DoorLock::Lock3 | DoorLock::Lock4 => door_wall + 6,
        DoorLock::Elevator => door_wall + 4,
    }
}

pub fn hit_horiz_push_wall(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level_state: &LevelState,
    assets: &Assets,
) {
    let mut post_source = (rc.x_intercept >> 4) & 0xFC0;
    let offset = consts.push_wall_pos << 10;
    if rc.y_tilestep == -1 {
        rc.y_intercept += TILEGLOBAL - offset;
    } else {
        post_source = 0xFC0 - post_source;
        rc.y_intercept += offset;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = horiz_wall(rc.tile_hit as usize & 63);
    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

pub fn hit_vert_push_wall(
    scaler_state: &mut ScalerState,
    rc: &mut RayCast,
    consts: &RayCastConsts,
    pixx: usize,
    prj: &ProjectionConfig,
    rdr: &VGARenderer,
    level_state: &LevelState,
    assets: &Assets,
) {
    let mut post_source = (rc.y_intercept >> 4) & 0xFC0;
    let offset = consts.push_wall_pos << 10;
    if rc.x_tilestep == -1 {
        post_source = 0xFC0 - post_source;
        rc.x_intercept += TILEGLOBAL - offset;
    } else {
        rc.x_intercept += offset;
    }

    let height = calc_height(prj.height_numerator, rc.x_intercept, rc.y_intercept, consts);
    rc.wall_height[pixx] = height;

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = vert_wall(rc.tile_hit as usize & 63);
    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

fn horiz_wall(i: usize) -> usize {
    if i == 0 {
        0
    } else {
        (i - 1) * 2
    }
}

fn vert_wall(i: usize) -> usize {
    if i == 0 {
        0
    } else {
        (i - 1) * 2 + 1
    }
}

#[cfg_attr(feature = "tracing", instrument(skip_all))]
fn draw_player_weapon(
    ticker: &Ticker,
    level_state: &LevelState,
    game_state: &GameState,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    if game_state.victory_flag {
        let player = level_state.player();
        if player.state == Some(&S_DEATH_CAM) && (ticker.get_count() & 32) != 0 {
            let sprite = &assets.sprites[Sprite::DeathCam as usize];
            simple_scale_shape(rdr, prj, prj.view_width / 2, sprite, prj.view_height + 1);
        }
        return;
    }

    if let Some(weapon) = game_state.weapon {
        let shape_num = WEAPON_SCALE[weapon as usize] as usize + game_state.weapon_frame;
        let sprite = &assets.sprites[shape_num];
        simple_scale_shape(rdr, prj, prj.view_width / 2, sprite, prj.view_height + 1);
    }

    // TODO handle demorecord || demoplayback
}

#[cfg_attr(feature = "tracing", instrument(skip_all))]
fn draw_scaleds(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    wall_height: &Vec<i32>,
    consts: &RayCastConsts,
    rdr: &VGARenderer,
    sound: &mut Sound,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    let mut visptr = 0;
    // place static objects
    for stat in &mut level_state.statics {
        if stat.sprite == Sprite::None {
            continue; // object has been deleted
        }
        level_state.vislist[visptr].sprite = stat.sprite;

        if !level_state.spotvis[stat.tile_x][stat.tile_y] {
            continue; // not visible shape
        }

        let vis = &mut level_state.vislist[visptr];
        let can_grab = transform_tile(consts, prj, stat, vis);
        if can_grab && (stat.flags & FL_BONUS) != 0 {
            get_bonus(game_state, rdr, sound, assets, stat);
            continue;
        }

        if vis.view_height == 0 {
            continue;
        }

        if visptr < level_state.vislist.len() - 1 {
            visptr += 1;
        }
    }

    // place active objects (player + enemies)

    // just to have shorter names below
    let vis = &level_state.spotvis;
    let tile = &level_state.level.tile_map;
    let player_angle = level_state.player().angle;
    for obj in &mut level_state.actors {
        if obj.state.expect("state").sprite.is_none() {
            continue; // no shape
        }
        let visobj = &mut level_state.vislist[visptr];
        visobj.sprite = obj.state.expect("state").sprite.expect("sprite");

        if vis[obj.tilex][obj.tiley]
            || (vis[obj.tilex - 1][obj.tiley + 1] && tile[obj.tilex - 1][obj.tiley + 1] == 0)
            || (vis[obj.tilex][obj.tiley + 1] && tile[obj.tilex][obj.tiley + 1] == 0)
            || (vis[obj.tilex + 1][obj.tiley + 1] && tile[obj.tilex + 1][obj.tiley + 1] == 0)
            || (vis[obj.tilex - 1][obj.tiley] && tile[obj.tilex - 1][obj.tiley] == 0)
            || (vis[obj.tilex + 1][obj.tiley] && tile[obj.tilex + 1][obj.tiley] == 0)
            || (vis[obj.tilex - 1][obj.tiley - 1] && tile[obj.tilex - 1][obj.tiley - 1] == 0)
            || (vis[obj.tilex][obj.tiley - 1] && tile[obj.tilex][obj.tiley - 1] == 0)
            || (vis[obj.tilex + 1][obj.tiley - 1] && tile[obj.tilex + 1][obj.tiley - 1] == 0)
        {
            transform_actor(consts, prj, obj);
            if obj.view_height == 0 {
                continue; // too close or far away
            }
            visobj.view_x = obj.view_x;
            visobj.view_height = obj.view_height;

            if visobj.sprite == Sprite::None {
                let special_shape = Sprite::try_from(obj.temp1 as usize);
                if special_shape.is_ok() {
                    visobj.sprite = special_shape.expect("special shape");
                }
            }

            if obj.state.expect("state").rotate != 0 {
                let rotate = calc_rotate(prj, player_angle, obj);
                let sprite_base = obj
                    .state
                    .expect("state")
                    .sprite
                    .expect("sprite be present (checked above)");

                let maybe_sprite = Sprite::try_from(sprite_base as usize + rotate);
                if maybe_sprite.is_err() {
                    panic!(
                        "invalid sprite: {:?}, base = {:?}, base_num={} rotate = {}",
                        maybe_sprite.err(),
                        sprite_base,
                        sprite_base as usize,
                        rotate
                    )
                }
                visobj.sprite = maybe_sprite.expect("valid sprite");
            }

            if visptr < level_state.vislist.len() - 1 {
                visptr += 1;
            }
            obj.flags |= FL_VISABLE;
        } else {
            obj.flags &= !FL_VISABLE;
        }
    }

    // draw from back to front
    level_state.vislist[0..visptr].sort_by(|a, b| a.view_height.cmp(&b.view_height));
    for i in 0..visptr {
        let vis_obj = &level_state.vislist[i];
        let sprite_data = &assets.sprites[vis_obj.sprite as usize];
        scale_shape(
            rdr,
            wall_height,
            prj,
            vis_obj.view_x as usize,
            sprite_data,
            vis_obj.view_height as usize,
        );
    }
}

fn calc_rotate(prj: &ProjectionConfig, player_angle: i32, obj: &ObjType) -> usize {
    let view_angle = player_angle + (prj.center_x as i32 - obj.view_x) / 8;
    let mut angle = if obj.class == ClassType::Rocket || obj.class == ClassType::HRocket {
        (view_angle - 180) - obj.angle
    } else {
        (view_angle - 180) - DIR_ANGLE[obj.dir as usize] as i32
    };
    angle += ANGLES as i32 / 16;
    while angle >= ANGLES as i32 {
        angle -= ANGLES as i32;
    }
    while angle < 0 {
        angle += ANGLES as i32;
    }

    if obj.state.expect("state").rotate == 2 {
        return 4 * (angle as usize / (ANGLES / 2));
    }

    angle as usize / (ANGLES / 8)
}

fn transform_actor(consts: &RayCastConsts, prj: &ProjectionConfig, obj: &mut ObjType) {
    let gx = new_fixed_i32(obj.x - consts.view_x);
    let gy = new_fixed_i32(obj.y - consts.view_y);

    let gxt = fixed_by_frac(gx, consts.view_cos);
    let gyt = fixed_by_frac(gy, consts.view_sin);
    let nx = gxt.to_i32() - gyt.to_i32() - ACTOR_SIZE; // fudge the shape forward a bit, because
                                                       // the midpoint could put parts of the shape
                                                       // into an adjacent wall
    let gxt = fixed_by_frac(gx, consts.view_sin);
    let gyt = fixed_by_frac(gy, consts.view_cos);
    let ny = gyt.to_i32() + gxt.to_i32();

    // calculate perspective ratio

    obj.trans_x = new_fixed_i32(nx);
    obj.trans_y = new_fixed_i32(ny);

    if nx < MIN_DIST {
        // too close, don't overflow the divide
        obj.view_height = 0;
        return;
    }

    obj.view_x = prj.center_x as i32 + ny * prj.scale / nx;

    // calculate height (heightnumerator/(nx>>8))
    obj.view_height = prj.height_numerator / (nx >> 8);
}

fn transform_tile(
    consts: &RayCastConsts,
    prj: &ProjectionConfig,
    stat: &StaticType,
    visobj: &mut VisObj,
) -> bool {
    // translate point to view centered coordinates
    let gx = new_fixed_i32(((stat.tile_x as i32) << TILESHIFT) + 0x8000 - consts.view_x);
    let gy = new_fixed_i32(((stat.tile_y as i32) << TILESHIFT) + 0x8000 - consts.view_y);

    let gxt = fixed_by_frac(gx, consts.view_cos);
    let gyt = fixed_by_frac(gy, consts.view_sin);
    let nx = gxt.to_i32() - gyt.to_i32() - 0x2000; // 0x2000 is size of object

    let gxt = fixed_by_frac(gx, consts.view_sin);
    let gyt = fixed_by_frac(gy, consts.view_cos);
    let ny = gyt.to_i32() + gxt.to_i32();

    // calculate perspective ratio
    if nx < MIN_DIST {
        // too close, don't overflow the divide
        visobj.view_height = 0;
        return false;
    }
    visobj.view_x = prj.center_x as i32 + ny * prj.scale / nx;

    // calculate height (heightnumerator/(nx>>8))
    visobj.view_height = prj.height_numerator / (nx >> 8);

    // see if it should be grabbed
    if nx < TILEGLOBAL && ny > -TILEGLOBAL / 2 && ny < TILEGLOBAL / 2 {
        return true;
    }
    return false;
}
