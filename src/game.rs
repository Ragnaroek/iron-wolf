
use super::vga_render::Renderer;
use super::assets::{GraphicNum, face_pic, num_pic};
use super::input;
use super::time;
use super::config;
use super::vga_render;

const STATUS_LINES : usize = 40;
const HEIGHT_RATIO : f32 = 0.5;

static SCREENLOC : [usize; 3] = [vga_render::PAGE_1_START, vga_render::PAGE_2_START, vga_render::PAGE_3_START];

pub struct ProjectionConfig {
	view_size: usize,
	view_width: usize,
	view_height: usize,
}

pub struct GameState {
	health: usize,
	face_frame: usize,
}

pub fn new_game_state() -> GameState {
	GameState {
		health: 100,
		face_frame: 0,
	}
}

pub fn new_projection_config(config: &config::WolfConfig) -> ProjectionConfig {
	let view_width = ((config.viewsize * 16) & !15) as usize;
	let view_height = ((((config.viewsize * 16) as f32 * HEIGHT_RATIO) as u16) & !1) as usize;
	
	ProjectionConfig {
		view_width,
		view_height,
		view_size: config.viewsize as usize,
	}
}

pub fn game_loop(state: &GameState, rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig) {
	draw_play_screen(state, rdr, prj);

	rdr.fade_in();
	input.user_input(time::TICK_BASE*1000);
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

	//TODO draw face, health, lives,...
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

// x in bytes
fn status_draw_pic(rdr: &dyn Renderer, x: usize, y: usize, pic: GraphicNum) {
	let y_status = (200-STATUS_LINES) + y;
	rdr.pic(x*8, y_status, pic);
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
