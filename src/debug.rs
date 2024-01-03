use vga::input::NumCode;

use crate::input;
use crate::play::center_window;
use crate::def::{UserState, GameState};
use crate::us1::print_centered;
use crate::vga_render::VGARenderer;

pub async fn debug_keys(rdr: &VGARenderer, user_state: &mut UserState, game_state: &mut GameState, input: &input::Input) {
    user_state.font_color = 0;
    user_state.font_number = 0;
    
    if input.key_pressed(NumCode::G) {
        center_window(rdr, user_state, 12, 2);
        if game_state.god_mode {
            print_centered(rdr, user_state, "God mode OFF");
        } else {
            print_centered(rdr, user_state, "God mode ON");
        }
        input.check_ack().await;
        game_state.god_mode = !game_state.god_mode;
        return;
    }
}