use vga::input::NumCode;

use crate::input;
use crate::play::center_window;
use crate::def::{WindowState, GameState, ObjType};
use crate::us1::{print_centered, print};
use crate::vga_render::VGARenderer;

pub async fn debug_keys(rdr: &VGARenderer, win_state: &mut WindowState, game_state: &mut GameState, player: &ObjType, input: &input::Input) {
    win_state.font_color = 0;
    win_state.font_number = 0;

    if input.key_pressed(NumCode::F) {
        center_window(rdr, win_state, 14, 4);
        print(rdr, win_state, &format!("X:{}\nY:{}\nA:{}", player.x, player.y, player.angle));
        input.check_ack().await;
        return;
    }
    if input.key_pressed(NumCode::G) {
        center_window(rdr, win_state, 12, 2);
        if game_state.god_mode {
            print_centered(rdr, win_state, "God mode OFF");
        } else {
            print_centered(rdr, win_state, "God mode ON");
        }
        input.check_ack().await;
        game_state.god_mode = !game_state.god_mode;
        return;
    }
}