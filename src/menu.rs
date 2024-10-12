
use std::ascii;
use vga::input::NumCode;

use crate::{input::{Input, ControlInfo, read_control, ControlDirection}, start::quit, us1::print, user::rnd_t, vga_render::VGARenderer, vh::{vw_hlin, vw_vlin}, vl::fade_in};
use crate::time::Ticker;
use crate::def::{WindowState, ItemType, MenuItem};
use crate::assets::GraphicNum;

const STRIPE : u8 = 0x2c;
const BORDER_COLOR : u8 = 0x29;
const BORDER2_COLOR : u8 = 0x23;
const DEACTIVE : u8 = 0x2b;
const BKGD_COLOR : u8 = 0x2d;

const READ_COLOR : u8 = 0x4a;
const READ_HCOLOR : u8 = 0x47;
const TEXT_COLOR : u8 = 0x17;
const HIGHLIGHT : u8 = 0x13;

pub const MENU_X : usize = 76;
pub const MENU_Y : usize = 55;
const MENU_W : usize = 178;
const MENU_H : usize = 13*10+6;

static END_STRINGS : [&'static str; 9] = [
    "Dost thou wish to\nleave with such hasty\nabandon?",
	"Chickening out...\nalready?",
	"Press N for more carnage.\nPress Y to be a weenie.",
	"So, you think you can\nquit this easily, huh?",
	"Press N to save the world.\nPress Y to abandon it in\nits hour of need.",
	"Press N if you are brave.\nPress Y to cower in shame.",
	"Heroes, press N.\nWimps, press Y.",
	"You are at an intersection.\nA sign says, 'Press Y to quit.'\n>",
	"For guns and glory, press N.\nFor work and worry, press Y."
];

fn menu_item_pos(win_state: &WindowState, which_pos: usize) -> Option<MenuItem> {
    for t in &win_state.main_menu {
        if t.item.pos() == which_pos {
            return Some(t.item)
        }
    }
    None
}

static COLOR_HLITE : [u8; 4] = [
    DEACTIVE,
    HIGHLIGHT,
    READ_HCOLOR,
    0x67,
];

static COLOR_NORML : [u8; 4] = [
    DEACTIVE,
    TEXT_COLOR,
    READ_COLOR,
    0x6b,
];

/// Wolfenstein Control Panel!  Ta Da!
pub async fn control_panel(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, scan: NumCode) {
    // TODO scan code handling
    // TODO StartCPMusic(MENUSONG)

    setup_control_panel(win_state);

    draw_main_menu(rdr, win_state);
    rdr.fade_in().await;

    // MAIN MENU LOOP
    loop {
       let which = handle_menu(ticker, rdr, input, win_state).await;
       println!("which = {:?}", which);
       match which {
        Some(MenuItem::ViewScores) => {

        },
        Some(MenuItem::BackToDemo) => {
            break;
        },
        None|Some(MenuItem::Quit) => {
            menu_quit(ticker, rdr, input, win_state).await;
        },
        _ => {
            draw_main_menu(rdr, win_state);
            rdr.fade_in().await; 
        }
       }
    }

    // RETURN/START GAME EXECUTION
}

async fn menu_quit(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    let text = END_STRINGS[((rnd_t()&0x07)+(rnd_t()&1)) as usize];
    if confirm(ticker, rdr, input, win_state, text).await {
        //TODO stop music
        rdr.fade_in().await;
        quit(None)
    }

    draw_main_menu(rdr, win_state)
}

async fn confirm(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, str: &str) -> bool {
    message(rdr, win_state, str);
    input.clear_keys_down();

    // BLINK CURSOR
    let x = win_state.print_x;
    let y = win_state.print_y;
    let mut tick = false;
    let mut time_count = 0;
    while !input.key_pressed(NumCode::Y) && !input.key_pressed(NumCode::N) && !input.key_pressed(NumCode::Escape) {
        if time_count >= 10 {
            if tick {
                rdr.bar(x, y, 8, 13, TEXT_COLOR);
            } else {
                win_state.print_x = x;
                win_state.print_y = y;
                print(rdr, win_state, "_")
            }
            tick = !tick;
            time_count = 0;
        }
        
        ticker.tics(1).await;
        time_count += 1;
    }

    let exit = if input.key_pressed(NumCode::Y) {
        // TODO ShootSnd
        true
    } else {
        false
    };

    input.clear_keys_down();
    // TODO SDPLaySound(whichsnd[exit])

    exit
}

async fn handle_menu(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) -> Option<MenuItem> {
    let mut which_pos = win_state.main_state.cur_pos.pos();
    let x = win_state.main_state.x & 8_usize.wrapping_neg();
    let base_y = win_state.main_state.y - 2;
    let mut y = base_y + which_pos * 13;

    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);

    let mut shape = GraphicNum::CCURSOR1PIC;
    let mut timer = 8;

    input.clear_keys_down();

    let exit;
    loop {
        // CHANGE GUN SHAPE
        if ticker.get_count() > timer {
            ticker.clear_count();
            if shape == GraphicNum::CCURSOR1PIC {
                shape = GraphicNum::CCURSOR2PIC;
                timer = 8;
            } else {
                shape = GraphicNum::CCURSOR1PIC;
                timer = 70;
            }
            rdr.pic(x, y, shape);
            // TODO call routine?
        }

        // TODO CheckPause

        // TODO check key presses

        let ci = read_any_control(input);
        
        match ci.dir {
            ControlDirection::North => {
                erase_gun(rdr, win_state, x, y, which_pos);

                if which_pos > 0 && win_state.main_menu[which_pos-1].active {
                    y -= 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == 0 {
                        which_pos = win_state.main_menu.len()-1;
                    } else {
                        which_pos -= 1;
                    }

                    if win_state.main_menu[which_pos].active {
                        break;
                    }  
                }
                y = draw_gun(rdr, win_state, x, y, which_pos, base_y);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            ControlDirection::South => {
                erase_gun(rdr, win_state, x, y, which_pos);

                if which_pos != win_state.main_menu.len()-1 && win_state.main_menu[which_pos+1].active {
                    y += 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == win_state.main_menu.len() - 1 {
                        which_pos = 0;
                    } else {
                        which_pos += 1;
                    }

                    if win_state.main_menu[which_pos].active {
                        break;
                    }
                }
                y = draw_gun(rdr, win_state, x, y, which_pos, base_y);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            _ => { /* ignore */ },
        }

        if input.key_pressed(NumCode::Space) || input.key_pressed(NumCode::Return) {
            exit = 1;
            break;
        }
        if input.key_pressed(NumCode::Escape) {
            exit = 2;
            break;
        }
    }

    input.clear_keys_down();

    win_state.main_state.cur_pos = menu_item_pos(win_state, which_pos).unwrap_or(MenuItem::NewGame);

    if exit == 1 {
        return menu_item_pos(win_state, which_pos);
    }
    if exit == 2 { //ESC
        return None
    }

    return Some(MenuItem::NewGame);
}

async fn tic_delay(ticker: &Ticker, input: &Input, count: u64) {
    input.clear_keys_down();
    for _ in 0..count {
        let ci = read_any_control(input);
        if ci.dir != ControlDirection::None {
            break;
        }
        ticker.tics(1).await
    }
}

async fn draw_half_step(ticker: &Ticker, rdr: &VGARenderer, x: usize, y: usize) {
    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);
    // TODO SD_PlaySound(MOVEGUN1SND)

    ticker.tics(8).await;
}

fn erase_gun(rdr: &VGARenderer, win_state: &mut WindowState, x: usize, y: usize, which_pos: usize) {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    set_text_color(win_state, which_pos, false);

    win_state.print_x = win_state.main_state.x + win_state.main_state.indent;
    win_state.print_y = win_state.main_state.y + which_pos * 13;
    print(rdr, win_state, win_state.main_menu[which_pos].string); 
}

fn draw_gun(rdr: &VGARenderer, win_state: &mut WindowState, x: usize, y: usize, which_pos: usize, base_y: usize) -> usize {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which_pos * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);
    set_text_color(win_state, which_pos, true);

    win_state.print_x = win_state.main_state.x + win_state.main_state.indent;
    win_state.print_y = win_state.main_state.y + which_pos * 13;
    print(rdr, win_state, win_state.main_menu[which_pos].string);

    // TODO call custom routine?
    // TODO PlaySound(MOVEGUN2SND)
    new_y
}

fn read_any_control(input: &Input) -> ControlInfo {
    read_control(input)
}

fn setup_control_panel(win_state: &mut WindowState) {
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.font_number = 1;
    win_state.window_h = 200;
}

fn draw_main_menu(rdr: &VGARenderer, win_state: &mut WindowState) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);
    draw_stripes(rdr, 10);
    rdr.pic(84, 0, GraphicNum::COPTIONSPIC);

    draw_window(rdr, MENU_X-8, MENU_Y-3, MENU_W, MENU_H, BKGD_COLOR);

    // TODO handle ingame menue here

    draw_menu(rdr, win_state);
}

fn draw_menu(rdr: &VGARenderer, win_state: &mut WindowState) {
    let which = win_state.main_state.cur_pos;

    let x = win_state.main_state.x + win_state.main_state.indent;
    win_state.window_x = x;
    win_state.print_x = x;
    win_state.window_y = win_state.main_state.y;
    win_state.print_y = win_state.main_state.y;
    win_state.window_w = 320;
    win_state.window_h = 200;

    for i in 0..win_state.main_menu.len() {
        set_text_color(win_state, i, which.pos() == i);

        win_state.print_y = win_state.main_state.y + i * 13;
        if win_state.main_menu[i].active {
            print(rdr, win_state, win_state.main_menu[i].string);
        } else {
            win_state.set_font_color(DEACTIVE, BKGD_COLOR);
            print(rdr, win_state, win_state.main_menu[i].string); 
            win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
        }

        print(rdr, win_state, "\n");
    }
}

fn set_text_color(win_state: &mut WindowState, which: usize, hlight: bool) {
    if hlight {
        win_state.set_font_color(COLOR_HLITE[if win_state.main_menu[which].active {1} else {0}], BKGD_COLOR)
    } else {
        win_state.set_font_color(COLOR_NORML[if win_state.main_menu[which].active {1} else {0}], BKGD_COLOR)
    }
}

pub fn draw_stripes(rdr: &VGARenderer, y: usize) {
    rdr.bar(0, y, 320, 24, 0);
    rdr.hlin(0, 319, y+22, STRIPE);
}

pub fn clear_ms_screen(rdr: &VGARenderer) {
    rdr.bar(0, 0, 320, 200, BORDER_COLOR)
}

/// The supplied message should only contain ASCII characters.
/// All other characters are not supported and ignored.
pub fn message(rdr: &VGARenderer, win_state: &mut WindowState, str: &str) {
    win_state.font_number = 1;
    win_state.font_color = 0;
    let font = &rdr.fonts[win_state.font_number];
    let mut h = font.height as usize;
    let mut w : usize = 0;
    let mut mw : usize = 0;
    for c in str.chars() {
        if let Some(ascii_char) = c.as_ascii() {
            if ascii_char == ascii::Char::LineFeed {
                if w > mw {
                    mw = w;
                }
                w = 0;
                h += font.height as usize;
            } else {
                w += font.width[ascii_char as usize] as usize;
            }
        }
    }

    if w+10 > mw {
        mw = w + 10;
    }

    win_state.print_y = (win_state.window_h/2)-h/2;
    win_state.window_x = 160-mw/2;
    win_state.print_x = win_state.window_x;
    
    let prev_buffer = rdr.buffer_offset();
    rdr.set_buffer_offset(rdr.active_buffer());
    draw_window(rdr, win_state.window_x-5, win_state.print_y-5, mw+10, h+10, TEXT_COLOR);
    draw_outline(rdr, win_state.window_x-5, win_state.print_y-5, mw+10, h+10, 0, HIGHLIGHT);
    print(rdr, win_state, str);
    rdr.set_buffer_offset(prev_buffer);
}

pub fn draw_window(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color: u8) {
    rdr.bar(x, y, width, height, color);
    draw_outline(rdr, x, y, width, height, BORDER2_COLOR, DEACTIVE);
}

pub fn draw_outline(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color1: u8, color2: u8) {
    vw_hlin(rdr, x, x+width, y, color2);
    vw_vlin(rdr, y, y+height, x, color2);
    vw_hlin(rdr, x,x+width,y+height, color1);
    vw_vlin(rdr, y, y+height, x+width, color1);
}