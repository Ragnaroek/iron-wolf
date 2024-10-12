use core::ascii;
use std::ascii::Char;

use vga::VGA;

use crate::{assets::GraphicNum, def::{GameState, WindowState, STATUS_LINES}, input::Input, menu::{clear_ms_screen, draw_stripes}, vga_render::VGARenderer, vh::vw_fade_in};

static ALPHA : [GraphicNum; 43] = [
    GraphicNum::NUM0PIC, GraphicNum::NUM1PIC, GraphicNum::NUM2PIC, GraphicNum::NUM3PIC, GraphicNum::NUM4PIC, GraphicNum::NUM5PIC,
    GraphicNum::NUM6PIC, GraphicNum::NUM7PIC, GraphicNum::NUM8PIC, GraphicNum::NUM9PIC, GraphicNum::COLONPIC, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::APIC, GraphicNum::BPIC, 
    GraphicNum::CPIC, GraphicNum::DPIC, GraphicNum::EPIC, GraphicNum::FPIC, GraphicNum::GPIC, GraphicNum::HPIC, GraphicNum::IPIC, GraphicNum::JPIC, GraphicNum::KPIC, 
    GraphicNum::LPIC, GraphicNum::MPIC, GraphicNum::NPIC, GraphicNum::OPIC, GraphicNum::PPIC, GraphicNum::QPIC, GraphicNum::RPIC, GraphicNum::SPIC, GraphicNum::TPIC, 
    GraphicNum::UPIC, GraphicNum::VPIC, GraphicNum::WPIC, GraphicNum::XPIC, GraphicNum::YPIC, GraphicNum::ZPIC];

const ASCII_ALPHA_RANGE : u8 = Char::SmallA as u8 - Char::CapitalA as u8; // 'a' - 'A'

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
pub async fn level_completed(vga: &VGA, rdr: &VGARenderer, input: &Input, game_state: &GameState, win_state: &mut WindowState) {
    rdr.set_buffer_offset(rdr.active_buffer());

    clear_split_vwb(win_state);
    rdr.bar(0, 0, 320, 200-STATUS_LINES, 127);
    // TODO StartCPMusic(ENDLEVEL_MUS)

    // do the intermission
    rdr.set_buffer_offset(rdr.active_buffer());
    rdr.pic(0, 16, GraphicNum::LGUYPIC);

    if game_state.map_on < 8 {
        // CURR: Imple write function and write "floor\ncompleted"!!!
        write(rdr, 14, 2, "floor\ncompleted");
        write(rdr, 14, 7, "bonus     0");
        write(rdr, 16, 10, "time");
        write(rdr, 16, 12, " par");
        write(rdr, 9, 14, "kill ratio    %");
        write(rdr, 5, 16, "secret ratio    %");
        write(rdr, 1, 18, "treasure ratio    %");

        write(rdr, 26, 2, (game_state.map_on+1).to_string().as_str())
    } else {
        // TODO secret floot completed
    }

    // TODO write level complete data into screen

    vw_fade_in(vga).await;
    
    input.ack().await;
}

fn write(rdr: &VGARenderer, x: usize, y: usize, str: &str) {
    let mut nx = x * 8;
    let ox = nx;
    let mut ny = y * 8;
    for c in str.chars() {
        if let Some(ascii_char) = c.as_ascii() {
            if ascii_char == Char::LineFeed {
                nx = ox;
                ny += 16;
            } else {
                match ascii_char {
                    Char::ExclamationMark => {
                        rdr.pic(nx, ny, GraphicNum::EXPOINTPIC);
                        nx += 8;
                    },
                    Char::Apostrophe => {
                        rdr.pic(nx, ny, GraphicNum::APOSTROPHEPIC);
                        nx += 8;
                    },
                    Char::Space => {
                        nx += 16;
                    },
                    Char::PercentSign => {
                        rdr.pic(nx, ny, GraphicNum::PERCENTPIC);
                        nx += 16;
                    },
                    _ => {
                        let mut ch = ascii_char;
                        if ch >= Char::SmallA {
                            ch = Char::from_u8(ch as u8 - ASCII_ALPHA_RANGE).expect("valid ascii")
                        }
                        ch = Char::from_u8(ch as u8 - Char::Digit0 as u8).expect("valid ascii");
                        
                        rdr.pic(nx, ny, ALPHA[ch as usize]);
                        nx += 16;
                    }
                }
            }
        }
    }
}