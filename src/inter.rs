use crate::{menu::{draw_stripes, clear_ms_screen}, vga_render::VGARenderer, input::Input, assets::GraphicNum};



pub async fn check_highscore(rdr: &VGARenderer, input: &Input, score: i32, map: usize) {

    // TODO load high_score and check whether user achieved high_score

    draw_high_scores(rdr);
    rdr.activate_buffer(rdr.buffer_offset()).await;
    rdr.fade_in().await;

    input.clear_keys_down();
    input.wait_user_input(500).await;
}

fn draw_high_scores(rdr: &VGARenderer) {
    clear_ms_screen(rdr);
    draw_stripes(rdr, 10);

    rdr.pic(48, 0, GraphicNum::HIGHSCOREPIC);
    rdr.pic(4*8, 68, GraphicNum::CNAMEPIC);
    rdr.pic(20*8, 68, GraphicNum::CLEVELPIC);
    rdr.pic(28*8, 68, GraphicNum::CSCOREPIC);

}
