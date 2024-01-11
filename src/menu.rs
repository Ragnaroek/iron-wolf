
use std::ascii;
use vga::input::NumCode;

use crate::{vga_render::VGARenderer, def::WindowState, vh::{vw_hlin, vw_vlin}, us1::print, input::{Input, ControlInfo, read_control, ControlDirection}, assets::GraphicNum};
use crate::time::Ticker;

const STRIPE : u8 = 0x2c;
const BORDER_COLOR : u8 = 0x29;
const BORDER2_COLOR : u8 = 0x23;
const DEACTIVE : u8 = 0x2b;
const BKGD_COLOR : u8 = 0x2d;

const READ_COLOR : u8 = 0x4a;
const READ_HCOLOR : u8 = 0x47;
const TEXT_COLOR : u8 = 0x17;
const HIGHLIGHT : u8 = 0x13;

const MENU_X : usize = 76;
const MENU_Y : usize = 55;
const MENU_W : usize = 178;
const MENU_H : usize = 13*10+6;

struct ItemInfo {
    pub x: usize,
    pub y: usize,
    pub cur_pos: usize,
    pub indent: usize,
}

struct ItemType {
    pub active: bool,
    pub string: &'static str,
    // TODO action pointer func
}

static MAIN_MENU : [ItemType; 9] = [
    ItemType{active: true, string: "New Game"},
    ItemType{active: true, string: "Sound"},
    ItemType{active: true, string: "Control"},
    ItemType{active: true, string: "Load Game"},
    ItemType{active: true, string: "Save Game"},
    ItemType{active: true, string: "Change View"},
    ItemType{active: true, string: "View Scores"},
    ItemType{active: true, string: "Back to Demo"},
    ItemType{active: true, string: "Quit"},
];

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

static MAIN_ITEMS : ItemInfo = ItemInfo{x: MENU_X, y: MENU_Y, cur_pos: 0, indent: 24}; // TODO define START_ITEM


/// Wolfenstein Control Panel!  Ta Da!
pub async fn control_panel(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, scan: NumCode) {
    // TODO scan code handling
    // TODO StartCPMusic(MENUSONG)

    setup_control_panel(win_state);

    draw_main_menu(rdr, win_state);
    rdr.fade_in().await;

    // MAIN MENU LOOP
    loop {
       handle_menu(ticker, rdr, input, win_state, &MAIN_ITEMS, &MAIN_MENU).await;
    }

    input.ack().await; //tmp. until menu handling implemented
}

async fn handle_menu(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType]) -> isize {
    let mut which = item_info.cur_pos;
    let x = item_info.x & 8_usize.wrapping_neg();
    let base_y = item_info.y - 2;
    let mut y = base_y + which * 13;

    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);

    input.clear_keys_down();

    let mut exit = 0;
    loop {
        // TODO Animate gun

        // TODO CheckPause

        // TODO check key press

        let ci = read_any_control(input);
        
        match ci.dir {
            ControlDirection::North => {
                erase_gun(rdr, win_state, item_info, items, x, y, which);

                if which > 0 && items[which-1].active {
                    y -= 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which == 0 {
                        which = items.len()-1;
                    } else {
                        which -= 1;
                    }

                    if items[which].active {
                        break;
                    }  
                }
                y = draw_gun(rdr, win_state, item_info, items, x, y, which, base_y);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            ControlDirection::South => {
                erase_gun(rdr, win_state, item_info, items, x, y, which);

                if which != items.len()-1 && items[which+1].active {
                    y += 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which == items.len() - 1 {
                        which = 0;
                    } else {
                        which += 1;
                    }

                    if items[which].active {
                        break;
                    }
                }
                y = draw_gun(rdr, win_state, item_info, items, x, y, which, base_y);

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

    if exit == 1 {
        return which as isize;
    }
    if exit == 2 { //ESC
        return -1;
    }

    return 0;
}

async fn tic_delay(ticker: &Ticker, input: &Input, count: u64) {
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

fn erase_gun(rdr: &VGARenderer, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType], x: usize, y: usize, which: usize) {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    set_text_color(win_state, &items[which], false);

    win_state.print_x = item_info.x + item_info.indent;
    win_state.print_y = item_info.y + which * 13;
    print(rdr, win_state, items[which].string); 
}

fn draw_gun(rdr: &VGARenderer, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType], x: usize, y: usize, which: usize, base_y: usize) -> usize {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);
    set_text_color(win_state, &items[which], true);

    win_state.print_x = item_info.x + item_info.indent;
    win_state.print_y = item_info.y + which * 13;
    print(rdr, win_state, items[which].string);

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

    draw_menu(rdr, win_state, &MAIN_ITEMS, &MAIN_MENU);
}

fn draw_menu(rdr: &VGARenderer, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType]) {
    let which = item_info.cur_pos;

    let x = item_info.x + item_info.indent;
    win_state.window_x = x;
    win_state.print_x = x;
    win_state.window_y = item_info.y;
    win_state.print_y = item_info.y;
    win_state.window_w = 320;
    win_state.window_h = 200;

    for i in 0..items.len() {
        let item = &items[i];
        set_text_color(win_state, item, which == i);

        win_state.print_y = item_info.y + i * 13;
        if item.active {
            print(rdr, win_state, item.string);
        } else {
            win_state.set_font_color(DEACTIVE, BKGD_COLOR);
            print(rdr, win_state, item.string); 
            win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
        }

        print(rdr, win_state, "\n");
    }
}

fn set_text_color(win_state: &mut WindowState, item: &ItemType, hlight: bool) {
    if hlight {
        win_state.set_font_color(COLOR_HLITE[if item.active {1} else {0}], BKGD_COLOR)
    } else {
        win_state.set_font_color(COLOR_NORML[if item.active {1} else {0}], BKGD_COLOR)
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