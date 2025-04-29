use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use std::{ascii, collections::HashMap, str};
use tokio::time::sleep;
use vga::input::{MouseButton, NumCode};

use crate::assets::{GraphicNum, Music, SoundName, W3D1, W3D3, W3D6, WolfVariant, is_sod};
use crate::config::{WolfConfig, write_wolf_config};
use crate::def::{Assets, Button, Difficulty, GameState, IWConfig, LevelState, WindowState};
use crate::draw::{RayCast, init_ray_cast};
use crate::input::{ControlDirection, ControlInfo, Input, read_control};
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::play::{BUTTON_JOY, ProjectionConfig};
use crate::sd::{DigiMode, MusicMode, Sound, SoundMode};
use crate::start::{load_the_game, new_view_size, quit, save_the_game, show_view_size};
use crate::text::help_screens;
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

const CTL_X: usize = 24;
const CTL_Y: usize = 70;
const CTL_W: usize = 284;
const CTL_H: usize = 13 * 7 - 7;
const CTL_INDENT: usize = 56;

const CST_Y: usize = 48;
const CST_START: usize = 60;
const CST_SPC: usize = 60;

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

static STR_CRUN: &'static str = "Run";
static STR_COPEN: &'static str = "Open";
static STR_CFIRE: &'static str = "Fire";
static STR_CSTRAFE: &'static str = "Strafe\n";

static STR_LEFT: &'static str = "Left";
static STR_RIGHT: &'static str = "Right";
static STR_FRWD: &'static str = "Frwd";
static STR_BKWD: &'static str = "Bkwrd\n";

static COLOR_HLITE: [u8; 4] = [DEACTIVE, HIGHLIGHT, READ_HCOLOR, 0x67];
static COLOR_NORML: [u8; 4] = [DEACTIVE, TEXT_COLOR, READ_COLOR, 0x6b];

static MB_ARRAY: [&'static str; 4] = ["b0", "b1", "b2", "b3"];

#[derive(PartialEq)]
enum InputType {
    Mouse,
    Joystick,
    KeyboardButtons,
    KeyboardMove,
}

#[repr(usize)]
#[derive(Copy, Clone)]
enum ButtonOrder {
    Fire,
    Strafe,
    Run,
    Open,
}

static BUTTON_ORDER: [ButtonOrder; 4] = [
    ButtonOrder::Run,
    ButtonOrder::Open,
    ButtonOrder::Fire,
    ButtonOrder::Strafe,
];

#[repr(usize)]
#[derive(Copy, Clone)]
enum MoveOrder {
    Fwrd,
    Right,
    Bkwd,
    Left,
}

static MOVE_ORDER: [MoveOrder; 4] = [
    MoveOrder::Left,
    MoveOrder::Right,
    MoveOrder::Fwrd,
    MoveOrder::Bkwd,
];

pub struct ItemInfo {
    pub x: usize,
    pub y: usize,
    pub cur_pos: Option<usize>,
    pub indent: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ItemActivity {
    Deactive,
    Active,
    Highlight,
    EpisodeTeaser,
}

pub struct ItemType {
    pub active: ItemActivity,
    pub string: &'static str,
    pub item: usize,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[repr(usize)]
pub enum MainMenuItem {
    NewGame = 0,
    Sound = 1,
    Control = 2,
    LoadGame = 3,
    SaveGame = 4,
    ChangeView = 5,
    ReadThis = 6,
    ViewScores = 7,
    BackTo = 8,
    Quit = 9,
}

impl MainMenuItem {
    pub fn id(self) -> usize {
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

#[derive(Copy, Clone, Debug)]
#[repr(usize)]
enum ControlItem {
    MouseEnabled = 0,
    JoystickEnabled = 1,
    JoystickPort2 = 2,
    GamepadEnabled = 3,
    MouseSensitivity = 4,
    CustomizeControls = 5,
}

impl ControlItem {
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
    //sub-menues
    DifficultySelect,
    CustomizeControls,
}

pub struct MenuStateEntry {
    pub items: Vec<ItemType>,
    pub state: ItemInfo,
}

impl MenuStateEntry {
    pub fn find_item(&mut self, id: usize) -> Option<&mut ItemType> {
        for item in &mut self.items {
            if item.item == id {
                return Some(item);
            }
        }
        None
    }
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

    // reset state to initial state (top menu).
    // Cursor positions are kept.
    pub fn reset(&mut self) {
        self.selected = Menu::Top;
    }
}

type MenuRoutine = fn(&VGARenderer, &Input, &mut WindowState, &mut MenuState, usize);

fn no_op_routine(
    _rdr: &VGARenderer,
    _input: &Input,
    _win_state: &mut WindowState,
    _menu_state: &mut MenuState,
    _which: usize,
) {
}

// MainItems
fn initial_main_menu(variant: &WolfVariant) -> MenuStateEntry {
    let mut items = vec![
        ItemType {
            item: MainMenuItem::NewGame.id(),
            active: ItemActivity::Active,
            string: "New Game",
        },
        ItemType {
            item: MainMenuItem::Sound.id(),
            active: ItemActivity::Active,
            string: "Sound",
        },
        ItemType {
            item: MainMenuItem::Control.id(),
            active: ItemActivity::Active,
            string: "Control",
        },
        ItemType {
            item: MainMenuItem::LoadGame.id(),
            active: ItemActivity::Active,
            string: "Load Game",
        },
        ItemType {
            item: MainMenuItem::SaveGame.id(),
            active: ItemActivity::Deactive,
            string: "Save Game",
        },
        ItemType {
            item: MainMenuItem::ChangeView.id(),
            active: ItemActivity::Active,
            string: "Change View",
        },
    ];

    if variant.id == W3D1.id {
        items.push(ItemType {
            item: MainMenuItem::ReadThis.id(),
            active: ItemActivity::Highlight,
            string: "Read This!",
        });
    }

    items.push(ItemType {
        item: MainMenuItem::ViewScores.id(),
        active: ItemActivity::Active,
        string: "View Scores",
    });
    items.push(ItemType {
        item: MainMenuItem::BackTo.id(),
        active: ItemActivity::Active,
        string: BACK_TO_DEMO,
    });
    items.push(ItemType {
        item: MainMenuItem::Quit.id(),
        active: ItemActivity::Active,
        string: "Quit",
    });

    MenuStateEntry {
        items,
        state: ItemInfo {
            x: MENU_X,
            y: MENU_Y,
            cur_pos: Some(MainMenuItem::NewGame.id()),
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
                active: ItemActivity::EpisodeTeaser,
                string: "Episode 2\nOperation: Eisenfaust",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode3.pos(),
                active: ItemActivity::EpisodeTeaser,
                string: "Episode 3\nDie, Fuhrer, Die!",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode4.pos(),
                active: ItemActivity::EpisodeTeaser,
                string: "Episode 4\nA Dark Secret",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode5.pos(),
                active: ItemActivity::EpisodeTeaser,
                string: "Episode 5\nTrail of the Madman",
            },
            placeholder(),
            ItemType {
                item: EpisodeItem::Episode6.pos(),
                active: ItemActivity::EpisodeTeaser,
                string: "Episode 6\nConfrontation",
            },
        ],
        state: ItemInfo {
            x: NE_X,
            y: NE_Y,
            cur_pos: Some(EpisodeItem::Episode1.pos()),
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
            cur_pos: Some(DifficultyItem::BringEmOn.pos()),
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
            cur_pos: Some(SoundItem::SoundEffectNone.pos()),
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
            cur_pos: Some(0),
            indent: 24,
        },
    }
}

fn initial_ctl_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: 0,
                active: ItemActivity::Active,
                string: "Mouse Enabled",
            },
            ItemType {
                item: 1,
                active: ItemActivity::Active,
                string: "Joystick Enabled",
            },
            ItemType {
                item: 2,
                active: ItemActivity::Active,
                string: "Use joystick port 2",
            },
            ItemType {
                item: 3,
                active: ItemActivity::Active,
                string: "Gravis GamePad Enabled",
            },
            ItemType {
                item: 4,
                active: ItemActivity::Active,
                string: "Mouse Sensitivity",
            },
            ItemType {
                item: 5,
                active: ItemActivity::Active,
                string: "Customize controls",
            },
        ],
        state: ItemInfo {
            x: CTL_X,
            y: CTL_Y,
            cur_pos: None,
            indent: CTL_INDENT,
        },
    }
}

pub fn initial_customize_controls_menu() -> MenuStateEntry {
    MenuStateEntry {
        items: vec![
            ItemType {
                item: 0,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 1,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 2,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 3,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 4,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 5,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 6,
                active: ItemActivity::Active,
                string: "",
            },
            ItemType {
                item: 7,
                active: ItemActivity::Deactive,
                string: "",
            },
            ItemType {
                item: 8,
                active: ItemActivity::Active,
                string: "",
            },
        ],
        state: ItemInfo {
            x: 8,
            y: CST_Y + 13 * 2,
            cur_pos: None,
            indent: 0,
        },
    }
}

pub fn initial_menu_state(variant: &WolfVariant) -> MenuState {
    MenuState {
        selected: Menu::Top,
        menues: HashMap::from([
            (Menu::Top, initial_main_menu(variant)),
            (
                Menu::MainMenu(MainMenuItem::NewGame),
                initial_episode_menu(),
            ),
            (Menu::MainMenu(MainMenuItem::Sound), initial_sound_menu()),
            (
                Menu::MainMenu(MainMenuItem::LoadGame),
                initial_load_save_menu(),
            ),
            (Menu::MainMenu(MainMenuItem::Control), initial_ctl_menu()),
            (Menu::CustomizeControls, initial_customize_controls_menu()),
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
    input: &mut Input,
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
        NumCode::F1 => {
            if loader.variant().help_text_lump_id.is_some() {
                Some(cp_read_this(rdr, sound, assets, input, loader).await)
            } else {
                None
            }
        }
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
            Some(cp_control(ticker, rdr, sound, assets, input, win_state, menu_state).await)
        }
        _ => None,
    };
    if let Some(handle) = f_key_handle {
        match handle {
            MenuHandle::QuitMenu | MenuHandle::OpenMenu(_) => {
                // overrule any OpenMenu from the quick keys and return always to the game
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
                    MainMenuItem::Control => {
                        cp_control(ticker, rdr, sound, assets, input, win_state, menu_state).await
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
                    MainMenuItem::ReadThis => cp_read_this(rdr, sound, assets, input, loader).await,
                    MainMenuItem::ViewScores => {
                        cp_view_scores(wolf_config, rdr, sound, assets, input, win_state, loader)
                            .await
                    }
                    MainMenuItem::Quit => MenuHandle::QuitMenu,
                    MainMenuItem::BackTo => MenuHandle::BackToGameLoop(None),
                },
                Menu::DifficultySelect => {
                    cp_difficulty_select(
                        ticker, game_state, rdr, sound, assets, input, win_state, menu_state,
                    )
                    .await
                }
                Menu::CustomizeControls => {
                    cp_custom_controls(
                        wolf_config,
                        ticker,
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

async fn cp_read_this(
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    loader: &dyn Loader,
) -> MenuHandle {
    sound.play_music(Music::CORNER, assets, loader);
    help_screens(rdr, input).await;
    sound.play_music(Music::WONDERIN, assets, loader);
    MenuHandle::QuitMenu
}

async fn cp_custom_controls(
    wolf_config: &mut WolfConfig,
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) -> MenuHandle {
    draw_custom_screen(rdr, input, win_state, menu_state).await;

    menu_state.select_menu(Menu::CustomizeControls);
    let handle = handle_menu(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        fixup_custom,
    )
    .await;

    if let MenuHandle::Selected(which) = &handle {
        match which {
            0 => {
                define_mouse_btns(ticker, rdr, sound, assets, input, win_state, menu_state);
                draw_cust_mouse(rdr, input, win_state, menu_state, true);
            }
            3 => {
                todo!("implement joystick button change");
            }
            6 => {
                define_key_btns(ticker, rdr, sound, assets, input, win_state, menu_state);
                draw_cust_keybd(rdr, input, win_state, menu_state, false);
            }
            8 => {
                define_key_move(ticker, rdr, sound, assets, input, win_state, menu_state);
                draw_cust_keys(rdr, input, win_state, menu_state, false);
            }
            _ => { /* do nothing */ }
        }

        wolf_config.button_mouse = input.button_mouse.clone();
        wolf_config.button_scan = input.button_scan.clone();
        wolf_config.dir_scan = input.dir_scan.clone();
        write_wolf_config(loader, wolf_config).expect("write config");
    }

    return handle;
}

fn define_mouse_btns(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    enter_ctrl_data(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        2,
        [false, true, true, true],
        draw_cust_mouse,
        print_cust_mouse,
        InputType::Mouse,
    );
}

fn define_key_btns(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    enter_ctrl_data(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        8,
        [true, true, true, true],
        draw_cust_keybd,
        print_cust_keybd,
        InputType::KeyboardButtons,
    );
}

fn define_key_move(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    enter_ctrl_data(
        ticker,
        rdr,
        sound,
        assets,
        input,
        win_state,
        menu_state,
        10,
        [true, true, true, true],
        draw_cust_keys,
        print_cust_keys,
        InputType::KeyboardMove,
    );
}

type DrawRoutine = fn(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    hilight: bool,
);

type PrintRoutine = fn(rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, i: usize);

fn enter_ctrl_data(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &mut Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    index: usize,
    allowed: [bool; 4],
    draw_routine: DrawRoutine,
    print_routine: PrintRoutine,
    input_type: InputType,
) {
    sound.play_sound(SoundName::SHOOT, assets);
    input.clear_keys_down();

    win_state.print_y = CST_Y + 13 * index;
    let mut exit = false;
    let mut redraw = true;

    // FIND FIRST SPOT IN ALLOWED ARRAY
    let mut which = 0;
    for i in 0..4 {
        if allowed[i] {
            which = i;
            break;
        }
    }

    let mut x = CST_START + CST_SPC * which;

    loop {
        if redraw {
            x = CST_START + CST_SPC * which;
            cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
            draw_routine(rdr, input, win_state, menu_state, true);
            cp_draw_window(rdr, x - 2, win_state.print_y, CST_SPC, 11, TEXT_COLOR);
            draw_outline(rdr, x - 2, win_state.print_y, CST_SPC, 11, 0, HIGHLIGHT);
            win_state.set_font_color(0, TEXT_COLOR);
            print_routine(rdr, input, win_state, which);
            win_state.print_x = x;
            wait_key_up(input);
            redraw = false;
        }

        let mut ci = read_any_control(input);

        if input_type == InputType::Mouse || input_type == InputType::Joystick {
            if input.key_pressed(NumCode::Return)
                || input.key_pressed(NumCode::Control)
                || input.key_pressed(NumCode::Alt)
            {
                input.clear_keys_down();
                ci.button_0 = false;
                ci.button_1 = false;
            }
        }

        // CHANGE BUTTON VALUE?
        if (ci.button_0 || ci.button_1 || ci.button_2 || ci.button_3)
            || (input_type == InputType::KeyboardButtons || input_type == InputType::KeyboardMove)
                && input.last_scan() == NumCode::Return
        {
            let mut picked = false;
            ticker.clear_count();
            let mut tick = false;
            win_state.set_font_color(0, TEXT_COLOR);

            loop {
                if input_type == InputType::KeyboardButtons || input_type == InputType::KeyboardMove
                {
                    input.clear_keys_down();
                }

                if ticker.get_count() > 10 {
                    if !tick {
                        rdr.bar(x, win_state.print_y + 1, CST_SPC - 2, 10, TEXT_COLOR);
                    } else {
                        win_state.print_x = x;
                        print(rdr, win_state, "?");
                        sound.play_sound(SoundName::HITWALL, assets);
                    }
                    tick = !tick;
                    ticker.clear_count();
                }

                // WHICH TYPE OF INPUT DO WE PROCESS?
                match input_type {
                    InputType::Mouse => {
                        let mut result = None;
                        if input.mouse_button_pressed(MouseButton::Left) {
                            result = Some(MouseButton::Left)
                        } else if input.mouse_button_pressed(MouseButton::Right) {
                            result = Some(MouseButton::Right)
                        } else if input.mouse_button_pressed(MouseButton::Middle) {
                            result = Some(MouseButton::Middle)
                        }

                        if let Some(button) = result {
                            for z in 0..4 {
                                if BUTTON_ORDER[which] as usize == input.button_mouse[z] as usize {
                                    input.button_mouse[which] = Button::NoButton;
                                    break;
                                }
                            }

                            input.button_mouse[button as usize - 1] =
                                Button::from_usize(BUTTON_ORDER[which] as usize);
                            picked = true;
                            sound.play_sound(SoundName::SHOOTDOOR, assets);
                        }
                    }
                    InputType::Joystick => {
                        todo!("enter joystick");
                    }
                    InputType::KeyboardButtons => {
                        let last_scan = input.last_scan();
                        if last_scan != NumCode::None {
                            input.button_scan[BUTTON_ORDER[which] as usize] = last_scan;
                            picked = true;
                            sound.play_sound(SoundName::SHOOT, assets);
                            input.clear_keys_down();
                        }
                    }
                    InputType::KeyboardMove => {
                        let last_scan = input.last_scan();
                        if last_scan != NumCode::None {
                            input.dir_scan[MOVE_ORDER[which] as usize] = last_scan;
                            picked = true;
                            sound.play_sound(SoundName::SHOOT, assets);
                            input.clear_keys_down();
                        }
                    }
                }

                // EXIT INPUT?
                if input.key_pressed(NumCode::Escape) {
                    picked = true;
                }

                if picked {
                    break;
                }
            }

            win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
            redraw = true;
            wait_key_up(input);
        }

        if ci.button_1 || input.key_pressed(NumCode::Escape) {
            exit = true;
        }

        // MOVE TO ANOTHER SPOT?

        match ci.dir {
            ControlDirection::West => {
                loop {
                    if which == 0 {
                        which = 3;
                    } else {
                        which -= 1;
                    }
                    if allowed[which] {
                        break;
                    }
                }
                redraw = true;
                sound.play_sound(SoundName::MOVEGUN1, assets);
                loop {
                    let ci = read_any_control(input);
                    if ci.dir == ControlDirection::None {
                        break;
                    }
                }
                input.clear_keys_down();
            }
            ControlDirection::East => {
                loop {
                    if which == 3 {
                        which = 0;
                    } else {
                        which += 1;
                    }
                    if allowed[which] {
                        break;
                    }
                }
                redraw = true;
                sound.play_sound(SoundName::MOVEGUN1, assets);
                loop {
                    let ci = read_any_control(input);
                    if ci.dir == ControlDirection::None {
                        break;
                    }
                }
                input.clear_keys_down();
            }
            ControlDirection::North | ControlDirection::South => {
                exit = true;
            }
            _ => { /* ignore */ }
        }

        if exit {
            break;
        }
    }

    sound.play_sound(SoundName::ESCPRESSED, assets);
    wait_key_up(input);
    cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
}

async fn draw_custom_screen(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    clear_ms_screen(rdr);
    win_state.window_x = 0;
    win_state.window_w = 320;

    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);
    draw_stripes(rdr, 10);
    rdr.pic(80, 0, GraphicNum::CCUSTOMIZEPIC);

    // MOUSE
    win_state.set_font_color(READ_COLOR, BKGD_COLOR);
    win_state.window_x = 0;
    win_state.window_w = 320;

    // MOUSE
    win_state.print_y = CST_Y;
    c_print(rdr, win_state, "Mouse\n");
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.print_x = CST_START;
    print(rdr, win_state, STR_CRUN);
    win_state.print_x = CST_START + CST_SPC * 1;
    print(rdr, win_state, STR_COPEN);
    win_state.print_x = CST_START + CST_SPC * 2;
    print(rdr, win_state, STR_CFIRE);
    win_state.print_x = CST_START + CST_SPC * 3;
    print(rdr, win_state, STR_CSTRAFE);
    cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
    draw_cust_mouse(rdr, input, win_state, menu_state, false);
    print(rdr, win_state, "\n");

    // JOYSTICK/PAD
    win_state.set_font_color(READ_COLOR, BKGD_COLOR);
    c_print(rdr, win_state, "Joystick/Gravis GamePad\n");
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.print_x = CST_START;
    print(rdr, win_state, STR_CRUN);
    win_state.print_x = CST_START + CST_SPC * 1;
    print(rdr, win_state, STR_COPEN);
    win_state.print_x = CST_START + CST_SPC * 2;
    print(rdr, win_state, STR_CFIRE);
    win_state.print_x = CST_START + CST_SPC * 3;
    print(rdr, win_state, STR_CSTRAFE);
    cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
    draw_cust_joy(rdr, input, win_state, menu_state, false);
    print(rdr, win_state, "\n");

    // KEYBOARD
    win_state.set_font_color(READ_COLOR, BKGD_COLOR);
    c_print(rdr, win_state, "Keyboard\n");
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.print_x = CST_START;
    print(rdr, win_state, STR_CRUN);
    win_state.print_x = CST_START + CST_SPC * 1;
    print(rdr, win_state, STR_COPEN);
    win_state.print_x = CST_START + CST_SPC * 2;
    print(rdr, win_state, STR_CFIRE);
    win_state.print_x = CST_START + CST_SPC * 3;
    print(rdr, win_state, STR_CSTRAFE);
    cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
    draw_cust_keybd(rdr, input, win_state, menu_state, false);
    print(rdr, win_state, "\n");

    // KEYBOARD MOVE KEYS
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.print_x = CST_START;
    print(rdr, win_state, STR_LEFT);
    win_state.print_x = CST_START + CST_SPC * 1;
    print(rdr, win_state, STR_RIGHT);
    win_state.print_x = CST_START + CST_SPC * 2;
    print(rdr, win_state, STR_FRWD);
    win_state.print_x = CST_START + CST_SPC * 3;
    print(rdr, win_state, STR_BKWD);
    cp_draw_window(rdr, 5, win_state.print_y - 1, 310, 13, BKGD_COLOR);
    draw_cust_keys(rdr, input, win_state, menu_state, false);

    // PICK STARTING POINT IN MENU
    let menu = menu_state
        .menues
        .get_mut(&Menu::CustomizeControls)
        .expect("customize control menue");

    if menu.state.cur_pos.is_none() {
        for i in 0..menu.items.len() {
            if menu.items[i].active == ItemActivity::Active {
                menu.state.cur_pos = Some(i);
                break;
            }
        }
    }

    rdr.fade_in().await;
}

// FIXUP GUN CURSOR OVERDRAW SHIT
fn fixup_custom(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    which: usize,
) {
    static LAST_WHICH: AtomicI32 = AtomicI32::new(-1);

    let y = CST_Y + 26 + which * 13;
    vw_hlin(rdr, 7, 32, y - 1, DEACTIVE);
    vw_hlin(rdr, 7, 32, y + 12, BORDER2_COLOR);
    vw_hlin(rdr, 7, 32, y - 2, BORDER_COLOR);
    vw_hlin(rdr, 7, 32, y + 13, BORDER_COLOR);

    match which {
        0 => draw_cust_mouse(rdr, input, win_state, menu_state, true),
        3 => draw_cust_joy(rdr, input, win_state, menu_state, true),
        6 => draw_cust_keybd(rdr, input, win_state, menu_state, true),
        8 => draw_cust_keys(rdr, input, win_state, menu_state, true),
        _ => {}
    }

    let last_which = LAST_WHICH.load(Ordering::Relaxed);
    if last_which >= 0 {
        let y = CST_Y + 26 + last_which as usize * 13;
        vw_hlin(rdr, 7, 32, y - 1, DEACTIVE);
        vw_hlin(rdr, 7, 32, y + 12, BORDER2_COLOR);
        vw_hlin(rdr, 7, 32, y - 2, BORDER_COLOR);
        vw_hlin(rdr, 7, 32, y + 13, BORDER_COLOR);

        if last_which as usize != which {
            match last_which {
                0 => draw_cust_mouse(rdr, input, win_state, menu_state, false),
                3 => draw_cust_joy(rdr, input, win_state, menu_state, false),
                6 => draw_cust_keybd(rdr, input, win_state, menu_state, false),
                8 => draw_cust_keys(rdr, input, win_state, menu_state, false),
                _ => {}
            }
        }
    }

    LAST_WHICH.store(which as i32, Ordering::Relaxed);
}

fn draw_cust_mouse(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    hilight: bool,
) {
    let color = if hilight { HIGHLIGHT } else { TEXT_COLOR };
    win_state.set_font_color(color, BKGD_COLOR);

    let menu = menu_state
        .menues
        .get_mut(&Menu::CustomizeControls)
        .expect("customize control menue");

    if !input.mouse_enabled {
        win_state.set_font_color(DEACTIVE, BKGD_COLOR);
        menu.items[0].active = ItemActivity::Deactive;
    } else {
        menu.items[0].active = ItemActivity::Active;
    }

    win_state.print_y = CST_Y + 13 * 2;
    for i in 0..4 {
        print_cust_mouse(rdr, input, win_state, i);
    }
}

fn print_cust_mouse(rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, i: usize) {
    for j in 0..4 {
        if BUTTON_ORDER[i] as usize == input.button_mouse[j] as usize {
            win_state.print_x = CST_START + CST_SPC * i;
            print(rdr, win_state, MB_ARRAY[j]);
            break;
        }
    }
}

fn draw_cust_joy(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    hilight: bool,
) {
    let color = if hilight { HIGHLIGHT } else { TEXT_COLOR };
    win_state.set_font_color(color, BKGD_COLOR);

    let menu = menu_state
        .menues
        .get_mut(&Menu::CustomizeControls)
        .expect("customize control menue");

    if !input.joystick_enabled {
        win_state.set_font_color(DEACTIVE, BKGD_COLOR);
        menu.items[3].active = ItemActivity::Deactive;
    } else {
        menu.items[3].active = ItemActivity::Active;
    }

    win_state.print_y = CST_Y + 13 * 5;
    for i in 0..4 {
        print_cust_joy(rdr, win_state, i);
    }
}

fn print_cust_joy(rdr: &VGARenderer, win_state: &mut WindowState, i: usize) {
    for j in 0..4 {
        if BUTTON_ORDER[i] as usize == BUTTON_JOY[j] as usize {
            win_state.print_x = CST_START + CST_SPC * i;
            print(rdr, win_state, MB_ARRAY[j]);
            break;
        }
    }
}

fn draw_cust_keybd(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    _: &mut MenuState,
    hilight: bool,
) {
    let color = if hilight { HIGHLIGHT } else { TEXT_COLOR };
    win_state.set_font_color(color, BKGD_COLOR);

    win_state.print_y = CST_Y + 13 * 8;
    for i in 0..4 {
        print_cust_keybd(rdr, input, win_state, i);
    }
}

fn print_cust_keybd(rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, i: usize) {
    win_state.print_x = CST_START + CST_SPC * i;
    let scan = input.button_scan[BUTTON_ORDER[i] as usize];
    print(rdr, win_state, numcode_name(scan));
}

fn draw_cust_keys(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    _: &mut MenuState,
    hilight: bool,
) {
    let color = if hilight { HIGHLIGHT } else { TEXT_COLOR };
    win_state.set_font_color(color, BKGD_COLOR);

    win_state.print_y = CST_Y + 13 * 10;
    for i in 0..4 {
        print_cust_keys(rdr, input, win_state, i);
    }
}

fn print_cust_keys(rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, i: usize) {
    let scan = input.dir_scan[MOVE_ORDER[i] as usize];
    win_state.print_x = CST_START + CST_SPC * i;
    print(rdr, win_state, numcode_name(scan));
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

    match handle {
        MenuHandle::Selected(selected) => {
            let top_menue = &menu_state.menues[&Menu::Top];
            let selected_item = top_menue.items[selected].item;

            if selected_item == MainMenuItem::NewGame.id() {
                MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::NewGame))
            } else if selected_item == MainMenuItem::Sound.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::Sound));
            } else if selected_item == MainMenuItem::Control.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::Control));
            } else if selected_item == MainMenuItem::LoadGame.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::LoadGame));
            } else if selected_item == MainMenuItem::SaveGame.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::SaveGame));
            } else if selected_item == MainMenuItem::ChangeView.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::ChangeView));
            } else if selected_item == MainMenuItem::ReadThis.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::ReadThis));
            } else if selected_item == MainMenuItem::ViewScores.id() {
                return MenuHandle::OpenMenu(Menu::MainMenu(MainMenuItem::ViewScores));
            } else if selected_item == MainMenuItem::BackTo.id() {
                return MenuHandle::BackToGameLoop(None);
            } else if selected_item == MainMenuItem::Quit.id() {
                menu_quit(ticker, rdr, sound, assets, input, win_state, menu_state).await;
                MenuHandle::QuitMenu
            } else {
                quit(Some("unknown menu selected"));
                handle
            }
        }
        MenuHandle::QuitMenu => {
            menu_quit(ticker, rdr, sound, assets, input, win_state, menu_state).await;
            MenuHandle::QuitMenu
        }
        _ => handle,
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
            let new_game_menu = menu_state
                .menues
                .get(&Menu::MainMenu(MainMenuItem::NewGame))
                .expect("NewGame menu not found");

            if new_game_menu.items[episode_selected].active == ItemActivity::EpisodeTeaser {
                sound.play_sound(SoundName::NOWAY, assets);
                message(
                    rdr,
                    win_state,
                    "Please select \"Read This!\"\nfrom the Options menu to\nfind out how to order this\nepisode from Apogee.",
                );
                input.clear_keys_down();
                input.ack().await;
                continue;
            } else {
                sound.play_sound(SoundName::SHOOT, assets);
                //TODO confirm dialog if already in a game
                game_state.episode = episode_selected / 2;
                return MenuHandle::OpenMenu(Menu::DifficultySelect);
            };
        } else {
            rdr.fade_out().await;
            return episode_handle;
        }
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
                sound.play_sound(SoundName::ESCPRESSED, assets);
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

fn draw_new_game_diff(
    rdr: &VGARenderer,
    _: &Input,
    _: &mut WindowState,
    _: &mut MenuState,
    which: usize,
) {
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

    let pos = menu_state.selected_state().state.cur_pos.unwrap_or(0);
    rdr.pic(NM_X + 185, NM_Y + 7, difficulty_pic(pos));
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

async fn cp_control(
    ticker: &Ticker,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) -> MenuHandle {
    draw_ctl_screen(rdr, input, win_state, menu_state).await;

    wait_key_up(input);

    loop {
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

        if let MenuHandle::Selected(which) = handle {
            if which == ControlItem::MouseEnabled.pos() {
                // TODO handle mouse enable
            }
            if which == ControlItem::JoystickEnabled.pos() {
                // TODO handle joystick enable
            }
            if which == ControlItem::JoystickPort2.pos() {
                // TODO handle joystick enable
            }
            if which == ControlItem::GamepadEnabled.pos() {
                // TODO handle gamepad enable
            }
            if which == ControlItem::MouseSensitivity.pos() {
                // TODO handle mouse sensitiviy
            }
            if which == ControlItem::CustomizeControls.pos() {
                return MenuHandle::OpenMenu(Menu::CustomizeControls);
            }
        } else {
            // ESC pressed
            rdr.fade_out().await;
            return handle;
        }
    }
}

async fn draw_ctl_screen(
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
) {
    clear_ms_screen(rdr);
    draw_stripes(rdr, 10);
    rdr.pic(80, 0, GraphicNum::CCONTROLPIC);
    rdr.pic(112, 184, GraphicNum::CMOUSELBACKPIC);

    cp_draw_window(rdr, CTL_X - 8, CTL_Y - 5, CTL_W, CTL_H, BKGD_COLOR);

    win_state.window_x = 0;
    win_state.window_w = 320;
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);

    menu_state.update_menu(Menu::MainMenu(MainMenuItem::Control), |entry| {
        // no gamepad support at the moment, always disable
        if !input.joystick_enabled {
            entry.items[1].active = ItemActivity::Deactive;
            entry.items[2].active = ItemActivity::Deactive;
            entry.items[3].active = ItemActivity::Deactive;
        }

        // no mouse support at the moment, always disable
        if !input.mouse_enabled {
            entry.items[0].active = ItemActivity::Deactive;
            entry.items[4].active = ItemActivity::Deactive;
        }
    });

    menu_state.select_menu(Menu::MainMenu(MainMenuItem::Control));
    draw_menu(rdr, win_state, menu_state);

    let x = CTL_X + CTL_INDENT - 24;
    let y = CTL_Y + 3;
    if input.mouse_enabled {
        rdr.pic(x, y, GraphicNum::CSELECTEDPIC);
    } else {
        rdr.pic(x, y, GraphicNum::CNOTSELECTEDPIC);
    }

    let y = CTL_Y + 16;
    if input.joystick_enabled {
        rdr.pic(x, y, GraphicNum::CSELECTEDPIC);
    } else {
        rdr.pic(x, y, GraphicNum::CNOTSELECTEDPIC);
    }

    let y = CTL_Y + 29;
    if input.joystick_enabled {
        rdr.pic(x, y, GraphicNum::CSELECTEDPIC);
    } else {
        rdr.pic(x, y, GraphicNum::CNOTSELECTEDPIC);
    }

    let y = CTL_Y + 42;
    if input.joystick_enabled {
        rdr.pic(x, y, GraphicNum::CSELECTEDPIC);
    } else {
        rdr.pic(x, y, GraphicNum::CNOTSELECTEDPIC);
    }

    // PICK FIRST AVAILABLE SPOT
    let menu = menu_state
        .menues
        .get_mut(&Menu::MainMenu(MainMenuItem::Control))
        .expect("control menue");
    if menu.state.cur_pos.is_none()
        || menu.items[menu.state.cur_pos.unwrap()].active != ItemActivity::Active
    {
        for i in 0..6 {
            if menu.items[i].active == ItemActivity::Active {
                menu.state.cur_pos = Some(i);
                break;
            }
        }
    }

    draw_menu_gun(rdr, &menu.state);
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
        menu_state.update_selected(|selected| selected.state.cur_pos = Some(which_pos));
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
    let (mut which_pos, x, base_y) = {
        let selected = menu_state.selected_state();
        let which_pos = selected.state.cur_pos.unwrap_or(0);
        let x = selected.state.x & 8_usize.wrapping_neg();
        let base_y = selected.state.y - 2;
        (which_pos, x, base_y)
    };
    let mut y = base_y + which_pos * 13;
    rdr.pic(x, y, GraphicNum::CCURSOR1PIC);

    // CALL CUSTOM ROUTINE IF IT IS NEEDED
    routine(rdr, input, win_state, menu_state, which_pos);

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
            routine(rdr, input, win_state, menu_state, which_pos);
        }

        // TODO CheckPause

        // TODO check key presses

        let ci = read_any_control(input);
        match ci.dir {
            ControlDirection::North => {
                erase_gun(rdr, win_state, menu_state.selected_state(), x, y, which_pos);

                if which_pos > 0
                    && menu_state.selected_state().items[which_pos - 1].active
                        != ItemActivity::Deactive
                {
                    y -= 6;
                    draw_half_step(ticker, rdr, sound, assets, x, y).await;
                }

                loop {
                    if which_pos == 0 {
                        which_pos = menu_state.selected_state().items.len() - 1;
                    } else {
                        which_pos -= 1;
                    }

                    if menu_state.selected_state().items[which_pos].active != ItemActivity::Deactive
                    {
                        break;
                    }
                }

                y = draw_gun(
                    rdr, sound, assets, input, win_state, menu_state, x, y, which_pos, base_y,
                    routine,
                );

                // WAIT FOR BUTTON-UP OR DELAY NEXT MOVE
                tic_delay(ticker, input, 20).await;
            }
            ControlDirection::South => {
                erase_gun(rdr, win_state, menu_state.selected_state(), x, y, which_pos);

                if which_pos != menu_state.selected_state().items.len() - 1
                    && menu_state.selected_state().items[which_pos + 1].active
                        != ItemActivity::Deactive
                {
                    y += 6;
                    draw_half_step(ticker, rdr, sound, assets, x, y).await;
                }

                loop {
                    if which_pos == menu_state.selected_state().items.len() - 1 {
                        which_pos = 0;
                    } else {
                        which_pos += 1;
                    }

                    if menu_state.selected_state().items[which_pos].active != ItemActivity::Deactive
                    {
                        break;
                    }
                }
                y = draw_gun(
                    rdr, sound, assets, input, win_state, menu_state, x, y, which_pos, base_y,
                    routine,
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

fn draw_menu_gun(rdr: &VGARenderer, item_info: &ItemInfo) {
    let y = item_info.y + item_info.cur_pos.unwrap_or(0) * 13 - 2;
    rdr.pic(item_info.x, y, GraphicNum::CCURSOR1PIC);
}

fn draw_gun(
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    input: &Input,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    x: usize,
    y: usize,
    which_pos: usize,
    base_y: usize,
    routine: MenuRoutine,
) -> usize {
    let selected = menu_state.selected_state();

    rdr.bar(x - 1, y, 25, 16, BKGD_COLOR);
    let new_y = base_y + which_pos * 13;
    rdr.pic(x, new_y, GraphicNum::CCURSOR1PIC);

    set_text_color(win_state, &selected.items, which_pos, true);

    win_state.print_x = selected.state.x + selected.state.indent;
    win_state.print_y = selected.state.y + which_pos * 13;
    print(rdr, win_state, selected.items[which_pos].string);

    routine(rdr, input, win_state, menu_state, which_pos);

    sound.play_sound(SoundName::MOVEGUN2, assets);

    new_y
}

fn read_any_control(input: &Input) -> ControlInfo {
    let mut ci = ControlInfo {
        button_0: false,
        button_1: false,
        button_2: false,
        button_3: false,
        dir: ControlDirection::None,
    };
    read_control(input, &mut ci);

    if input.mouse_enabled {
        ci.button_0 = input.mouse_button_pressed(MouseButton::Left);
        ci.button_1 = input.mouse_button_pressed(MouseButton::Right);
        ci.button_2 = input.mouse_button_pressed(MouseButton::Middle);
        // TODO read mouse direction
    }

    // TODO read joystick input
    ci
}

fn wait_key_up(input: &Input) {
    loop {
        let ci = read_any_control(input);
        let something_pressed = ci.button_0
            | ci.button_1
            | ci.button_2
            | ci.button_3
            | input.key_pressed(NumCode::Space)
            | input.key_pressed(NumCode::Return)
            | input.key_pressed(NumCode::Escape);
        if !something_pressed {
            return;
        }
    }
}

fn setup_control_panel(win_state: &mut WindowState, menu_state: &mut MenuState) {
    win_state.set_font_color(TEXT_COLOR, BKGD_COLOR);
    win_state.font_number = 1;
    win_state.window_h = 200;

    if win_state.in_game {
        menu_state.update_menu(Menu::Top, |entry| {
            entry
                .find_item(MainMenuItem::SaveGame.id())
                .expect("item")
                .active = ItemActivity::Active;
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
            let back_item = main_menu
                .find_item(MainMenuItem::BackTo.id())
                .expect("back_item");
            back_item.active = ItemActivity::Highlight;
            back_item.string = BACK_TO_GAME;
        }
    } else {
        let main_menu_opt = menu_state.menues.get_mut(&Menu::Top);
        if let Some(main_menu) = main_menu_opt {
            let demo_item = main_menu
                .find_item(MainMenuItem::BackTo.id())
                .expect("demo_item");
            demo_item.active = ItemActivity::Active;
            demo_item.string = BACK_TO_DEMO;
        }
    }

    menu_state.select_menu(Menu::Top);
    draw_menu(rdr, win_state, menu_state);
}

fn draw_menu(rdr: &VGARenderer, win_state: &mut WindowState, menu_state: &MenuState) {
    let selected = menu_state.selected_state();
    let which = selected.state.cur_pos.unwrap_or(0);

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
        ItemActivity::EpisodeTeaser => 3,
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

pub fn check_for_episodes(menu_state: &mut MenuState, variant: &WolfVariant) {
    menu_state.update_menu(Menu::MainMenu(MainMenuItem::NewGame), |entry| {
        entry.items[0].active = ItemActivity::Active;
        if variant.id == W3D3.id || variant.id == W3D6.id {
            entry.items[2].active = ItemActivity::Active;
            entry.items[4].active = ItemActivity::Active;
        }
        if variant.id == W3D6.id {
            entry.items[6].active = ItemActivity::Active;
            entry.items[8].active = ItemActivity::Active;
            entry.items[10].active = ItemActivity::Active;
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

fn numcode_name(scan: NumCode) -> &'static str {
    match scan {
        // Scan codes with >1 char names
        NumCode::Escape => "Esc",
        NumCode::BackSpace => "BkSp",
        NumCode::Tab => "Tab",
        NumCode::Control => "Ctrl",
        NumCode::LShift => "LShft",
        NumCode::Space => "Space",
        NumCode::CapsLock => "CapsLk",
        NumCode::F1 => "F1",
        NumCode::F2 => "F2",
        NumCode::F3 => "F3",
        NumCode::F4 => "F4",
        NumCode::F5 => "F5",
        NumCode::F6 => "F6",
        NumCode::F7 => "F7",
        NumCode::F8 => "F8",
        NumCode::F9 => "F9",
        NumCode::F10 => "F10",
        NumCode::F11 => "F11",
        NumCode::F12 => "F12",
        NumCode::ScrollLock => "ScrlLk",
        NumCode::Return => "Enter",
        NumCode::RShift => "RShft",
        NumCode::PrintScreen => "PrtSc",
        NumCode::Alt => "Alt",
        NumCode::Home => "Home",
        NumCode::PgUp => "PgUp",
        NumCode::End => "End",
        NumCode::PgDn => "PgDn",
        NumCode::Insert => "Ins",
        NumCode::Delete => "Del",
        NumCode::NumLock => "NumLk",
        NumCode::UpArrow => "Up",
        NumCode::DownArrow => "Down",
        NumCode::LeftArrow => "Left",
        NumCode::RightArrow => "Right",
        NumCode::None => "",
        // Scan code names with single chars
        NumCode::Num1 => "1",
        NumCode::Num2 => "2",
        NumCode::Num3 => "3",
        NumCode::Num4 => "4",
        NumCode::Num5 => "5",
        NumCode::Num6 => "6",
        NumCode::Num7 => "7",
        NumCode::Num8 => "8",
        NumCode::Num9 => "9",
        NumCode::Num0 => "0",
        NumCode::Minus => "-",
        NumCode::Equals => "=",
        NumCode::Q => "Q",
        NumCode::W => "W",
        NumCode::E => "E",
        NumCode::R => "R",
        NumCode::T => "T",
        NumCode::Y => "Y",
        NumCode::U => "U",
        NumCode::I => "I",
        NumCode::O => "O",
        NumCode::P => "P",
        NumCode::LeftBracket => "[",
        NumCode::RightBracket => "]",
        NumCode::A => "A",
        NumCode::S => "S",
        NumCode::D => "D",
        NumCode::F => "F",
        NumCode::G => "G",
        NumCode::H => "H",
        NumCode::J => "J",
        NumCode::K => "K",
        NumCode::L => "L",
        NumCode::Semicolon => ";",
        NumCode::Backslash => "\\",
        NumCode::Z => "Z",
        NumCode::X => "X",
        NumCode::C => "C",
        NumCode::V => "V",
        NumCode::B => "B",
        NumCode::N => "N",
        NumCode::M => "M",
        NumCode::Comma => ",",
        NumCode::Slash => "/",
        NumCode::Plus => "+",
        _ => "?",
    }
}
