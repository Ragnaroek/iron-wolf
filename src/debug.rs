use vga::input::NumCode;

use crate::def::{At, GameState, LevelState, ObjKey, ObjType, WindowState};
use crate::input;
use crate::play::center_window;
use crate::us1::{print, print_centered};
use crate::vga_render::VGARenderer;

pub async fn debug_keys(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    game_state: &mut GameState,
    player: &ObjType,
    input: &input::Input,
) {
    win_state.font_color = 0;
    win_state.font_number = 0;

    if input.key_pressed(NumCode::F) {
        center_window(rdr, win_state, 14, 4);
        print(
            rdr,
            win_state,
            &format!("X:{}\nY:{}\nA:{}", player.x, player.y, player.angle),
        );
        input.ack().await;
        return;
    }
    if input.key_pressed(NumCode::G) {
        center_window(rdr, win_state, 12, 2);
        if game_state.god_mode {
            print_centered(rdr, win_state, "God mode OFF");
        } else {
            print_centered(rdr, win_state, "God mode ON");
        }
        input.ack().await;
        game_state.god_mode = !game_state.god_mode;
        return;
    }
}

pub fn debug_actor_at(level_state: &LevelState, x: usize, y: usize, width: usize, height: usize) {
    print!("   |");
    for w in 0..width {
        print!("{:>3}|", x + w);
    }
    println!();

    for h in 0..height {
        print!("{:>3}|", y + h);
        for w in 0..width {
            let at = level_state.actor_at[x + w][y + h];
            match at {
                At::Wall(_) => print!("###|"),
                At::Nothing => print!("   |"),
                At::Obj(ObjKey(k)) => print!("{:>3}|", k),
            }
        }
        println!();
    }
}
