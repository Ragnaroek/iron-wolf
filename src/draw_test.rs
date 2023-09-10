use crate::def::{ObjType, LevelState, Level, MAP_SIZE, DoorType, DoorAction, FL_NEVERMARK, DirType, ClassType};
use crate::draw::{Op, Hit, init_ray_cast, init_ray_cast_consts, calc_height};
use crate::fixed::new_fixed_i32;
use crate::play;
use crate::agent::S_PLAYER;

use super::RayCast;

#[test]
fn test_cast_angle_63() -> Result<(), String>{
    let mut prj = play::calc_projection(19);
    // fix rounding errors fine_tangents to make the
    // test results fully compatible with the original
    prj.fine_tangents[898] = prj.fine_tangents[898]+2; 
    let mut level_state = mock_level_state(); 
    level_state.mut_player().angle = 63; // an interesting angle in the start level
    let consts = init_ray_cast_consts(&prj, level_state.player());
    let mut rc = init_ray_cast(prj.view_width);
    
    assert_eq!(consts.x_partialup, 42879);
    assert_eq!(consts.y_partialup, 12924);
    assert_eq!(consts.x_partialdown, 22657);
    assert_eq!(consts.y_partialdown, 52612);

    for pixx in 0..prj.view_width {
        rc.init_cast(&prj, pixx, &consts);
        match pixx {
            0 => check_init_pixx_0(&rc),
            1 => check_init_pixx_1(&rc),
            42 => check_init_pixx_42(&rc),
            46 => check_init_pixx_46(&rc),
            _ => () /* no check */,
        }
        
        rc.cast(&mut level_state);
        match pixx {
            0 => check_cast_pixx_0(&rc),
            1 => check_cast_pixx_1(&rc),
            42 => check_cast_pixx_42(&rc),
            46 => check_cast_pixx_46(&rc),
            _ => () /* no check */,
        }
    }
    Ok(())
}

fn check_init_pixx_0(rc : &RayCast) {
    assert_eq!(rc.si, 0x0737);
    assert_eq!(rc.di, 0x0778, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, 0x0037, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001C);
    assert_eq!(rc.bp, 0x0038);

    assert_eq!(rc.x_tilestep, -1);
    assert_eq!(rc.y_tilestep, -1);
    assert_eq!(rc.horizop, Op::JLE);
    assert_eq!(rc.vertop, Op::JLE);
    assert_eq!(rc.x_step, -10438);
    assert_eq!(rc.y_step, -411453);
    assert_eq!(rc.y_intercept, 3645918);
    assert_eq!(rc.x_intercept&0xFFFF, 14278);
    assert_eq!(rc.x_tile, 28);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_pixx_0(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x09);
    assert_eq!(rc.x_intercept, 0x1D0F00, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x370000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 0x1D);
    assert_eq!(rc.y_tile, 0x37);
}

fn check_init_pixx_1(rc : &RayCast) {
    assert_eq!(rc.si, 0x0737);
    assert_eq!(rc.di, 0x0778, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, 0x0037, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001C);
    assert_eq!(rc.bp, 0x0038);

    assert_eq!(rc.x_tilestep, -1);
    assert_eq!(rc.y_tilestep, -1);
    assert_eq!(rc.horizop, Op::JLE);
    assert_eq!(rc.vertop, Op::JLE);
    assert_eq!(rc.x_step, -10204);
    assert_eq!(rc.y_step, -420906);
    assert_eq!(rc.y_intercept, 3642650);
    assert_eq!(rc.x_intercept&0xFFFF, 0x3882);
    assert_eq!(rc.x_tile, 28);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_pixx_1(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x09);
    assert_eq!(rc.x_intercept, 0x1D10A6, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x370000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 0x1D);
    assert_eq!(rc.y_tile, 0x37);
}

fn check_init_pixx_42(rc : &RayCast) {
    assert_eq!(rc.si, 0x071B);
    assert_eq!(rc.di, 0x0778, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, 0x001B, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001C);
    assert_eq!(rc.bp, 0x0038);

    assert_eq!(rc.x_tilestep, -1);
    assert_eq!(rc.y_tilestep, -1);
    assert_eq!(rc.horizop, Op::JLE);
    assert_eq!(rc.vertop, Op::JLE);
    assert_eq!(rc.x_step, -743);
    assert_eq!(rc.y_step, -5776577);
    assert_eq!(rc.y_intercept, 1791096);
    assert_eq!(rc.x_intercept&0xFFFF, 22061);
    assert_eq!(rc.x_tile, 28);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_pixx_42(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x09);
    assert_eq!(rc.x_intercept, 0x1D5346, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x370000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 0x1D);
    assert_eq!(rc.y_tile, 0x37);
}

fn check_init_pixx_46(rc : &RayCast) {
    assert_eq!(rc.si, 0x06BF, "si={:x}", rc.si);
    assert_eq!(rc.di, 0x0778, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, -193, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001E);
    assert_eq!(rc.bp, 0x0038);

    assert_eq!(rc.x_tilestep, 1);
    assert_eq!(rc.y_tilestep, -1);
    assert_eq!(rc.horizop, Op::JGE);
    assert_eq!(rc.vertop, Op::JLE);
    assert_eq!(rc.x_step, 0xAB);
    assert_eq!(rc.y_step, -25032852, "ystep={:x}", rc.y_step);
    assert_eq!(rc.y_intercept, -12590370);
    assert_eq!(rc.x_intercept&0xFFFF, 0x590A);
    assert_eq!(rc.x_tile, 30);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_pixx_46(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x09);
    assert_eq!(rc.x_intercept, 0x1D59B5, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x370000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 0x1D);
    assert_eq!(rc.y_tile, 0x37);
}

#[test]
fn test_cast_angle_353() -> Result<(), String>{
    let prj = play::calc_projection(19);
    let mut level_state = mock_level_state(); 
    level_state.mut_player().angle = 353;
    let consts = init_ray_cast_consts(&prj, level_state.player());
    let mut rc = init_ray_cast(prj.view_width);

    assert_eq!(level_state.player().x, 1933312);
    assert_eq!(level_state.player().y, 3768320);
    assert_eq!(consts.view_cos, new_fixed_i32(65047));
    assert_eq!(consts.view_sin, new_fixed_i32(-2147475662));
    assert_eq!(consts.view_x, 1911207);
    assert_eq!(consts.view_y, 3765607); 
    assert_eq!(consts.x_partialup, 54873);
    assert_eq!(consts.y_partialup, 35481);
    assert_eq!(consts.x_partialdown, 10663);
    assert_eq!(consts.y_partialdown, 30055);

    //Do one ray cast with the const vars
    for pixx in 0..prj.view_width {
        rc.init_cast(&prj, pixx, &consts);
        rc.cast(&mut level_state);
    }
    Ok(())
}

#[test]
fn test_cast_angle_26() -> Result<(), String>{
    let prj = play::calc_projection(19);
    let mut level_state = mock_level_state();
    level_state.mut_player().angle = 26; 
    let consts = init_ray_cast_consts(&prj, level_state.player());
    let mut rc = init_ray_cast(prj.view_width);
    
    assert_eq!(consts.x_partialup, 52785);
    assert_eq!(consts.y_partialup, 23005);
    assert_eq!(consts.x_partialdown, 12751);
    assert_eq!(consts.y_partialdown, 42531);

    for pixx in 0..prj.view_width {
        rc.init_cast(&prj, pixx, &consts);
        match pixx {
            0 => check_init_angle_26_pixx_0(&rc),
            _ => () /* no check */,
        }
        
        rc.cast(&mut level_state);
        match pixx {
            0 => check_cast_angle_26_pixx_0(&rc),
            _ => () /* no check */,
        }
    }
    Ok(())
}

fn check_init_angle_26_pixx_0(rc : &RayCast) {
    assert_eq!(rc.si, 0x07B8);
    assert_eq!(rc.di, 0x0778, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, 0x0038, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001E);
    assert_eq!(rc.bp, 0x0038);

    assert_eq!(rc.x_tilestep, 1);
    assert_eq!(rc.y_tilestep, -1);
    assert_eq!(rc.horizop, Op::JGE);
    assert_eq!(rc.vertop, Op::JLE);
    assert_eq!(rc.x_step, 34772);
    assert_eq!(rc.y_step, -123515);
    assert_eq!(rc.y_intercept, 3678600);
    assert_eq!(rc.x_intercept&0xFFFF, 35317);
    assert_eq!(rc.x_tile, 30);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_angle_26_pixx_0(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x08);
    assert_eq!(rc.x_intercept, 0x1E11C9, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x370000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 0x1E);
    assert_eq!(rc.y_tile, 0x37);
}

#[test]
fn test_cast_angle_288() -> Result<(), String>{
    let prj = play::calc_projection(19);
    let mut level_state = mock_level_state();
    level_state.mut_player().angle = 288; 
    let consts = init_ray_cast_consts(&prj, level_state.player());
    let mut rc = init_ray_cast(prj.view_width);
    
    assert_eq!(consts.x_partialup, 39650);
    assert_eq!(consts.y_partialup, 53949);
    assert_eq!(consts.x_partialdown, 25886);
    assert_eq!(consts.y_partialdown, 11587);

    for pixx in 0..prj.view_width {
        rc.init_cast(&prj, pixx, &consts);
        match pixx {
            274 => check_init_angle_288_pixx_274(&rc),
            _ => () /* no check */,
        }
        
        rc.cast(&mut level_state);
        match pixx {
            274 => check_cast_angle_288_pixx_274(&rc),
            _ => () /* no check */,
        }
    }
    Ok(())
}

fn check_init_angle_288_pixx_274(rc : &RayCast) {
    assert_eq!(rc.x_tilestep, -1);
    assert_eq!(rc.y_tilestep, 1);
    assert_eq!(rc.horizop, Op::JLE);
    assert_eq!(rc.vertop, Op::JGE);
    
    assert_eq!(rc.si, 0x073A, "si={:x}", rc.si);
    assert_eq!(rc.di, 0x077A, "di={:x}", rc.di);
    assert_eq!(rc.cx, 0x001D, "cx={:x}", rc.cx);
    assert_eq!(rc.dx, 0x003A, "dx={:x}", rc.dx);
    assert_eq!(rc.bx, 0x001C);
    assert_eq!(rc.bp, 0x003A);

    assert_eq!(rc.x_step, -14349);
    assert_eq!(rc.y_step, 299320);
    assert_eq!(rc.y_intercept, 3865367);
    assert_eq!(rc.x_intercept&0xFFFF, 14074);
    assert_eq!(rc.x_tile, 28);
    assert_eq!(rc.y_tile, 0);
}
fn check_cast_angle_288_pixx_274(rc : &RayCast) {
    assert_eq!(rc.hit, Hit::HorizontalWall);
    assert_eq!(rc.tile_hit, 0x09);
    assert_eq!(rc.x_intercept, 0x1CFEED, "x_intercept={:x}", rc.x_intercept);
    assert_eq!(rc.y_intercept, 0x3B0000, "y_intercept={:x}", rc.y_intercept);
    assert_eq!(rc.x_tile, 28);
    assert_eq!(rc.y_tile, 59);
}

#[test]
fn test_init_ray_cast_consts() {
    let prj = play::calc_projection(19);
    let mut player = test_player();
    player.angle = 63;
    let consts = init_ray_cast_consts(&prj, &player);
    assert_eq!(consts.view_x, 1923201);
    assert_eq!(consts.view_y, 3788164);
}

#[test]
fn test_calc_height() {
    let prj = play::calc_projection(19);
    let mut player = test_player();
    player.angle = 63;

    let consts = init_ray_cast_consts(&prj, &player);
    assert_eq!(
        calc_height(prj.height_numerator, 1904384, 3670016, &consts),
        562,
    )
}

// Helper

fn mock_level_state() -> LevelState {
    let mut tile_map = vec![vec![0; MAP_SIZE]; MAP_SIZE]; 
    tile_map[28][59] = 9;
    tile_map[29][55] = 9;
    tile_map[29][59] = 9;
    tile_map[30][55] = 8;
    tile_map[30][59] = 8;
    tile_map[31][55] = 8;
    tile_map[31][59] = 8;
    tile_map[32][56] = 72;
    tile_map[32][57] = 148;
    tile_map[32][58] = 72;

    let mut player = test_player();
    player.tilex = 29;
    player.tiley = 57;
    
    LevelState {
        level: Level {
            tile_map,
        },
        actors: vec![player],
        actor_at: Vec::with_capacity(0),
        doors: mock_doors(),
        statics: Vec::with_capacity(0),
        spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
        vislist: Vec::with_capacity(0),
    }
}

fn mock_doors() -> Vec<DoorType>{
    let mut doors = Vec::with_capacity(22);
    for i in 0..22 {
        doors.push(DoorType{
            num: i | 0x80,
            tile_x: 0,
            tile_y: 0,
            vertical: true,
            lock: 0,
            action: DoorAction::Closed,
            tic_count: 0,
            position: 0,
        });
    }
    return doors;
}

fn test_player() -> ObjType {
    ObjType{
        class: ClassType::Player,
        flags: FL_NEVERMARK,
        view_height: 0,
        view_x: 0,
        trans_x: new_fixed_i32(0),
        trans_y: new_fixed_i32(0),
        active: true,
        angle: 0,
        pitch: 0,
        x: 1933312,
        y: 3768320,
        tilex: 1904384,
        tiley: 1923201,
        dir: DirType::NoDir,
        speed: 0,
        state: &S_PLAYER,
    }
}