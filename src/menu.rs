use std::time::Duration;
use std::{ascii, collections::HashMap, str};
use tokio::time::sleep;
use vga::input::NumCode;

use crate::assets::{GraphicNum, Music, SoundName, WolfVariant, is_sod};
use crate::config::{WolfConfig, write_wolf_config};
use crate::def::{Assets, Difficulty, GameState, IWConfig, LevelState, WindowState};
use crate::draw::{RayCast, init_ray_cast};
use crate::input::{ControlDirection, ControlInfo, Input, read_control};
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::play::ProjectionConfig;
use crate::sd::{DigiMode, MusicMode, Sound, SoundMode};
use crate::start::{load_the_game, new_view_size, quit, save_the_game, show_view_size};
use crate::time::Ticker;
use crate::us1::{c_print, line_input, print};
use crate::user::rnd_t;
use crate::vga_render::VGARenderer;
use crate::vh::{vw_hlin, vw_vlin};

const NUM_SAVE_GAMES: usize = 10;

const MAIN_COLOR: u8 = 0x6c;
const EMS_COLOR: u8 = MAIN_COLOR;
const XMS_COLOR: u8 = MAIN_COLOR;
const FILL_COLOR: u8 = 14;

const STRIPE: u8 = 0x2c;
pub const BORDER_COLOR: u8 = 0x29;
const BORDER2_COLOR: u8 = 0x23;
const DEACTIVE: u8 = 0x2b;
const BKGD_COLOR: u8 = 0x2d;

pub const READ_COLOR: u8 = 0x4a;
pub const READ_HCOLOR: u8 = 0x47;
const VIEW_COLOR: u8 = 0x7f;
const TEXT_COLOR: u8 = 0x17;
const HIGHLIGHT: u8 = 0x13;

pub const MENU_X: usize = 76;
pub const MENU_Y: usize = 55;
const MENU_W: usize = 178;
const MENU_H: usize = 13 * 10 + 6;

pub const LSA_X: usize = 96;
pub const LSA_Y: usize = 80;
const LSA_W: usize = 130;
const LSA_H: usize = 42;

const LSM_X: usize = 85;
const LSM_Y: usize = 55;
const LSM_W: usize = 175;
const LSM_H: usize = 10 * 13 + 10;

const SM_X: usize = 48;
const SM_W: usize = 250;
const SM_Y1: usize = 20;
const SM_H1: usize = 4 * 13 - 7;
const SM_Y2: usize = SM_Y1 + 5 * 13;
const SM_H2: usize = 4 * 13 - 7;
const SM_Y3: usize = SM_Y2 + 5 * 13;
const SM_H3: usize = 3 * 13 - 7;

const NM_X: usize = 50;
const NM_Y: usize = 100;
const NM_W: usize = 225;
const NM_H: usize = 13 * 4 + 15;

const NE_X: usize = 10;
const NE_Y: usize = 23;
const NE_W: usize = 320 - NE_X * 2;
const NE_H: usize = 200 - NE_Y * 2;

static END_STRINGS: [&'static str; 9] = [
    "Dost thou wish to\nleave with such hasty\nabandon?",
    "Chickening out...\nalready?",
    "Press N for more carnage.\nPress Y to be a weenie.",
    "So, you think you can\nquit this easily, huh?",
    "Press N to save the world.\nPress Y to abandon it in\nits hour of need.",
    "Press N if you are brave.\nPress Y to cower in shame.",
    "Heroes, press N.\nWimps, press Y.",
    "You are at an intersection.\nA sign says, 'Press Y to quit.'\n>",
    "For guns and glory, press N.\nFor work and worry, press Y.",
];
static GAME_SAVED: &'static str =
    "There's already a game\nsaved at this position.\n      Overwrite?";

static BACK_TO_DEMO: &'static str = "Back to Demo";
static BACK_TO_GAME: &'static str = "Back to Game";

static STR_EMPTY: &'static str = "      - empty -";
static STR_LOADING: &'static str = "Loading...";
static STR_SAVING: &'static str = "Saving...";

static COLOR_HLITE: [u8; 4] = [DEACTIVE, HIGHLIGHT, READ_HCOLOR, 0x67];

static COLOR_NORML: [u8; 4] = [DEACTIVE, TEXT_COLOR, READ_COLOR, 0x6b];

pub struct ItemInfo {
    pub x: usize,
    pub y: usize,
    pub cur_pos: usize,
    pub indent: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ItemActivity {
    Deactive,
    Active,
    Highlight,
}

pub struct ItemType {
    pub active: ItemActivity,
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
    BackTo = 7,
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

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
enum SoundItem {
    SoundEffectNone = 0,
    SoundEffectPCSpeaker = 1,
    SoundEffectAdLib = 2,
    DigitizedNone = 5,
    DigitizedSoundSource = 6,
    DigitizedSoundBlaster = 7,
    MusicNone = 10,
    MusicAdLib = 11,
}

impl SoundItem {
    pub fn pos(self) -> usize {
        self as usize
    }
}

#[derive(PartialEq, Debug)]
pub enum MenuHandle {
    None,
    OpenMenu(Menu),
    Selected(usize),
    QuitMenu,
    BackToGameLoop(Option<usize>),
}

pub struct GameStateUpdate {
    pub load: Option<usize>,
    pub projection_config: ProjectionConfig,
    pub ray_cast: RayCast,
}

impl GameStateUpdate {
    pub fn with_render_update(prj: ProjectionConfig, rc: RayCast) -> GameStateUpdate {
        GameStateUpdate {
            load: None,
            projection_config: prj,
            ray_cast: rc,
        }
    }

    pub fn with_load(prj: ProjectionConfig, rc: RayCast, load: Option<usize>) -> GameStateUpdate {
        GameStateUpdate {
            load,
            projection_config: prj,
            ray_cast: rc,
        }
    }
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
    where
        F: FnOnce(&mut MenuStateEntry),
    {
        self.update_menu(self.selected, f)
    }

    pub fn update_menu<F>(&mut self, menue: Menu, f: F)
    where
        F: FnOnce(&mut MenuStateEntry),
    {
        let entry_opt = self.menues.get_mut(&menue);
        if let Some(state) = entry_opt {
            f(state)
        }
    }
}

type MenuRoutine = fn(&VGARenderer, usize);

fn no_op_routine(_rdr: &VGARenderer, _which: usize) {}

// MainItems
fn initial_main_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: MainMenuItem::NewGame.pos(),
                active: ItemActivity::Active,
                string: "New Game",
            },
            ItemType {
                item: MainMenuItem::Sound.pos(),
                active: ItemActivity::Active,
                string: "Sound",
            },
            ItemType {
                item: MainMenuItem::Control.pos(),
                active: ItemActivity::Active,
                string: "Control",
            },
            ItemType {
                item: MainMenuItem::LoadGame.pos(),
                active: ItemActivity::Active,
                string: "Load Game",
            },
            ItemType {
                item: MainMenuItem::SaveGame.pos(),
                active: ItemActivity::Deactive,
                string: "Save Game",
            },
            ItemType {
                item: MainMenuItem::ChangeView.pos(),
                active: ItemActivity::Active,
                string: "Change View",
            },
            ItemType {
                item: MainMenuItem::ViewScores.pos(),
                active: ItemActivity::Active,
                string: "View Scores",
            },
            ItemType {
                item: MainMenuItem::BackTo.pos(),
                active: ItemActivity::Active,
                string: BACK_TO_DEMO,
            },
            ItemType {
                item: MainMenuItem::Quit.pos(),
                active: ItemActivity::Active,
                string: "Quit",
            },
        ],
        state: ItemInfo {
            x: MENU_X,
            y: MENU_Y,
            cur_pos: MainMenuItem::NewGame.pos(),
            indent: 24,
        },
    }
}

// NewEitems
fn initial_episode_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: EpisodeItem::Episode1.pos(),
                active: ItemActivity::Active,
                string: "Episode 1\nEscape from Wolfenstein",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode2.pos(),
                active: ItemActivity::Deactive,
                string: "Episode 2\nOperation: Eisenfaust",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode3.pos(),
                active: ItemActivity::Deactive,
                string: "Episode 3\nDie, Fuhrer, Die!",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode4.pos(),
                active: ItemActivity::Deactive,
                string: "Episode 4\nA Dark Secret",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode5.pos(),
                active: ItemActivity::Deactive,
                string: "Episode 5\nTrail of the Madman",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode6.pos(),
                active: ItemActivity::Deactive,
                string: "Episode 6\nConfrontation",
            },
        ],
        state: ItemInfo {
            x: NE_X,
            y: NE_Y,
            cur_pos: EpisodeItem::Episode1.pos(),
            indent: 88,
        },
    }
}

// NewItems
fn initial_difficulty_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: DifficultyItem::Daddy.pos(),
                active: ItemActivity::Active,
                string: "Can I play, Daddy?",
            },
            ItemType {
                item: DifficultyItem::HurtMe.pos(),
                active: ItemActivity::Active,
                string: "Don't hurt me.",
            },
            ItemType {
                item: DifficultyItem::BringEmOn.pos(),
                active: ItemActivity::Active,
                string: "Bring 'em on!",
            },
            ItemType {
                item: DifficultyItem::Death.pos(),
                active: ItemActivity::Active,
                string: "I am Death incarnate!",
            },
        ],
        state: ItemInfo {
            x: NM_X,
            y: NM_Y,
            cur_pos: DifficultyItem::BringEmOn.pos(),
            indent: 24,
        },
    }
}

fn initial_sound_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: 0,
                active: ItemActivity::Active,
                string: "None",
            },
            ItemType {
                item: 1,
                active: ItemActivity::Deactive, // TODO Activate if PC Speaker emulation implemented
                string: "PC Speaker",
            },
            ItemType {
                item: 2,
                active: ItemActivity::Active,
                string: "AdLib/Sound Blaster",
            },
            ItemType {
                item: 3,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 4,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 5,
                active: ItemActivity::Active,
                string: "None",
            },
            ItemType {
                item: 6,
                active: ItemActivity::Deactive, // Implement if Sound Source emulation implemented
                string: "Disney Sound Source",
            },
            ItemType {
                item: 7,
                active: ItemActivity::Active,
                string: "Sound Blaster",
            },
            ItemType {
                item: 8,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 9,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 10,
                active: ItemActivity::Active,
                string: "None",
            },
            ItemType {
                item: 11,
                active: ItemActivity::Active,
                string: "AdLib/Sound Blaster",
            },
        ],
        state: ItemInfo {
            x: SM_X,
            y: SM_Y1,
            cur_pos: SoundItem::SoundEffectNone.pos(),
            indent: 52,
        },
    }
}

fn initial_load_save_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: 0,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 1,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 2,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 3,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 4,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 5,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 6,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 7,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 8,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 9,
                active: ItemActivity::Active,
                string: "",
            },
        ],
        state: ItemInfo {
            x: LSM_X,
            y: LSM_Y,
            cur_pos: 0,
            indent: 24,
        },
    }
}

pub fn initial_menu_state() -> MenuState {
    MenuState {
        selected: Menu::Top,
        menues: HashMap::from([
            (Menu::Top, initial_main_menu()),
            (
                Menu::MainMenu(MainMenuItem::NewGame),
                initial_episode_menu(),
            ),
            (Menu::MainMenu(MainMenuItem::Sound), initial_sound_menu()),
            (
                Menu::MainMenu(MainMenuItem::LoadGame),
                initial_load_save_menu(),
            ),
            (
                Menu::MainMenu(MainMenuItem::SaveGame),
                initial_load_save_menu(),
            ),
            (Menu::DifficultySelect, initial_difficulty_menu()),
        ]),
    }
}

fn placeholder() -> ItemType {
    ItemType {
        item: 0,
        active: ItemActivity::Deactive,
        string: "",
    }
}

/// Wolfenstein Control Panel!  Ta Da!
pub async fn control_panel(
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rc: RayCast,
    rdr: &VGARenderer,
    input: &Input,
    prj: ProjectionConfig,
    assets: &Assets,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
    scan: NumCode,
) -> GameStateUpdate {
    sound.play_music(Music::WONDERIN, assets, loader);
    setup_control_panel(win_state, menu_state);

    let mut prj_return = prj;
    let mut rc_return = rc;

    let f_key_handle = match scan {
        NumCode::F1 => todo!("show helpscreen"),
        NumCode::F2 => Some(
            cp_save_game(
                iw_config,
                ticker,
                level_state,
                game_state,
                rdr,
                sound,
                assets,
                input,
                win_state,
                menu_state,
                loader,
            )
            .await,
        ),
        NumCode::F3 => Some(
            cp_load_game(
                iw_config,
                ticker,
                level_state,
                game_state,
                rdr,
                sound,
                assets,
                input,
                win_state,
                menu_state,
                loader,
            )
            .await,
        ),
        NumCode::F4 => Some(
            cp_sound(
                ticker, rdr, sound, assets, input, win_state, menu_state, loader,
            )
            .await,
        ),
        NumCode::F5 => {
            let (handle, prj, rc) = cp_change_view(
                wolf_config,
                iw_config,
                ticker,
                rdr,
                sound,
                rc_return,
                assets,
                input,
                win_state,
                prj_return,
                loader,
            )
            .await;

            prj_return = prj;
            rc_return = rc;
            Some(handle)
        }
        NumCode::F6 => {
            todo!("show cp_control_panel")
        }
        _ => None,
    };
    if let Some(handle) = f_key_handle {
        match handle {
            MenuHandle::QuitMenu | MenuHandle::OpenMenu(_) => {
                // overrule any OpenMenu from the quick keys and return alway to the game
                rdr.fade_out().await;
                return GameStateUpdate::with_load(prj_return, rc_return, None);
            }
            MenuHandle::BackToGameLoop(load) => {
                rdr.fade_out().await;
                return GameStateUpdate::with_load(prj_return, rc_return, load);
            }
            _ => { /* ignore */ }
        }
    }

    let mut menu_stack: Vec<Menu> = Vec::new();
    menu_stack.push(menu_state.selected);

    // MAIN MENU LOOP
    loop {
        let menu_opt = menu_stack.last();
        if let Some(menu) = menu_opt {
            let handle = match menu {
                Menu::Top => {
                    cp_main_menu(ticker, rdr, sound, assets, input, win_state, menu_state).await
                }
                Menu::MainMenu(item) => match item {
                    MainMenuItem::NewGame => {
                        cp_new_game(
                            ticker, game_state, rdr, sound, assets, input, win_state, menu_state,
                        )
                        .await
                    }
                    MainMenuItem::Sound => {
                        cp_sound(
                            ticker, rdr, sound, assets, input, win_state, menu_state, loader,
                        )
                        .await
                    }
                    MainMenuItem::LoadGame => {
                        cp_load_game(
                            iw_config,
                            ticker,
                            level_state,
                            game_state,
                            rdr,
                            sound,
                            assets,
                            input,
                            win_state,
                            menu_state,
                            loader,
                        )
                        .await
                    }
                    MainMenuItem::SaveGame => {
                        cp_save_game(
                            iw_config,
                            ticker,
                            level_state,
                            game_state,
                            rdr,
                            sound,
                            assets,
                            input,
                            win_state,
                            menu_state,
                            loader,
                        )
                        .await
                    }
                    MainMenuItem::ChangeView => {
                        let (handle, prj_new, rc_new) = cp_change_view(
                            wolf_config,
                            iw_config,
                            ticker,
                            rdr,
                            sound,
                            rc_return,
                            assets,
                            input,
                            win_state,
                            prj_return,
                            loader,
                        )
                        .await;
                        prj_return = prj_new;
                        rc_return = rc_new;
                        handle
                    }
                    MainMenuItem::ViewScores => {
                        cp_view_scores(wolf_config, rdr, sound, assets, input, win_state, loader)
                            .await
                    }
                    _ => todo!("implement other menu selects"),
                },
                Menu::DifficultySelect => {
                    cp_difficulty_select(
                        ticker, game_state, rdr, sound, assets, input, win_state, menu_state,
                    )
                    .await
                }
            };
            match handle {
                MenuHandle::OpenMenu(menu) => menu_stack.push(menu),
                MenuHandle::QuitMenu => {
                    menu_stack.pop();
                }
                MenuHandle::BackToGameLoop(load) => {
                    rdr.fade_out().await;
                    return GameStateUpdate::with_load(prj_return, rc_return, load);
                }
                _ => { /* ignore */ }
            }
        } else {
            return GameStateUpdate::with_render_update(prj_return, rc_return); // back to game loop
        }
    }
}

async fn cp_main_menu(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) -> MenuHandle {
    draw_main_menu(rdr, win_state, menu_state);
    rdr.fade_in().await;

    let handle = handle_menu(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        no_op_routine,
    )
    .await;
    if handle == MenuHandle::Selected(MainMenuItem::NewGame.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::NewGame));
    } else if handle == MenuHandle::Selected(MainMenuItem::Sound.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::Sound));
    } else if handle == MenuHandle::Selected(MainMenuItem::LoadGame.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::LoadGame));
    } else if handle == MenuHandle::Selected(MainMenuItem::SaveGame.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::SaveGame));
    } else if handle == MenuHandle::Selected(MainMenuItem::ChangeView.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::ChangeView));
    } else if handle == MenuHandle::Selected(MainMenuItem::ViewScores.pos()) {
        return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::ViewScores));
    } else if handle == MenuHandle::Selected(MainMenuItem::BackTo.pos()) {
        return MenuHandle::BackToGameLoop(None);
    } else if handle == MenuHandle::QuitMenu
        || handle == MenuHandle::Selected(MainMenuItem::Quit.pos())
    {
        menu_quit(ticker, rdr, sound, assets, input, win_state, menu_state).await;
        return MenuHandle::QuitMenu;
    } else {
        return handle;
    }
}

async fn cp_new_game(
    ticker: &Ticker,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) -> MenuHandle {
    loop {
        draw_new_episode(rdr, win_state, menu_state).await;

        let episode_handle = handle_menu(
            ticker,
            rdr,
            sound,
            assets,
            input,
            win_state,
            menu_state,
            no_op_routine,
        )
        .await;
        if let MenuHandle::Selected(episode_selected) = episode_handle {
            sound.play_sound(SoundName::SHOOT, assets);
            //TODO confirm dialog if already in a game
            game_state.episode = episode_selected / 2;
            return MenuHandle::OpenMenu(Menu::DifficultySelect);
        } else {
            rdr.fade_out().await;
            return episode_handle;
        }

        // TODO Shareware support + hint messge how to order the game
    }
}

// Sound

async fn cp_sound(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) -> MenuHandle {
    draw_sound_menu(rdr, win_state, menu_state, sound).await;
    loop {
        let sound_handle = handle_menu(
            ticker,
            rdr,
            sound,
            assets,
            input,
            win_state,
            menu_state,
            no_op_routine,
        )
        .await;
        if let MenuHandle::Selected(which) = sound_handle {
            // SOUND EFFECTS
            if which == SoundItem::SoundEffectNone.pos() {
                sound.set_sound_mode(SoundMode::Off);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
            }
            if which == SoundItem::SoundEffectPCSpeaker.pos() {
                sound.set_sound_mode(SoundMode::PC);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
                sound.play_sound(SoundName::SHOOT, assets);
            }
            if which == SoundItem::SoundEffectAdLib.pos() {
                sound.set_sound_mode(SoundMode::AdLib);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
                sound.play_sound(SoundName::SHOOT, assets);
            }
            // DIGITIZED SOUND
            if which == SoundItem::DigitizedNone.pos() {
                sound.set_digi_mode(DigiMode::Off);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
            }
            if which == SoundItem::DigitizedSoundSource.pos() {
                sound.set_digi_mode(DigiMode::SoundSource);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
                sound.play_sound(SoundName::SHOOT, assets);
            }
            if which == SoundItem::DigitizedSoundBlaster.pos() {
                sound.set_digi_mode(DigiMode::SoundBlaster);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
                sound.play_sound(SoundName::SHOOT, assets);
            }
            // MUSIC
            if which == SoundItem::MusicNone.pos() {
                sound.set_music_mode(MusicMode::Off);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
            }
            if which == SoundItem::MusicAdLib.pos() {
                let changed = sound.music_mode() != MusicMode::AdLib;
                sound.set_music_mode(MusicMode::AdLib);
                draw_sound_menu(rdr, win_state, menu_state, sound).await;
                sound.play_sound(SoundName::SHOOT, assets);
                if changed {
                    sound.play_music(Music::WONDERIN, assets, loader);
                }
            }
        } else {
            // ESC pressed
            rdr.fade_out().await;
            return sound_handle;
        }
    }
}

async fn draw_sound_menu(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    sound: &Sound,
) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, SM_X - 8, SM_Y1 - 3, SM_W, SM_H1, BKGD_COLOR);
    cp_draw_window(rdr, SM_X - 8, SM_Y2 - 3, SM_W, SM_H2, BKGD_COLOR);
    cp_draw_window(rdr, SM_X - 8, SM_Y3 - 3, SM_W, SM_H3, BKGD_COLOR);

    // TODO Active/Inactive state

    menu_state.select_menu(Menu::MainMenu(MainMenuItem::Sound));
    draw_menu(rdr, win_state, menu_state);

    rdr.pic(100, SM_Y1 - 20, GraphicNum::CFXTITLEPIC);
    rdr.pic(100, SM_Y2 - 20, GraphicNum::CDIGITITLEPIC);
    rdr.pic(100, SM_Y3 - 20, GraphicNum::CMUSICTITLEPIC);

    for item in &menu_state.selected_state().items {
        if item.string == "" {
            continue;
        }

        let mut on = false;
        if item.item == SoundItem::SoundEffectNone.pos() {
            on = sound.sound_mode() == SoundMode::Off;
        }
        if item.item == SoundItem::SoundEffectPCSpeaker.pos() {
            on = sound.sound_mode() == SoundMode::PC;
        }
        if item.item == SoundItem::SoundEffectAdLib.pos() {
            on = sound.sound_mode() == SoundMode::AdLib;
        }

        if item.item == SoundItem::DigitizedNone.pos() {
            on = sound.digi_mode() == DigiMode::Off;
        }
        if item.item == SoundItem::DigitizedSoundSource.pos() {
            on = sound.digi_mode() == DigiMode::SoundSource;
        }
        if item.item == SoundItem::DigitizedSoundBlaster.pos() {
            on = sound.digi_mode() == DigiMode::SoundBlaster;
        }

        if item.item == SoundItem::MusicNone.pos() {
            on = sound.music_mode() == MusicMode::Off;
        }
        if item.item == SoundItem::MusicAdLib.pos() {
            on = sound.music_mode() == MusicMode::AdLib;
        }

        if on {
            rdr.pic(
                SM_X + 24,
                SM_Y1 + item.item * 13 + 2,
                GraphicNum::CSELECTEDPIC,
            );
        } else {
            rdr.pic(
                SM_X + 24,
                SM_Y1 + item.item * 13 + 2,
                GraphicNum::CNOTSELECTEDPIC,
            );
        }
    }

    rdr.fade_in().await;
}
// Load & Save

async fn cp_load_game(
    iw_config: &IWConfig,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) -> MenuHandle {
    // TODO Handle QuickLoad
    let state = read_save_game_state(loader);
    draw_load_save_screen(rdr, win_state, menu_state, &state, false).await;
    loop {
        let load_handle = handle_menu(
            ticker,
            rdr,
            sound,
            assets,
            input,
            win_state,
            menu_state,
            no_op_routine,
        )
        .await;
        if let MenuHandle::Selected(which) = load_handle {
            if state[which].available {
                draw_ls_action(rdr, win_state, false);
                game_state.loaded_game = true;
                load_the_game(
                    iw_config,
                    level_state,
                    game_state,
                    win_state,
                    rdr,
                    input,
                    assets,
                    loader,
                    which,
                    LSA_X + 8,
                    LSA_Y + 5,
                )
                .await;
                sound.play_sound(SoundName::SHOOT, assets);
                return MenuHandle::BackToGameLoop(Some(which));
            } // else: loop back to handle_menu
        } else {
            // ESC pressed
            rdr.fade_out().await;
            return load_handle;
        }
    }
}

async fn cp_view_scores(
    wolf_config: &WolfConfig,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    loader: &dyn Loader,
) -> MenuHandle {
    win_state.font_number = 0;
    sound.play_music(Music::ROSTER, assets, loader);
    draw_high_scores(rdr, win_state, &wolf_config.high_scores);
    rdr.fade_in().await;
    win_state.font_number = 1;

    input.ack().await;

    sound.play_music(Music::WONDERIN, assets, loader);
    rdr.fade_out().await;

    MenuHandle::BackToGameLoop(None)
}

async fn cp_save_game(
    iw_config: &IWConfig,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) -> MenuHandle {
    // TODO Handle QuickSave
    let state = read_save_game_state(loader);
    draw_load_save_screen(rdr, win_state, menu_state, &state, true).await;
    loop {
        let save_handle = handle_menu(
            ticker,
            rdr,
            sound,
            assets,
            input,
            win_state,
            menu_state,
            no_op_routine,
        )
        .await;
        if let MenuHandle::Selected(which) = save_handle {
            // TODO Check overwrite existing game

            if state[which].available {
                if !confirm(ticker, rdr, sound, assets, input, win_state, GAME_SAVED).await {
                    draw_load_save_screen(rdr, win_state, menu_state, &state, true).await;
                    continue;
                } else {
                    draw_load_save_screen(rdr, win_state, menu_state, &state, true).await;
                    print_ls_entry(rdr, win_state, menu_state, &state[which], which, HIGHLIGHT);
                }
            }

            sound.play_sound(SoundName::SHOOT, assets);

            let save_menu = &menu_state.menues[&Menu::MainMenu(MainMenuItem::SaveGame)];
            win_state.font_number = 0;
            let initial_input = if state[which].available {
                state[which].name.as_ref().expect("save game name")
            } else {
                // new save game
                // clear save slot text
                rdr.bar(
                    LSM_X + save_menu.state.indent + 1,
                    LSM_Y + which * 13 + 1,
                    LSM_W - save_menu.state.indent - 16,
                    10,
                    BKGD_COLOR,
                );
                ""
            };

            let (input, escape) = line_input(
                ticker,
                rdr,
                input,
                win_state,
                LSM_X + save_menu.state.indent + 2,
                LSM_Y + which * 13 + 1,
                true,
                31,
                LSM_W - save_menu.state.indent - 30,
                initial_input,
            );
            if !escape {
                draw_ls_action(rdr, win_state, true);
                save_the_game(
                    iw_config,
                    level_state,
                    game_state,
                    rdr,
                    loader,
                    which,
                    &input,
                    LSA_X + 8,
                    LSA_Y + 5,
                );
                win_state.font_number = 1;
                return MenuHandle::BackToGameLoop(None);
            } else {
                //TODO repaint entry
                //TODO SD_PlaySound(ESCPRESSEDSND)
                continue;
            }
        } else {
            rdr.fade_out().await;
            return save_handle;
        }
    }
}

struct SaveGameState {
    available: bool,
    name: Option<String>,
}

fn read_save_game_state(loader: &dyn Loader) -> Vec<SaveGameState> {
    let mut result = Vec::with_capacity(NUM_SAVE_GAMES);

    for which in 0..NUM_SAVE_GAMES {
        let header_result = loader.load_save_game_head(which);
        if let Ok(header_bytes) = header_result {
            let mut end = header_bytes.len();
            for i in 0..header_bytes.len() {
                if header_bytes[i] == 0 {
                    end = i;
                    break;
                }
            }
            let name = str::from_utf8(&header_bytes[0..end]).expect("savegame file name");
            result.push(SaveGameState {
                available: true,
                name: Some(name.to_owned()),
            });
        } else {
            result.push(SaveGameState {
                available: false,
                name: None,
            });
        }
    }
    return result;
}

fn draw_ls_action(rdr: &VGARenderer, win_state: &mut WindowState, save: bool) {
    cp_draw_window(rdr, LSA_X, LSA_Y, LSA_W, LSA_H, TEXT_COLOR);
    draw_outline(rdr, LSA_X, LSA_Y, LSA_W, LSA_H, 0, HIGHLIGHT);
    rdr.pic(LSA_X + 8, LSA_Y + 5, GraphicNum::CDISKLOADING1PIC);

    win_state.font_number = 1;
    win_state.set_font_color(0, TEXT_COLOR);
    win_state.print_x = LSA_X + 46;
    win_state.print_y = LSA_Y + 13;

    if save {
        print(rdr, win_state, &STR_SAVING);
    } else {
        print(rdr, win_state, &STR_LOADING);
    }
}

async fn draw_load_save_screen(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    state: &Vec<SaveGameState>,
    save: bool,
) {
    clear_ms_screen(rdr);

    win_state.font_number = 1;
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);
    cp_draw_window(rdr, LSM_X - 10, LSM_Y - 5, LSM_W, LSM_H, BKGD_COLOR);
    draw_stripes(rdr, 10);

    if save {
        rdr.pic(60, 0, GraphicNum::CSAVEGAMEPIC);
        menu_state.select_menu(Menu::MainMenu(MainMenuItem::SaveGame));
    } else {
        rdr.pic(60, 0, GraphicNum::CLOADGAMEPIC);
        menu_state.select_menu(Menu::MainMenu(MainMenuItem::LoadGame));
    }

    let mut i = 0;
    for save_game in state {
        print_ls_entry(rdr, win_state, menu_state, save_game, i, TEXT_COLOR);
        i += 1;
    }

    rdr.fade_in().await;
}

fn print_ls_entry(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    save_game: &SaveGameState,
    w: usize,
    color: u8,
) {
    let ls_entry = &menu_state.menues[&Menu::MainMenu(MainMenuItem::LoadGame)];

    win_state.set_font_color(color, BKGD_COLOR);
    draw_outline(
        rdr,
        LSM_X + ls_entry.state.indent,
        LSM_Y + w * 13,
        LSM_W - ls_entry.state.indent - 15,
        11,
        color,
        color,
    );

    win_state.print_x = LSM_X + ls_entry.state.indent + 2;
    win_state.print_y = LSM_Y + w * 13 + 1;
    win_state.font_number = 0;

    if save_game.available && save_game.name.is_some() {
        print(rdr, win_state, save_game.name.as_ref().unwrap());
    } else {
        print(rdr, win_state, &STR_EMPTY);
    }

    win_state.font_number = 1;
}

fn draw_new_game_diff(rdr: &VGARenderer, which: usize) {
    rdr.pic(NM_X + 185, NM_Y + 7, difficulty_pic(which));
}

// Diffculty Select

async fn cp_difficulty_select(
    ticker: &Ticker,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) -> MenuHandle {
    draw_difficulty_select(rdr, win_state, menu_state).await;
    let handle = handle_menu(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        draw_new_game_diff,
    )
    .await;

    if let MenuHandle::Selected(diff_selected) = handle {
        sound.play_sound(SoundName::SHOOT, assets);
        game_state.prepare_episode_select();
        game_state.difficulty = Difficulty::from_pos(diff_selected);

        rdr.fade_out().await;
        return MenuHandle::BackToGameLoop(None);
    }
    rdr.fade_out().await;
    handle
}

async fn draw_difficulty_select(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, NM_X - 5, NM_Y - 10, NM_W, NM_H, BKGD_COLOR);
    win_state.set_font_color(READ_HCOLOR, BKGD_COLOR);
    win_state.print_x = NM_X + 20;
    win_state.print_y = NM_Y - 32;
    print(rdr, win_state, "How tough are you?");

    menu_state.select_menu(Menu::DifficultySelect);
    draw_menu(rdr, win_state, menu_state);

    menu_state.selected_state().state.cur_pos;
    rdr.pic(
        NM_X + 185,
        NM_Y + 7,
        difficulty_pic(menu_state.selected_state().state.cur_pos),
    );
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

async fn draw_new_episode(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, NE_X - 4, NE_Y - 4, NE_W + 8, NE_H + 8, BKGD_COLOR);
    win_state.set_font_color(READ_HCOLOR, BKGD_COLOR);
    win_state.print_y = 2;
    win_state.window_x = 0;
    c_print(rdr, win_state, "Which episode to play?");

    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    menu_state.select_menu(Menu::MainMenu(MainMenuItem::NewGame));
    draw_menu(rdr, win_state, menu_state);

    for i in 0..6 {
        rdr.pic(NE_X + 32, NE_Y + i * 26, episode_pic(i))
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

async fn menu_quit(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    let text = END_STRINGS[((rnd_t() & 0x07) + (rnd_t() & 1)) as usize];
    if confirm(ticker, rdr, sound, assets, input, win_state, text).await {
        //TODO stop music
        rdr.fade_in().await;
        quit(None)
    }

    draw_main_menu(rdr, win_state, menu_state)
}

async fn cp_change_view(
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    rc: RayCast,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    prj: ProjectionConfig,
    loader: &dyn Loader,
) -> (MenuHandle, ProjectionConfig, RayCast) {
    let old_view = (prj.view_width / 16) as u16;
    let mut new_view = old_view;
    draw_change_view(rdr, win_state, new_view).await;

    loop {
        // TODO CheckPause()
        let ci = read_any_control(input);
        match ci.dir {
            ControlDirection::South | ControlDirection::West => {
                new_view -= 1;
                if new_view < 4 {
                    new_view = 4;
                }
                show_view_size(rdr, new_view);
                sound.play_sound(SoundName::HITWALL, assets);
                tic_delay(ticker, input, 10).await;
            }
            ControlDirection::North | ControlDirection::East => {
                new_view += 1;
                if new_view > 19 {
                    new_view = 19;
                }
                show_view_size(rdr, new_view);
                sound.play_sound(SoundName::HITWALL, assets);
                tic_delay(ticker, input, 10).await;
            }
            _ => { /* ignore */ }
        }

        // TODO PicturePause

        // TODO Check mouse button
        if input.key_pressed(NumCode::Return) {
            break;
        } else if input.key_pressed(NumCode::Escape) {
            sound.play_sound(SoundName::ESCPRESSED, assets);
            rdr.fade_out().await;
            return (MenuHandle::OpenMenu(Menu::Top), prj, rc);
        }
    }

    let mut prj_return = prj;
    let mut rc_return = rc;
    if old_view != new_view {
        sound.play_sound(SoundName::SHOOT, assets);
        message(rdr, win_state, "Thinking...");
        if !iw_config.options.fast_loading {
            sleep(Duration::from_millis(2500)).await;
        }
        prj_return = new_view_size(new_view);
        rc_return = init_ray_cast(prj_return.view_width);
        wolf_config.viewsize = new_view;
        write_wolf_config(loader, wolf_config).expect("write config");
    }

    sound.play_sound(SoundName::SHOOT, assets);
    rdr.fade_out().await;
    return (MenuHandle::OpenMenu(Menu::Top), prj_return, rc_return);
}

async fn draw_change_view(rdr: &VGARenderer, win_state: &mut WindowState, view_size: u16) {
    rdr.bar(0, 160, 320, 40, VIEW_COLOR);
    show_view_size(rdr, view_size);

    win_state.print_y = 161;
    win_state.window_x = 0;
    win_state.window_y = 320;
    win_state.set_font_color(HIGHLIGHT, BKGD_COLOR);

    c_print(rdr, win_state, "Use arrows to size\n");
    c_print(rdr, win_state, "ENTER to accept\n");
    c_print(rdr, win_state, "ESC to cancel");

    rdr.fade_in().await;
}

// helper

async fn confirm(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    str: &str,
) -> bool {
    message(rdr, win_state, str);
    input.clear_keys_down();

    // BLINK CURSOR
    let x = win_state.print_x;
    let y = win_state.print_y;
    let mut tick = false;
    let mut time_count = 0;
    while !input.key_pressed(NumCode::Y)
        && !input.key_pressed(NumCode::N)
        && !input.key_pressed(NumCode::Escape)
    {
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
        true
    } else {
        false
    };

    input.clear_keys_down();

    if exit {
        sound.play_sound(SoundName::ESCPRESSED, assets);
    } else {
        sound.play_sound(SoundName::SHOOT, assets);
    }
    exit
}

/// Handle moving gun around a menu
async fn handle_menu(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    routine: MenuRoutine,
) -> MenuHandle {
    let handle = handle_menu_loop(
        ticker, rdr, sound, assets, input, win_state, menu_state, routine,
    )
    .await;

    input.clear_keys_down();

    if let MenuHandle::Selected(which_pos) = handle {
        menu_state.update_selected(|selected| selected.state.cur_pos = which_pos);
    }
    handle
}

async fn handle_menu_loop(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    routine: MenuRoutine,
) -> MenuHandle {
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

                if which_pos > 0 && selected.items[which_pos - 1].active != ItemActivity::Deactive {
                    y -= 6;
                    draw_half_step(ticker, rdr, sound, assets, x, y).await;
                }

                loop {
                    if which_pos == 0 {
                        which_pos = selected.items.len() - 1;
                    } else {
                        which_pos -= 1;
                    }

                    if selected.items[which_pos].active != ItemActivity::Deactive {
                        break;
                    }
                }
                y = draw_gun(
                    rdr, sound, assets, win_state, selected, x, y, which_pos, base_y, routine,
                );

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            }
            ControlDirection::South => {
                erase_gun(rdr, win_state, selected, x, y, which_pos);

                if which_pos != selected.items.len() - 1
                    && selected.items[which_pos + 1].active != ItemActivity::Deactive
                {
                    y += 6;
                    draw_half_step(ticker, rdr, sound, assets, x, y).await;
                }

                loop {
                    if which_pos == selected.items.len() - 1 {
                        which_pos = 0;
                    } else {
                        which_pos += 1;
                    }

                    if selected.items[which_pos].active != ItemActivity::Deactive {
                        break;
                    }
                }
                y = draw_gun(
                    rdr, sound, assets, win_state, selected, x, y, which_pos, base_y, routine,
                );

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            }
            _ => { /* ignore */ }
        }

        if input.key_pressed(NumCode::Space) || input.key_pressed(NumCode::Return) {
            sound.play_sound(SoundName::SHOOT, assets);
            exit = MenuHandle::Selected(which_pos);
            break;
        }
        if input.key_pressed(NumCode::Escape) {
            sound.play_sound(SoundName::ESCPRESSED, assets);
            exit = MenuHandle::QuitMenu;
            break;
        }
    }
    return exit;
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

async fn draw_half_step(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    x: usize,
    y: usize,
) {
    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);
    sound.play_sound(SoundName::MOVEGUN1, assets);

    ticker.tics(8).await;
}

fn erase_gun(
    rdr: &VGARenderer,
    win_state: &mut WindowState,
    selected: &MenuStateEntry,
    x: usize,
    y: usize,
    which_pos: usize,
) {
    rdr.bar(x - 1, y, 25, 16, BKGD_COLOR);
    set_text_color(win_state, &selected.items, which_pos, false);

    win_state.print_x = selected.state.x + selected.state.indent;
    win_state.print_y = selected.state.y + which_pos * 13;
    print(rdr, win_state, selected.items[which_pos].string);
}

fn draw_gun(
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    win_state: &mut WindowState,
    selected: &MenuStateEntry,
    x: usize,
    y: usize,
    which_pos: usize,
    base_y: usize,
    routine: MenuRoutine,
) -> usize {
    rdr.bar(x - 1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which_pos * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);
    set_text_color(win_state, &selected.items, which_pos, true);

    win_state.print_x = selected.state.x + selected.state.indent;
    win_state.print_y = selected.state.y + which_pos * 13;
    print(rdr, win_state, selected.items[which_pos].string);

    routine(rdr, which_pos);

    sound.play_sound(SoundName::MOVEGUN2, assets);

    new_y
}

fn read_any_control(input: &Input) -> ControlInfo {
    read_control(input)
    // TODO also read mouse and joystick input
}

fn setup_control_panel(win_state: &mut WindowState, menu_state: &mut MenuState) {
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.font_number = 1;
    win_state.window_h = 200;

    if win_state.in_game {
        menu_state.update_menu(Menu::Top, |entry| {
            entry.items[MainMenuItem::SaveGame.pos()].active = ItemActivity::Active;
        })
    }
}

fn draw_main_menu(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &mut MenuState) {
    clear_ms_screen(rdr);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);
    draw_stripes(rdr, 10);
    rdr.pic(84, 0, GraphicNum::COPTIONSPIC);

    cp_draw_window(rdr, MENU_X - 8, MENU_Y - 3, MENU_W, MENU_H, BKGD_COLOR);

    if win_state.in_game {
        let main_menu_opt = menu_state.menues.get_mut(&Menu::Top);
        if let Some(main_menu) = main_menu_opt {
            let demo_item = &mut main_menu.items[MainMenuItem::BackTo.pos()];
            demo_item.active = ItemActivity::Highlight;
            demo_item.string = BACK_TO_GAME;
        }
    } else {
        let main_menu_opt = menu_state.menues.get_mut(&Menu::Top);
        if let Some(main_menu) = main_menu_opt {
            let demo_item = &mut main_menu.items[MainMenuItem::BackTo.pos()];
            demo_item.active = ItemActivity::Active;
            demo_item.string = BACK_TO_DEMO;
        }
    }

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
        if selected.items[i].active != ItemActivity::Deactive {
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
        win_state.set_font_color(COLOR_HLITE[active_ix(items[which].active)], BKGD_COLOR)
    } else {
        win_state.set_font_color(COLOR_NORML[active_ix(items[which].active)], BKGD_COLOR)
    }
}

fn active_ix(active: ItemActivity) -> usize {
    match active {
        ItemActivity::Deactive => 0,
        ItemActivity::Active => 1,
        ItemActivity::Highlight => 2,
    }
}

pub fn draw_stripes(rdr: &VGARenderer, y: usize) {
    rdr.bar(0, y, 320, 24, 0);
    rdr.hlin(0, 319, y + 22, STRIPE);
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
    let mut w: usize = 0;
    let mut mw: usize = 0;
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

    if w + 10 > mw {
        mw = w + 10;
    }

    win_state.print_y = (win_state.window_h / 2) - h / 2;
    win_state.window_x = 160 - mw / 2;
    win_state.print_x = win_state.window_x;

    let prev_buffer = rdr.buffer_offset();
    rdr.set_buffer_offset(rdr.active_buffer());
    cp_draw_window(
        rdr,
        win_state.window_x - 5,
        win_state.print_y - 5,
        mw + 10,
        h + 10,
        TEXT_COLOR,
    );
    draw_outline(
        rdr,
        win_state.window_x - 5,
        win_state.print_y - 5,
        mw + 10,
        h + 10,
        0,
        HIGHLIGHT,
    );
    print(rdr, win_state, str);
    rdr.set_buffer_offset(prev_buffer);
}

fn cp_draw_window(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color: u8) {
    rdr.bar(x, y, width, height, color);
    draw_outline(rdr, x, y, width, height, BORDER2_COLOR, DEACTIVE);
}

fn draw_outline(
    rdr: &VGARenderer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color1: u8,
    color2: u8,
) {
    vw_hlin(rdr, x, x + width, y, color2);
    vw_vlin(rdr, y, y + height, x, color2);
    vw_hlin(rdr, x, x + width, y + height, color1);
    vw_vlin(rdr, y, y + height, x + width, color1);
}

pub fn check_for_episodes(menu_state: &mut MenuState) {
    //TODO Actually check what data versions there are and enable the menues based on this
    menu_state.update_menu(Menu::MainMenu(MainMenuItem::NewGame), |entry| {
        for i in 0..entry.items.len() {
            if i % 2 == 0 {
                entry.items[i].active = ItemActivity::Active;
            }
        }
    })
}

pub fn intro_song(variant: &WolfVariant) -> Music {
    if is_sod(variant) {
        todo!("select SOD intro song")
    } else {
        Music::NAZINOR
    }
}

////////////////////////////////////////////////////////////////////
//
// HANDLE INTRO SCREEN (SYSTEM CONFIG)
//
////////////////////////////////////////////////////////////////////
pub fn intro_screen(rdr: &VGARenderer) {
    // DRAW MAIN MEMORY
    for i in 0..10 {
        // iw assumes that there is always enough main memory
        rdr.bar(49, 163 - 8 * i, 6, 5, MAIN_COLOR - i as u8);
    }

    // DRAW EMS MEMORY
    for i in 0..10 {
        // iw assumes that there is always enough EMS memory
        rdr.bar(89, 163 - 8 * i, 6, 5, EMS_COLOR - i as u8);
    }

    // DRAW XMS MEMORY
    for i in 0..10 {
        // iw assumes that there is always enough XMS memory
        rdr.bar(129, 163 - 8 * i, 6, 5, XMS_COLOR - i as u8);
    }

    // FILL BOXES
    // assume mouse always present
    rdr.bar(164, 82, 12, 2, FILL_COLOR);

    //joystick never present, as there is no controler support a.t.m
    //rdr.bar(164, 105, 12, 2, FILL_COLOR);

    // Adlib never present, as there is always soundblaster emulation
    //rdr.bar(164, 128, 12, 2, FILL_COLOR);

    // SoundBlaster always present through emulation
    rdr.bar(164, 151, 12, 2, FILL_COLOR);

    // SoundSource never present, as there is no emulation for it yet
    //rdr.bar(164, 174, 12, 2, FILL_COLOR);
}
