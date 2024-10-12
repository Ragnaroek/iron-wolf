
use std::ascii;
use vga::input::NumCode;

use crate::{assets::GraphicNum, input::{Input, ControlInfo, read_control, ControlDirection}, start::quit, us1::print, user::rnd_t, vga_render::VGARenderer, vh::{vw_hlin, vw_vlin}, vl::fade_in};
use crate::time::Ticker;
use crate::def::WindowState;

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
    pub cur_pos: MenuItem,
    pub indent: usize,
}

struct ItemType {
    pub active: bool,
    pub string: &'static str,
    pub item: MenuItem,
    // TODO action pointer func
}

// usize = position in the main menu of the entry
#[derive(Copy, Clone, Debug)]
#[repr(usize)]
enum MenuItem {
    NewGame = 0,
    Sound = 1,
    Control = 2,
    LoadGame = 3,
    SaveGame = 4,
    ChangeView = 5,
    ViewScores = 6,
    BackToDemo = 7,
    Quit = 8,
}

impl MenuItem {
    fn pos(self) -> usize {
        self as usize
    }
}

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

static MAIN_MENU : [ItemType; 9] = [
    ItemType{item: MenuItem::NewGame, active: true, string: "New Game"},
    ItemType{item: MenuItem::Sound, active: true, string: "Sound"},
    ItemType{item: MenuItem::Control, active: true, string: "Control"},
    ItemType{item: MenuItem::LoadGame, active: true, string: "Load Game"},
    ItemType{item: MenuItem::SaveGame, active: true, string: "Save Game"},
    ItemType{item: MenuItem::ChangeView, active: true, string: "Change View"},
    ItemType{item: MenuItem::ViewScores, active: true, string: "View Scores"},
    ItemType{item: MenuItem::BackToDemo, active: true, string: "Back to Demo"},
    ItemType{item: MenuItem::Quit, active: true, string: "Quit"},
];

fn menu_item_pos(which_pos: usize) -> Option<MenuItem> {
    for t in &MAIN_MENU {
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

static MAIN_ITEMS : ItemInfo = ItemInfo{x: MENU_X, y: MENU_Y, cur_pos: MenuItem::NewGame, indent: 24}; // TODO define START_ITEM

/// Wolfenstein Control Panel!  Ta Da!
pub async fn control_panel(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, scan: NumCode) {
    // TODO scan code handling
    // TODO StartCPMusic(MENUSONG)

    setup_control_panel(win_state);

    draw_main_menu(rdr, win_state);
    rdr.fade_in().await;

    // MAIN MENU LOOP
    loop {
       let which = handle_menu(ticker, rdr, input, win_state, &MAIN_ITEMS, &MAIN_MENU).await;
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

async fn handle_menu(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType]) -> Option<MenuItem> {
    let mut which_pos = item_info.cur_pos.pos();
    let x = item_info.x & 8_usize.wrapping_neg();
    let base_y = item_info.y - 2;
    let mut y = base_y + which_pos * 13;

    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);

    input.clear_keys_down();

    let exit;
    loop {
        // TODO Animate gun

        // TODO CheckPause

        // TODO check key press

        let ci = read_any_control(input);
        
        match ci.dir {
            ControlDirection::North => {
                erase_gun(rdr, win_state, item_info, items, x, y, which_pos);

                if which_pos > 0 && items[which_pos-1].active {
                    y -= 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == 0 {
                        which_pos = items.len()-1;
                    } else {
                        which_pos -= 1;
                    }

                    if items[which_pos].active {
                        break;
                    }  
                }
                y = draw_gun(rdr, win_state, item_info, items, x, y, which_pos, base_y);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            ControlDirection::South => {
                erase_gun(rdr, win_state, item_info, items, x, y, which_pos);

                if which_pos != items.len()-1 && items[which_pos+1].active {
                    y += 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == items.len() - 1 {
                        which_pos = 0;
                    } else {
                        which_pos += 1;
                    }

                    if items[which_pos].active {
                        break;
                    }
                }
                y = draw_gun(rdr, win_state, item_info, items, x, y, which_pos, base_y);

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
        return menu_item_pos(which_pos);
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

fn erase_gun(rdr: &VGARenderer, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType], x: usize, y: usize, which_pos: usize) {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    set_text_color(win_state, &items[which_pos], false);

    win_state.print_x = item_info.x + item_info.indent;
    win_state.print_y = item_info.y + which_pos * 13;
    print(rdr, win_state, items[which_pos].string); 
}

fn draw_gun(rdr: &VGARenderer, win_state: &mut WindowState, item_info: &ItemInfo, items: &[ItemType], x: usize, y: usize, which_pos: usize, base_y: usize) -> usize {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which_pos * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);
    set_text_color(win_state, &items[which_pos], true);

    win_state.print_x = item_info.x + item_info.indent;
    win_state.print_y = item_info.y + which_pos * 13;
    print(rdr, win_state, items[which_pos].string);

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
        set_text_color(win_state, item, which.pos() == i);

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