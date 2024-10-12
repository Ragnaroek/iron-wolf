use super::vga_render::Renderer;
use super::def::{GameState, WeaponType, Assets, ObjType};
use super::assets::{GraphicNum, face_pic, num_pic, weapon_pic, load_map_from_assets};
use super::input;
use super::time;
use super::config;
use super::vga_render;

const STATUS_LINES : usize = 40;
const HEIGHT_RATIO : f32 = 0.5;
const SCREEN_WIDTH : usize = 80;
const MAX_VIEW_WIDTH : usize = 320;

const MAP_SIZE : usize = 64;

const AREATILE : u16 = 107;
const NUMAREAS : u16 = 37;

const NORTH : i32 = 0;
const EAST : i32 = 0;
const SOUTH : i32 = 0;
const WEST : i32 = 0;

const TILESHIFT : u32 = 16;
const TILEGLOBAL : u32 = 1<<16;

const FINE_ANGLES : i32 = 3600;
const ANGLES : i32 = 360;

const VIEW_GLOBAL : usize = 0x10000;
const FOCAL_LENGTH : usize = 0x5700;
const MIN_DIST : usize = 0x5800;

const RAD_TO_INT : f32 = FINE_ANGLES as f32 / 2.0 / std::f32::consts::PI;

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
	view_size: usize,
	view_width: usize,
	view_height: usize,
	screenofs: usize,
	pixelangle: [i32; MAX_VIEW_WIDTH],
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

pub fn new_projection_config(config: &config::WolfConfig) -> ProjectionConfig {
	let view_width = ((config.viewsize * 16) & !15) as usize;
	let view_height = ((((config.viewsize * 16) as f32 * HEIGHT_RATIO) as u16) & !1) as usize;
	let screenofs = (200-STATUS_LINES-view_height)/2*SCREEN_WIDTH+(320-view_width)/8;
	
	let pixelangle = calc_pixelangle(FOCAL_LENGTH, view_width);

	ProjectionConfig {
		view_width,
		view_height,
		view_size: config.viewsize as usize,
		screenofs,
		pixelangle,
	}
}

fn calc_pixelangle(focal: usize, view_width: usize) -> [i32; MAX_VIEW_WIDTH] {
	let half_view = view_width/2;
	let face_dist = (focal + MIN_DIST) as f32;

	let mut pixelangles = [0; MAX_VIEW_WIDTH];

	for i in 0..half_view {
		let tang = (i * VIEW_GLOBAL / view_width) as f32 / face_dist;
		let angle = (tang.atan() * RAD_TO_INT) as i32;
		pixelangles[half_view-1-i] = angle;
		pixelangles[half_view+i] = -angle;
	}

	pixelangles
}

struct Level {
	tile_map: [[u8;MAP_SIZE]; MAP_SIZE],
	actor_at: [[Option<ObjType>;MAP_SIZE]; MAP_SIZE],
	player: ObjType,
}

pub fn game_loop(state: &GameState, rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig, assets: &Assets) {
	draw_play_screen(state, rdr, prj);
	
	let level = setup_game_level(state, assets).unwrap();
	
	//TODO StartMusic
	//TODO PreloadGraphics

	draw_level(state, rdr);
	//TODO DrawLevel
	
	play_loop(state, rdr, prj);

	//TODO Go to next level (gamestate.map_on+=1)

	rdr.fade_in();
	input.user_input(time::TICK_BASE*1000);
}

fn play_loop(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	//TODO A lot to do here (clear palette, poll controls, prepare world)

	three_d_refresh(state, rdr, prj);
}

fn three_d_refresh(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	rdr.set_buffer_offset(rdr.buffer_offset() + prj.screenofs);

	clear_screen(state, rdr, prj);

	wall_refresh(state, prj);

	rdr.set_buffer_offset(rdr.buffer_offset() - prj.screenofs);

	// TODO: increment buffer offset to next page
}

fn wall_refresh(state: &GameState, prj: &ProjectionConfig) {
	
	//TODO take player angle from Level: level.player
	//let midangle = state.player.angle * (FINE_ANGLES/ANGLES);
	
	let midangle = 0;
	for pixx in 0..prj.view_width {
		let angle = midangle + prj.pixelangle[pixx];
		
		// compute xstep and ystep from angle!
		// depending on case 0-90, 90-180, 180-270, ... one of xstep or ystep is always 1!!

		// cast ray and see if it intersects with vertical or horizontal wall
	}
}

// Clears the screen and already draws the bottom and ceiling
fn clear_screen(state: &GameState, rdr: &dyn Renderer, prj: &ProjectionConfig) {
	let ceil_color = VGA_CEILING[state.episode*10+state.map_on];

	let half = prj.view_height/2;
	rdr.bar(0, 0, prj.view_width, half, ceil_color); 
	rdr.bar(0, half, prj.view_width, half, 0x19);	
}

fn setup_game_level(state: &GameState, assets: &Assets) -> Result<Level, String> {

	let map = &assets.map_headers[state.map_on];
	if map.width != MAP_SIZE as u16 || map.height != MAP_SIZE as u16 {
		panic!("Map not 64*64!");
	}

	let map_data = load_map_from_assets(assets, state.map_on)?;

	let mut tile_map = [[0;MAP_SIZE];MAP_SIZE];
	let actor_at = [[None;MAP_SIZE];MAP_SIZE];

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[0][map_ptr];
			map_ptr += 1;
			if tile < AREATILE {
				tile_map[x][y] = tile as u8;
			} else {
				tile_map[x][y] = 0;
			}
		}
	}

	//TODO init_actor_list?
	//TODO init_door_list?
	//TODO init_static_list?

	//TODO something with doors 90 to 101

	let player = scan_info_plane(&map_data);

	//TODO ambush markers

	Ok(Level {
		tile_map,
		actor_at,
		player,
	})
}

//Returns the player object
fn scan_info_plane(map_data: &libiw::map::MapData) -> ObjType {

	let mut player = None;

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[1][map_ptr];
			map_ptr += 1;
			match tile {
				19..=22 => player = Some(spawn_player(x, y, NORTH+(tile-19)as i32)),
				_ => {},
			}
		}
	}

	if player.is_none() {
		panic!("No player start position in map");
	}

	player.unwrap()
}

fn spawn_player(tilex: usize, tiley: usize, dir: i32) -> ObjType {
	ObjType{
		angle: (1-dir)*90,
		tilex,
		tiley,
		x: ((tilex as u32) << TILESHIFT) + TILEGLOBAL / 2,
		y: ((tiley as u32) << TILESHIFT) + TILEGLOBAL / 2,
	}
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
