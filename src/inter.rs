use vga::VGA;

use crate::{menu::{draw_stripes, clear_ms_screen}, vga_render::VGARenderer, input::Input, assets::GraphicNum, def::{WindowState, STATUS_LINES}, vl::fade_in, vh::vw_fade_in};

pub fn clear_split_vwb(win_state: &mut WindowState) {
    // TODO clear 'update' global variable?
    win_state.window_x = 0;
    win_state.window_y = 0;
    win_state.window_w = 320;
    win_state.window_h = 160;
}

pub async fn check_highscore(rdr: &VGARenderer, input: &Input, score: i32, map: usize) {

    // TODO load high_score and check whether user achieved high_score

    draw_high_scores(rdr);
    rdr.activate_buffer(rdr.buffer_offset()).await;
    rdr.fade_in().await;

    input.clear_keys_down();
    input.wait_user_input(500).await;
}

pub fn draw_high_scores(rdr: &VGARenderer) {
    clear_ms_screen(rdr);
    draw_stripes(rdr, 10);

    rdr.pic(48, 0, GraphicNum::HIGHSCOREPIC);
    rdr.pic(4*8, 68, GraphicNum::CNAMEPIC);
    rdr.pic(20*8, 68, GraphicNum::CLEVELPIC);
    rdr.pic(28*8, 68, GraphicNum::CSCOREPIC);
}
/// LevelCompleted
///
/// Entered with the screen faded out
/// Still in split screen mode with the status bar
///
/// Exit with the screen faded out
pub async fn level_completed(vga: &VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    rdr.set_buffer_offset(rdr.active_buffer());

    clear_split_vwb(win_state);
    rdr.bar(0, 0, 320, 200-STATUS_LINES, 127);
    // TODO StartCPMusic(ENDLEVEL_MUS)

    // do the intermission
    rdr.set_buffer_offset(rdr.active_buffer());
    rdr.pic(0, 16, GraphicNum::LGUYPIC);

    // TODO write level complete data into screen

    vw_fade_in(vga).await;
    
    input.ack().await;
}