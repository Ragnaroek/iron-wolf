
use super::vga_render::Renderer;
use super::assets::{GraphicNum};
use super::input;
use super::time;
use super::config;

const STATUS_LINES : usize = 40;
const HEIGHT_RATIO : f32 = 0.5;

pub struct ProjectionConfig {
	view_size: usize,
	view_width: usize,
	view_height: usize,
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

pub fn game_loop(rdr: &dyn Renderer, input: &input::Input, prj: &ProjectionConfig) {
	draw_play_screen(rdr, prj);

	rdr.fade_in();
	input.user_input(time::TICK_BASE*1000);
}

fn draw_play_screen(rdr: &dyn Renderer, prj: &ProjectionConfig) {
	rdr.fade_out();

	for i in 0..3 { 
		draw_play_border(rdr, prj);
		//TODO draw border to all three buffers
		rdr.pic(0, 200-STATUS_LINES, GraphicNum::STATUSBARPIC);
	}

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