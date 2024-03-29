use std::{ascii, collections::HashMap};
use vga::input::NumCode;

use crate::input::{Input, ControlInfo, read_control, ControlDirection};
use crate::start::quit;
use crate::us1::{c_print, print};
use crate::user::rnd_t;
use crate::vga_render::VGARenderer;
use crate::vh::{vw_hlin, vw_vlin};
use crate::time::Ticker;
use crate::def::{Difficulty, GameState, WindowState};
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

const NM_X : usize = 50;
const NM_Y : usize = 100;
const NM_W : usize = 225;
const NM_H : usize = 13 * 4 + 15;

const NE_X : usize = 10;
const NE_Y : usize = 23;
const NE_W : usize = 320 - NE_X * 2;
const NE_H : usize = 200 - NE_Y * 2;

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

pub struct ItemInfo {
    pub x: usize,
    pub y: usize,
    pub cur_pos: usize,
    pub indent: usize,
}

pub struct ItemType {
    pub active: bool,
    pub string: &'static str,
    pub item: usize,
}

// usize = position in the main menu of the entry
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[repr(usize)]
pub enum MainMenuItem {
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

impl MainMenuItem {
    pub fn pos(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
enum EpisodeItem {
    Episode1 = 0,
    Episode2 = 1,
    Episode3 = 2,
    Episode4 = 3,
    Episode5 = 4,
    Episode6 = 5,
}

impl EpisodeItem {
    pub fn pos(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
enum DifficultyItem {
    Daddy,
    HurtMe,
    BringEmOn,
    Death,
}

impl DifficultyItem {
    pub fn pos(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MenuHandle {
    None,
    OpenMenu(Menu),
    Selected(usize),
    QuitMenu,
    BackToGameLoop,
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum Menu {
    Top,
    MainMenu(MainMenuItem),
    DifficultySelect,
}

pub struct MenuStateEntry {
    pub items: Vec<ItemType>,
    pub state: ItemInfo,
}

pub struct MenuState {
    pub selected: Menu,
    pub menues: HashMap<Menu, MenuStateEntry>,
}

impl MenuState {
    pub fn select_menu(&mut self, menu: Menu) {
        self.selected = menu;
    }

    pub fn selected_state(&self) -> &MenuStateEntry {
        &self.menues[&self.selected]        
    }

    pub fn update_selected<F>(&mut self, f: F) 
    where F: FnOnce(&mut MenuStateEntry)
    {
        self.update_menu(self.selected, f)
    }

    pub fn update_menu<F>(&mut self, menue: Menu, f: F)
    where F: FnOnce(&mut MenuStateEntry)
    {
        let entry_opt = self.menues.get_mut(&menue);
        if let Some(state) = entry_opt {
            f(state)
        }
    } 
}

type MenuRoutine = fn(&VGARenderer, usize);

fn no_op_routine(_rdr: &VGARenderer, _which: usize) {}

fn initial_main_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType{item: MainMenuItem::NewGame.pos(), active: true, string: "New Game"},
            ItemType{item: MainMenuItem::Sound.pos(), active: true, string: "Sound"},
            ItemType{item: MainMenuItem::Control.pos(), active: true, string: "Control"},
            ItemType{item: MainMenuItem::LoadGame.pos(), active: true, string: "Load Game"},
            ItemType{item: MainMenuItem::SaveGame.pos(), active: true, string: "Save Game"},
            ItemType{item: MainMenuItem::ChangeView.pos(), active: true, string: "Change View"},
            ItemType{item: MainMenuItem::ViewScores.pos(), active: true, string: "View Scores"},
            ItemType{item: MainMenuItem::BackToDemo.pos(), active: true, string: "Back to Demo"},
            ItemType{item: MainMenuItem::Quit.pos(), active: true, string: "Quit"},
        ],
        state: ItemInfo{x: MENU_X, y: MENU_Y, cur_pos: MainMenuItem::NewGame.pos(), indent: 24},
    }
}

fn initial_episode_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType{item: EpisodeItem::Episode1.pos(), active: true, string: "Episode 1\nEscape from Wolfenstein"},
            placeholder(),
            ItemType{item: EpisodeItem::Episode2.pos(), active: false, string: "Episode 2\nOperation: Eisenfaust"},
            placeholder(),
            ItemType{item: EpisodeItem::Episode3.pos(), active: false, string: "Episode 3\nDie, Fuhrer, Die!"},
            placeholder(),
            ItemType{item: EpisodeItem::Episode4.pos(), active: false, string: "Episode 4\nA Dark Secret"},
            placeholder(),
            ItemType{item: EpisodeItem::Episode5.pos(), active: false, string: "Episode 5\nTrail of the Madman"},
            placeholder(),
            ItemType{item: EpisodeItem::Episode6.pos(), active: false, string: "Episode 6\nConfrontation"},
        ],
        state: ItemInfo{x: NE_X, y: NE_Y, cur_pos: EpisodeItem::Episode1.pos(), indent: 88 },
    }
}

fn placeholder() -> ItemType {
    ItemType{item: 0, active: false, string: ""}
}

fn initial_difficulty_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType{item: DifficultyItem::Daddy.pos(), active: true, string: "Can I play, Daddy?"},
            ItemType{item: DifficultyItem::HurtMe.pos(), active: true, string: "Don't hurt me."},
            ItemType{item: DifficultyItem::BringEmOn.pos(), active: true, string: "Bring 'em on!"},
            ItemType{item: DifficultyItem::Death.pos(), active: true, string: "I am Death incarnate!"},

        ],
        state: ItemInfo{x: NM_X, y: NM_Y, cur_pos: DifficultyItem::Daddy.pos(), indent: 24},
    }
}

pub fn initial_menu_state() -> MenuState {
    MenuState {
        selected: Menu::Top,
        menues: HashMap::from([
            (Menu::Top, initial_main_menu()), 
            (Menu::MainMenu(MainMenuItem::NewGame), initial_episode_menu()),
            (Menu::DifficultySelect, initial_difficulty_menu()),
        ]),
    }
}

/// Wolfenstein Control Panel!  Ta Da!
pub async fn control_panel(ticker: &Ticker, game_state: &mut GameState, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState, scan: NumCode) {
    // TODO scan code handling
    // TODO StartCPMusic(MENUSONG)

    setup_control_panel(win_state);

    let mut menu_stack: Vec<Menu> = Vec::new();
    menu_stack.push(menu_state.selected);

    // MAIN MENU LOOP
    loop {
        // TODO Put this loop into cp_main_menu function and execute that if Menu::Top is on top of stack!
        // TODO call menu function that is on top. quit loop if stack gets empty.
        let menu_opt = menu_stack.last();
        if let Some(menu) = menu_opt {
            let handle = match menu {
                Menu::Top => cp_main_menu(ticker, rdr, input, win_state, menu_state).await,
                Menu::MainMenu(item) => {
                    match item {
                        MainMenuItem::NewGame => cp_new_game(ticker, game_state, rdr, input, win_state, menu_state).await,
                        _ => todo!("implement other menu selects"),
                    }
                },
                Menu::DifficultySelect => cp_difficulty_select(ticker, game_state, rdr, input, win_state, menu_state).await,
            };
            match handle {
                MenuHandle::OpenMenu(menu) => menu_stack.push(menu),
                MenuHandle::QuitMenu => {
                    menu_stack.pop();
                },
                MenuHandle::BackToGameLoop => {
                    break;
                },
                _ => {/* ignore */},
            }
        } else {
            return // back to game loop
        }
    }
    // RETURN/START GAME EXECUTION
}

async fn cp_main_menu(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState) -> MenuHandle {
    draw_main_menu(rdr, win_state, menu_state);
    rdr.fade_in().await;
    
    let handle = handle_menu(ticker, rdr, input, win_state, menu_state, no_op_routine).await;

    if handle == MenuHandle::Selected(MainMenuItem::NewGame.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::NewGame));
    } else if handle == MenuHandle::Selected(MainMenuItem::ViewScores.pos()) {
        todo!("show view scores");
    } else if handle == MenuHandle::Selected(MainMenuItem::BackToDemo.pos()) {
        return MenuHandle::BackToGameLoop;
    } else if handle == MenuHandle::QuitMenu || handle == MenuHandle::Selected(MainMenuItem::Quit.pos()) {
        menu_quit(ticker, rdr, input, win_state, menu_state).await;
        return MenuHandle::QuitMenu;
    } else {
        return handle;
    }
}

async fn cp_new_game(ticker: &Ticker, game_state: &mut GameState, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState) -> MenuHandle {
    loop {
        draw_new_episode(rdr, win_state, menu_state).await;

        let episode_handle = handle_menu(ticker, rdr, input, win_state, menu_state, no_op_routine).await;
        if let MenuHandle::Selected(episode_selected) = episode_handle {
            //TODO SD_PlaySound(SHOOTSND)
            //TODO confirm dialog if already in a game
            game_state.episode = episode_selected / 2;
            return MenuHandle::OpenMenu(Menu::DifficultySelect);

        } else {
            rdr.fade_out().await;
            return episode_handle;
        }
    }
}

fn draw_new_game_diff(rdr: &VGARenderer, which: usize) {
   rdr.pic(NM_X+185, NM_Y+7, difficulty_pic(which)); 
}

async fn cp_difficulty_select(ticker: &Ticker, game_state: &mut GameState, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState) -> MenuHandle {
    draw_difficulty_select(rdr, win_state, menu_state).await;
    let handle = handle_menu(ticker, rdr, input, win_state, menu_state, draw_new_game_diff).await;

    if let MenuHandle::Selected(diff_selected) = handle {
        // TODO SD_PlaySound(SHOOTSND)
        game_state.prepare_episode_select();
        game_state.difficulty = difficulty(diff_selected);

        rdr.fade_out().await;
        return MenuHandle::BackToGameLoop;
    }
    rdr.fade_out().await;
    handle
}

async fn draw_difficulty_select(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &mut MenuState) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, NM_X-5, NM_Y-10, NM_W, NM_H, BKGD_COLOR);
    win_state.set_font_color(READ_HCOLOR, BKGD_COLOR);
    win_state.print_x = NM_X + 20;
    win_state.print_y = NM_Y - 32;
    print(rdr, win_state, "How tough are you?");

    menu_state.select_menu(Menu::DifficultySelect);
    draw_menu(rdr, win_state, menu_state);

    menu_state.selected_state().state.cur_pos;
    rdr.pic(NM_X+185, NM_Y+7, difficulty_pic(menu_state.selected_state().state.cur_pos));
    rdr.fade_in().await;
}

fn difficulty_pic(i: usize) -> GraphicNum {
    match i {
        0 => GraphicNum::CBABYMODEPIC,
        1 => GraphicNum::CEASYPIC,
        2 => GraphicNum::CNORMALPIC,
        3 => GraphicNum::CHARDPIC,
        _ => GraphicNum::CBABYMODEPIC,
    }
}

fn difficulty(i: usize) -> Difficulty {
    match i {
        0 => Difficulty::Baby,
        1 => Difficulty::Easy,
        2 => Difficulty::Hard,
        3 => Difficulty::Hard,
        _ => Difficulty::Baby,
    } 
}

async fn draw_new_episode(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &mut MenuState) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, NE_X-4, NE_Y-4, NE_W+8, NE_H+8, BKGD_COLOR);
    win_state.set_font_color(READ_HCOLOR, BKGD_COLOR);
    win_state.print_y = 2;
    win_state.window_x = 0;
    c_print(rdr, win_state, "Which episode to play?");

    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    menu_state.select_menu(Menu::MainMenu(MainMenuItem::NewGame));
    draw_menu(rdr, win_state, menu_state);

    for i in 0..6 {
        rdr.pic(NE_X+32, NE_Y+i*26, episode_pic(i))
    }

    rdr.fade_in().await;
}

fn episode_pic(i: usize) -> GraphicNum {
    match i {
        0 => GraphicNum::CEPISODE1PIC,
        1 => GraphicNum::CEPISODE2PIC,
        2 => GraphicNum::CEPISODE3PIC,
        3 => GraphicNum::CEPISODE4PIC,
        4 => GraphicNum::CEPISODE5PIC,
        5 => GraphicNum::CEPISODE6PIC,
        _ => GraphicNum::CEPISODE1PIC,
    }
}

async fn menu_quit(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState) {
    let text = END_STRINGS[((rnd_t()&0x07)+(rnd_t()&1)) as usize];
    if confirm(ticker, rdr, input, win_state, text).await {
        //TODO stop music
        rdr.fade_in().await;
        quit(None)
    }

    draw_main_menu(rdr, win_state, menu_state)
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

/// Handle moving gun around a menu
async fn handle_menu(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState, routine: MenuRoutine) -> MenuHandle {
    let handle = handle_menu_loop(ticker, rdr, input, win_state, menu_state, routine).await;

    input.clear_keys_down();

    if let MenuHandle::Selected(which_pos) = handle {
        menu_state.update_selected(|selected| selected.state.cur_pos = which_pos);
        //let items = &menu_state.selected_state().items;
        //let item_type = &items[which_pos];
        //return item_type.handle;
    }
    handle
}

async fn handle_menu_loop(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, menu_state: &mut MenuState, routine: MenuRoutine) -> MenuHandle {
    let selected = menu_state.selected_state();

    let mut which_pos = selected.state.cur_pos;
    let x = selected.state.x & 8_usize.wrapping_neg();
    let base_y = selected.state.y - 2;
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
            routine(rdr, which_pos);
        }

        // TODO CheckPause

        // TODO check key presses

        let ci = read_any_control(input);
        
        match ci.dir {
            ControlDirection::North => {
                erase_gun(rdr, win_state, selected, x, y, which_pos);

                if which_pos > 0 && selected.items[which_pos-1].active {
                    y -= 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == 0 {
                        which_pos = selected.items.len()-1;
                    } else {
                        which_pos -= 1;
                    }

                    if selected.items[which_pos].active {
                        break;
                    }  
                }
                y = draw_gun(rdr, win_state, selected, x, y, which_pos, base_y, routine);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            ControlDirection::South => {
                erase_gun(rdr, win_state, selected, x, y, which_pos);

                if which_pos != selected.items.len()-1 && selected.items[which_pos+1].active {
                    y += 6;
                    draw_half_step(ticker, rdr, x, y).await;
                }

                loop {
                    if which_pos == selected.items.len() - 1 {
                        which_pos = 0;
                    } else {
                        which_pos += 1;
                    }

                    if selected.items[which_pos].active {
                        break;
                    }
                }
                y = draw_gun(rdr, win_state, selected, x, y, which_pos, base_y, routine);

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            },
            _ => { /* ignore */ },
        }

        if input.key_pressed(NumCode::Space) || input.key_pressed(NumCode::Return) {
            exit = MenuHandle::Selected(which_pos); 
            break;
        }
        if input.key_pressed(NumCode::Escape) {
            exit = MenuHandle::QuitMenu;
            break;
        }
    }
    return exit
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

fn erase_gun(rdr: &VGARenderer, win_state: &mut WindowState, selected: &MenuStateEntry, x: usize, y: usize, which_pos: usize) {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    set_text_color(win_state, &selected.items, which_pos, false);

    win_state.print_x = selected.state.x + selected.state.indent;
    win_state.print_y = selected.state.y + which_pos * 13;
    print(rdr, win_state, selected.items[which_pos].string); 
}

fn draw_gun(rdr: &VGARenderer, win_state: &mut WindowState, selected: &MenuStateEntry, x: usize, y: usize, which_pos: usize, base_y: usize, routine: MenuRoutine) -> usize {
    rdr.bar(x-1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which_pos * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);
    set_text_color(win_state, &selected.items, which_pos, true);

    win_state.print_x = selected.state.x + selected.state.indent;
    win_state.print_y = selected.state.y + which_pos * 13;
    print(rdr, win_state, selected.items[which_pos].string);

    routine(rdr, which_pos);

    // TODO PlaySound(MOVEGUN2SND)
    new_y
}

fn read_any_control(input: &Input) -> ControlInfo {
    read_control(input)
    // TODO also read mouse and jostick input
}

fn setup_control_panel(win_state: &mut WindowState) {
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.font_number = 1;
    win_state.window_h = 200;
}

fn draw_main_menu(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &mut MenuState) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);
    draw_stripes(rdr, 10);
    rdr.pic(84, 0, GraphicNum::COPTIONSPIC);

    cp_draw_window(rdr, MENU_X-8, MENU_Y-3, MENU_W, MENU_H, BKGD_COLOR);

    menu_state.select_menu(Menu::Top);
    draw_menu(rdr, win_state, menu_state);
}

fn draw_menu(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &MenuState) {
    let selected = menu_state.selected_state();
    let which = selected.state.cur_pos;

    let x = selected.state.x + selected.state.indent;
    win_state.window_x = x;
    win_state.print_x = x;
    win_state.window_y = selected.state.y;
    win_state.print_y = selected.state.y;
    win_state.window_w = 320;
    win_state.window_h = 200;
    
    for i in 0..selected.items.len() {
        set_text_color(win_state, &selected.items, i, which == i);

        win_state.print_y = selected.state.y + i * 13;
        if selected.items[i].active {
            print(rdr, win_state, selected.items[i].string);
        } else {
            win_state.set_font_color(DEACTIVE, BKGD_COLOR);
            print(rdr, win_state, selected.items[i].string); 
            win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
        }

        print(rdr, win_state, "\n");
    }
}

fn set_text_color(win_state: &mut WindowState, items: &[ItemType], which: usize, hlight: bool) {
    if hlight {
        win_state.set_font_color(COLOR_HLITE[if items[which].active {1} else {0}], BKGD_COLOR)
    } else {
        win_state.set_font_color(COLOR_NORML[if items[which].active {1} else {0}], BKGD_COLOR)
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
    cp_draw_window(rdr, win_state.window_x-5, win_state.print_y-5, mw+10, h+10, TEXT_COLOR);
    draw_outline(rdr, win_state.window_x-5, win_state.print_y-5, mw+10, h+10, 0, HIGHLIGHT);
    print(rdr, win_state, str);
    rdr.set_buffer_offset(prev_buffer);
}


fn cp_draw_window(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color: u8) {
    rdr.bar(x, y, width, height, color);
    draw_outline(rdr, x, y, width, height, BORDER2_COLOR, DEACTIVE);
}

fn draw_outline(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color1: u8, color2: u8) {
    vw_hlin(rdr, x, x+width, y, color2);
    vw_vlin(rdr, y, y+height, x, color2);
    vw_hlin(rdr, x,x+width,y+height, color1);
    vw_vlin(rdr, y, y+height, x+width, color1);
}

pub fn check_for_episodes(menu_state: &mut MenuState) {
    //TODO Actually check what data versions there are and enable the menues based on this
    menu_state.update_menu(Menu::MainMenu(MainMenuItem::NewGame), |entry| {
        for i in 0..entry.items.len() {
            if i%2==0 {
                entry.items[i].active = true
            }
        }
    })
}