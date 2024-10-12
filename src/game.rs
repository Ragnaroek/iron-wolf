
use super::vga_render::Renderer;
use super::assets::{GraphicNum};
use super::input;
use super::time;

const STATUS_LINES : usize = 40;

pub fn game_loop(rdr: &dyn Renderer, input: &input::Input) {
	draw_play_screen(rdr);

	rdr.fade_in();
	input.user_input(time::TICK_BASE*1000);
}

fn draw_play_screen(rdr: &dyn Renderer) {
	rdr.fade_out();

	for i in 0..3 { //TODO Why the hell is this done three times??
		draw_play_border(rdr);
		rdr.pic(0, 200-STATUS_LINES, GraphicNum::STATUSBARPIC);
	}

	
}

fn draw_play_border(rdr: &dyn Renderer) {
	//clear the background:
	rdr.bar(0, 0, 320, 200-STATUS_LINES, 127);

	//TODO draw remaining stuff
}