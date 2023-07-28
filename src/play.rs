#[cfg(test)]
#[path = "./play_test.rs"]
mod play_test;

use vgaemu::input::NumCode;

use crate::act1::move_doors;
use crate::fixed::{Fixed, new_fixed, new_fixed_u32};
use crate::draw::three_d_refresh;
use crate::vga_render::Renderer;
use crate::def::{GameState, ControlState, WeaponType, Button, Assets,ObjKey, LevelState, Control, GLOBAL1, TILEGLOBAL, ANGLES, ANGLE_QUAD, FINE_ANGLES, FOCAL_LENGTH, NUM_BUTTONS};
use crate::assets::{GraphicNum, face_pic, num_pic, weapon_pic};
use crate::input;
use crate::time;
use crate::vga_render;
use crate::game::setup_game_level;
use crate::scale::{CompiledScaler, setup_scaling};

//TODO separate draw.c stuff from play.c stuff in here

const MIN_DIST : i32 = 0x5800;

const STATUS_LINES : usize = 40;
const HEIGHT_RATIO : f32 = 0.5;
const SCREEN_WIDTH : usize = 80;

const PI : f32 = 3.141592657;

const ANG90 : usize = FINE_ANGLES/4;
const ANG180 : usize = ANG90*2;

const NUM_FINE_TANGENTS : usize = FINE_ANGLES/2 + ANG180;
const VIEW_GLOBAL : usize = 0x10000;
const RAD_TO_INT : f64 = FINE_ANGLES as f64 / 2.0 / std::f64::consts::PI;

const RUN_MOVE : u64 = 70;
const BASE_MOVE: u64 = 35;

static SCREENLOC : [usize; 3] = [vga_render::PAGE_1_START, vga_render::PAGE_2_START, vga_render::PAGE_3_START];

static BUTTON_SCAN : [NumCode; NUM_BUTTONS] = [NumCode::Control, NumCode::Alt, NumCode::RShift, NumCode::Space, NumCode::Num1, NumCode::Num2, NumCode::Num3, NumCode::Num4];

pub struct ProjectionConfig {
	pub view_width: usize,
	pub view_height: usize,
	pub screenofs: usize,
    pub height_numerator: i32,
	pub pixelangle: Vec<i32>,
    pub sines: Vec<Fixed>,
    pub fine_tangents: [i32; NUM_FINE_TANGENTS],
    pub scaler: CompiledScaler,
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
        weapon_frame: 0,
		face_frame: 0,
		episode: 0
	}
}

pub fn new_control_state() -> ControlState {
    ControlState {
        control: Control { x: 0, y: 0 },
        angle_frac: 0,
        button_held: [false; NUM_BUTTONS],
        button_state: [false; NUM_BUTTONS]
    }
}

pub fn calc_projection(view_size: usize) -> ProjectionConfig {
	let view_width = (view_size * 16) & !15;
	let view_height = ((((view_size * 16) as f32 * HEIGHT_RATIO) as u16) & !1) as usize;
	let screenofs = (200-STATUS_LINES-view_height)/2*SCREEN_WIDTH+(320-view_width)/8;
    let half_view = view_width/2;

    let face_dist = FOCAL_LENGTH + MIN_DIST;

	let pixelangle = calc_pixelangle(view_width, face_dist as f64);
    let sines = calc_sines();
    let fine_tangents = calc_fine_tangents();

    let scale = half_view as i32 * face_dist/(VIEW_GLOBAL as i32/2);
    let height_numerator = (TILEGLOBAL*scale)>>6;

    let scaler = setup_scaling((view_width as f32 * 1.5) as usize, view_height);

	ProjectionConfig {
		view_width,
		view_height,
		screenofs,
        height_numerator,
		pixelangle,
        sines,
        fine_tangents,
        scaler,
	}
}

fn calc_fine_tangents() -> [i32; NUM_FINE_TANGENTS] {
    let mut tangents = [0; FINE_ANGLES];
    for i in 0..FINE_ANGLES/8 {
        let tang = ((i as f64 +0.5)/RAD_TO_INT).tan();
        tangents[i] = (tang * TILEGLOBAL as f64) as i32;
        tangents[FINE_ANGLES/4-1-i] = (1.0/tang*TILEGLOBAL as f64) as i32;
    }
    tangents
}

fn calc_pixelangle(view_width: usize, face_dist: f64) -> Vec<i32> {
	let half_view = view_width/2;

	let mut pixelangles = vec![0; view_width as usize]; 
	for i in 0..half_view {
		let tang = ((i * VIEW_GLOBAL) as f64 / view_width as f64) / face_dist;
		let angle = (tang.atan() * RAD_TO_INT) as i32;
        pixelangles[half_view-1-i] = angle;
		pixelangles[half_view+i] = -angle;
	}

	pixelangles
}

fn calc_sines() -> Vec<Fixed> {
    //TODO_VANILLA +1?? Bug in the original? does it write outside the array there in the original?
    let mut sines: Vec<Fixed> = vec![new_fixed(0, 0); ANGLES+ANGLE_QUAD+1]; 

    let mut angle : f32 = 0.0;
    let angle_step = PI/2.0/ANGLE_QUAD as f32;
    for i in 0..=ANGLE_QUAD {
        let value : u32 = (GLOBAL1 as f32 * angle.sin()) as u32;
        //TODO ugly fixes in here, make this exact to the old c-code
        let v_fixed = new_fixed_u32(value.min(65535));
        let mut value_neg = value | 0x80000000u32;
        if i == 90 {
            //otherwise a ??rounding error?? occurs and walking
            //backward does not work anymore (TODO Fix this proper,
            //latest in the generalisation)
            value_neg -= 1;
        }
        let v_fixed_neg = new_fixed_u32(value_neg);
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
    let mut control_state : ControlState = new_control_state();
    
    draw_play_screen(&game_state, rdr, prj);

	let mut level_state = setup_game_level(prj, game_state.map_on, assets).unwrap();

	//TODO StartMusic
	//TODO PreloadGraphics
    
	draw_level(&game_state, rdr);
    
    rdr.fade_in();

	play_loop(ticker, &mut level_state, &game_state, &mut control_state, rdr, input, prj, assets);

	//TODO Go to next level (gamestate.map_on+=1)

	input.wait_user_input(time::TICK_BASE*1000);
}

fn play_loop(ticker: &time::Ticker, level_state: &mut LevelState, game_state: &GameState, control_state: &mut ControlState, rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig, assets: &Assets) {
	//TODO A lot to do here (clear palette, poll controls, prepare world)
    loop {
        let tics = ticker.calc_tics();

        poll_controls(control_state, tics, input);

        if input.key_pressed(NumCode::P) {
            let player = level_state.player();
            println!("x={},y={},angle={}", player.x, player.y, player.angle); 
        }

        move_doors(level_state, tics);

        for i in 0..level_state.actors.len() {
            do_actor(ObjKey(i), level_state, control_state, prj);
        }
        
	    three_d_refresh(game_state, level_state, rdr, prj, assets);

        let offset_prev = rdr.buffer_offset();
        for i in 0..3 {
            rdr.set_buffer_offset(SCREENLOC[i]);
        } 
        rdr.set_buffer_offset(offset_prev);
    }
}

fn do_actor(k: ObjKey, level_state: &mut LevelState, control_state: &mut ControlState, prj: &ProjectionConfig) {
    //TODO do ob->ticcount part from DoActor here
    let may_think = level_state.obj(k).state.think;
    if let Some(think) = may_think {
        think(k, level_state, control_state, prj)
    }

    //TODO remove obj if state becomes None
    //TODO return if flag = FL_NEVERMARK (the player obj always has this flag)
    //TODO Impl think for player = T_Player function and supply the corret (mutable) args
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
fn poll_controls(state: &mut ControlState, tics: u64, input: &input::Input) {
    state.control.x = 0;
    state.control.y = 0;
    state.button_held.copy_from_slice(&state.button_state);

    poll_keyboard_buttons(state, input);

    poll_keyboard_move(state, input, tics);
    //TODO Mouse Move
    //TODO Joystick Move?

    //bound movement to a maximum
    let max = 100*tics as i32;
    let min = -max;

    if state.control.x > max {
        state.control.x = max;
    } else if state.control.x < min {
        state.control.x = min;
    }

    if state.control.y > max {
        state.control.y = max;
    } else if state.control.y < min {
        state.control.y = min;
    }
}

fn poll_keyboard_buttons(state: &mut ControlState, input: &input::Input) {
    for i in 0..NUM_BUTTONS {
        state.button_state[i] = input.key_pressed(BUTTON_SCAN[i])
    }
}

fn poll_keyboard_move(state: &mut ControlState, input: &input::Input, tics: u64) {
    //TODO impl button mapping, uses hardcoded buttons as for now
    let move_factor = if state.button_state[Button::Run as usize] {
        RUN_MOVE * tics
    } else {
        BASE_MOVE * tics
    } as i32;

    if input.key_pressed(NumCode::UpArrow) {
        state.control.y -= move_factor;
    }
    if input.key_pressed(NumCode::DownArrow) {
        state.control.y += move_factor;
    }
    if input.key_pressed(NumCode::LeftArrow) {
        state.control.x -= move_factor;
    }
    if input.key_pressed(NumCode::RightArrow) {
        state.control.x += move_factor;
    }
}


