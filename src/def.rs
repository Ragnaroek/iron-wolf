use crate::assets::{DigiChannel, SoundName};
use crate::fixed::Fixed;
use crate::gamedata::{GamedataHeaders, SpriteData, TextureData};
use crate::map::{MapFileType, MapSegs, MapType};
use crate::play::ProjectionConfig;
use crate::sd::Sound;
use crate::vga_render::{PAGE_1_START, PAGE_2_START, PAGE_3_START, VGARenderer};
use opl::AdlSound;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

pub const MAX_STATS: usize = 400;
pub const MAX_DOORS: usize = 64;

// tile constants

pub const ICON_ARROWS: u16 = 90;
pub const PUSHABLE_TILE: u16 = 98;
pub const NUM_AREAS: usize = 37;
pub const ELEVATOR_TILE: u16 = 21;
pub const AMBUSH_TILE: u16 = 106;
pub const ALT_ELEVATOR_TILE: u16 = 107;

pub const HEIGHT_RATIO: f64 = 0.5;

pub const GLOBAL1: i32 = 1 << 16;
pub const MAP_SIZE: usize = 64;
pub const MIN_DIST: i32 = 0x5800;
pub const PLAYER_SIZE: i32 = MIN_DIST;
pub const ANGLES: usize = 360; //must be divisable by 4
pub const ANGLES_I32: i32 = ANGLES as i32;
pub const ANGLE_QUAD: usize = ANGLES / 4;
pub const TILEGLOBAL: i32 = 1 << 16;

pub const EXTRA_POINTS: u32 = 40000;

pub const RUN_SPEED: i32 = 6000;

pub const MIN_ACTOR_DIST: i32 = 0x10000;

pub const TILESHIFT: i32 = 16;
pub const UNSIGNEDSHIFT: i32 = 8;

pub const FOCAL_LENGTH: i32 = 0x5700;
pub const FINE_ANGLES: usize = 3600;

pub const NUM_BUTTONS: usize = 8;
pub const NUM_WEAPONS: usize = 4;

pub const FL_SHOOTABLE: u8 = 1;
pub const FL_BONUS: u8 = 2;
pub const FL_NEVERMARK: u8 = 4;
pub const FL_VISABLE: u8 = 8;
pub const FL_ATTACKMODE: u8 = 16;
pub const FL_FIRSTATTACK: u8 = 32;
pub const FL_AMBUSH: u8 = 64;
pub const FL_NONMARK: u8 = 128;

pub const SPD_PATROL: i32 = 512;
pub const SPD_DOG: i32 = 1500;

pub const STATUS_LINES: usize = 40;
pub static SCREENLOC: [usize; 3] = [PAGE_1_START, PAGE_2_START, PAGE_3_START];

pub const EXIT_TILE: u16 = 99;

pub static DIR_ANGLE: [usize; 9] = [
    0,
    ANGLES / 8,
    2 * ANGLES / 8,
    3 * ANGLES / 8,
    4 * ANGLES / 8,
    5 * ANGLES / 8,
    6 * ANGLES / 8,
    7 * ANGLES / 8,
    ANGLES,
];

#[macro_export]
macro_rules! derive_from {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<usize> for $name {
            type Error = ();

            fn try_from(v: usize) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as usize => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}
pub use derive_from;

pub struct DigiSound {
    pub chunk: Box<[u8]>,
    pub channel: DigiChannel,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
#[repr(usize)]
pub enum WeaponType {
    Knife = 0,
    Pistol = 1,
    MachineGun = 2,
    ChainGun = 3,
}

impl WeaponType {
    // Note: An illegal usize will be mapped to Knife (everything > 3)
    pub fn from_usize(i: usize) -> WeaponType {
        match i {
            0 => WeaponType::Knife,
            1 => WeaponType::Pistol,
            2 => WeaponType::MachineGun,
            3 => WeaponType::ChainGun,
            _ => WeaponType::Knife,
        }
    }
}

/// static level data (map and actors)
pub struct Level {
    pub map_segs: MapSegs, // contains the unmodified loaded map data from the asset file
    pub tile_map: Vec<Vec<u16>>, // map plan, plane 0 (will be manipulated during play and on level load)
    pub info_map: Vec<Vec<u16>>, // info plane, plane 1 (will be manipulated during play and on leve load)
}

#[derive(Debug)]
pub struct Control {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    Nothing,
    Wall(u16),
    Obj(ObjKey),
}

#[derive(Clone, Copy)]
pub struct VisObj {
    pub view_x: i32,
    pub view_height: i32,
    pub sprite: Sprite,
}

/// State for one level
pub struct LevelState {
    pub level: Level,
    pub map_width: usize,
    /// Player stuff
    pub actor_at: Vec<Vec<At>>,
    pub actors: Vec<ObjType>,
    /// Door stuff
    pub doors: Vec<DoorType>,
    pub area_connect: Vec<Vec<u8>>, // len() is NUM_AREAS
    pub area_by_player: Vec<bool>,  // len() is NUM_AREAS
    pub statics: Vec<StaticType>,
    pub spotvis: Vec<Vec<bool>>,
    pub vislist: Vec<VisObj>, // allocate this once and re-use
    //misc
    pub thrustspeed: i32,
    pub last_attacker: Option<ObjKey>,
}

// This is the key of the actor in the LevelState actors[] array
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjKey(pub usize);

pub const PLAYER_KEY: ObjKey = ObjKey(0); // The player is always at position 0

impl LevelState {
    #[inline]
    pub fn mut_player(&mut self) -> &mut ObjType {
        &mut self.actors[0]
    }

    #[inline]
    pub fn player(&self) -> &ObjType {
        &self.actors[0]
    }

    #[inline]
    pub fn obj(&self, k: ObjKey) -> &ObjType {
        &self.actors[k.0]
    }

    #[inline]
    pub fn mut_obj(&mut self, k: ObjKey) -> &mut ObjType {
        &mut self.actors[k.0]
    }

    #[inline]
    pub fn update_obj<F>(&mut self, k: ObjKey, f: F)
    where
        F: FnOnce(&mut ObjType),
    {
        f(&mut self.actors[k.0])
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LevelState),
    {
        f(self)
    }
}

derive_from! {
    #[repr(usize)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum Dir {
        North,
        East,
        South,
        West,
    }
}

/// State about the controls
pub struct ControlState {
    pub control: Control,
    pub angle_frac: i32,
    pub button_held: [bool; NUM_BUTTONS],
    pub button_state: [bool; NUM_BUTTONS],
}

impl ControlState {
    pub fn set_button_held(&mut self, button: Button, held: bool) {
        self.button_held[button as usize] = held;
    }

    pub fn button_held(&self, button: Button) -> bool {
        self.button_held[button as usize]
    }

    pub fn button_state(&self, button: Button) -> bool {
        self.button_state[button as usize]
    }
}

// nums here are an index into ControlState::button_state
#[repr(usize)]
#[derive(Debug)]
pub enum Button {
    NoButton = usize::MAX,
    Attack = 0,
    Strafe = 1,
    Run = 2,
    Use = 3,
    ReadyKnife = 4,
    ReadyPistol = 5,
    ReadyMachineGun = 6,
    ReadyChainGun = 7,
}

impl Button {
    pub fn from_usize(u: usize) -> Button {
        match u {
            0 => Button::Attack,
            1 => Button::Strafe,
            2 => Button::Run,
            3 => Button::Use,
            4 => Button::ReadyKnife,
            5 => Button::ReadyPistol,
            6 => Button::ReadyMachineGun,
            7 => Button::ReadyChainGun,
            _ => Button::NoButton,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(usize)]
pub enum Difficulty {
    Baby,   // Can I play, Daddy?
    Easy,   // Don't hurt me.
    Medium, // Bring 'em on!
    Hard,   // I am Death incarnate!
}

impl Difficulty {
    pub fn from_pos(i: usize) -> Difficulty {
        match i {
            0 => Difficulty::Baby,
            1 => Difficulty::Easy,
            2 => Difficulty::Medium,
            3 => Difficulty::Hard,
            _ => Difficulty::Baby,
        }
    }
}

pub struct LevelRatio {
    pub kill: i32,
    pub secret: i32,
    pub treasure: i32,
    pub time: i32, // in seconds
}

/// State across the whole game
pub struct GameState {
    pub loaded_game: bool,
    pub start_game: bool,

    pub died: bool,
    pub difficulty: Difficulty,
    pub map_on: usize,
    pub old_score: u32,
    pub score: u32,
    pub next_extra: u32,
    pub lives: i32,
    pub health: i32,
    pub ammo: i32,
    pub keys: i32,

    pub best_weapon: WeaponType,
    pub weapon: Option<WeaponType>,
    pub chosen_weapon: WeaponType,

    pub face_frame: usize,
    pub attack_frame: usize,
    pub attack_count: i32,
    pub weapon_frame: usize,

    pub episode: usize,
    pub secret_count: i32,
    pub treasure_count: i32,
    pub kill_count: i32,
    pub secret_total: i32,
    pub treasure_total: i32,
    pub kill_total: i32,

    pub time_count: u64,
    pub kill_x: usize,
    pub kill_y: usize,

    pub victory_flag: bool,
    pub play_state: PlayState,
    pub killer_obj: Option<ObjKey>,
    // cheats
    pub god_mode: bool,

    pub face_count: u64,

    pub made_noise: bool,

    pub got_gat_gun: bool,

    pub bonus_count: i32,
    pub damage_count: i32,

    pub pal_shifted: bool,
    pub fizzle_in: bool,
    // push wall states
    pub push_wall_state: u64, // push wall animation going on
    pub push_wall_pos: i32,   // amount a pushable wall has been moved (0-63)
    pub push_wall_x: usize,
    pub push_wall_y: usize,
    pub push_wall_dir: Dir,

    pub level_ratios: Vec<LevelRatio>,
}

pub fn new_game_state() -> GameState {
    let mut level_ratios = Vec::with_capacity(8);
    for _ in 0..8 {
        level_ratios.push(LevelRatio {
            kill: 0,
            secret: 0,
            treasure: 0,
            time: 0,
        });
    }

    GameState {
        loaded_game: false,
        start_game: false,
        died: false,
        map_on: 0,
        difficulty: Difficulty::Hard,
        old_score: 0,
        score: 0,
        next_extra: EXTRA_POINTS,
        lives: 3,
        health: 100,
        ammo: 8,
        keys: 0,
        best_weapon: WeaponType::Pistol,
        weapon: Some(WeaponType::Pistol),
        chosen_weapon: WeaponType::Pistol,
        weapon_frame: 0,
        face_frame: 0,
        episode: 0,
        secret_count: 0,
        treasure_count: 0,
        kill_count: 0,
        secret_total: 0,
        treasure_total: 0,
        kill_total: 0,
        time_count: 0,
        kill_x: 0,
        kill_y: 0,
        victory_flag: false,
        god_mode: false,
        got_gat_gun: false,
        play_state: PlayState::StillPlaying,
        killer_obj: None,
        attack_frame: 0,
        attack_count: 0,
        face_count: 0,
        made_noise: false,
        damage_count: 0,
        bonus_count: 0,
        pal_shifted: false,
        fizzle_in: false,
        push_wall_state: 0,
        push_wall_pos: 0,
        push_wall_x: 0,
        push_wall_y: 0,
        push_wall_dir: crate::def::Dir::North,
        level_ratios,
    }
}

impl GameState {
    /// Set up new game to start from the beginning
    /// Keeps the difficulty and episode setting
    pub fn prepare_episode_select(&mut self) {
        let difficulty = self.difficulty;
        let episode = self.episode;
        *self = new_game_state();
        self.difficulty = difficulty;
        self.episode = episode;
    }
}

pub struct WindowState {
    pub window_x: usize,
    pub window_y: usize,
    pub window_w: usize,
    pub window_h: usize,

    pub print_x: usize,
    pub print_y: usize,

    pub font_number: usize,
    pub font_color: u8,
    pub back_color: u8,

    pub debug_ok: bool,

    pub in_game: bool,
}

impl WindowState {
    pub fn set_font_color(&mut self, f: u8, b: u8) {
        self.font_color = f;
        self.back_color = b;
    }
}

#[derive(Debug, PartialEq)]
pub enum PlayState {
    StillPlaying,
    Completed,
    Died,
    Warped,
    ResetGame,
    LoadedGame,
    Victorious,
    Abort,
    DemoDone,
    SecretLevel,
}

derive_from! {
    #[repr(usize)]
    #[derive(Eq, PartialEq, Debug, Clone, Copy)]
    pub enum DirType {
        East = 0,
        NorthEast = 1,
        North = 2,
        NorthWest = 3,
        West = 4,
        SouthWest = 5,
        South = 6,
        SouthEast = 7,
        NoDir = 8,
    }
}

pub const NUM_ENEMIES: usize = 22;

#[derive(Debug)]
pub enum EnemyType {
    Guard,
    Officer,
    SS,
    Dog,
    Boss,
    Schabbs,
    Fake,
    Hitler,
    Mutant,
    Blinky,
    Clyde,
    Pinky,
    Inky,
    Gretel,
    Gift,
    Fat,
    Spectre,
    Angel,
    Trans,
    Uber,
    Will,
    Death,
}

derive_from! {
    #[repr(usize)]
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub enum ClassType {
        Nothing,
        Player,
        Inert,
        Guard,
        Officer,
        SS,
        Dog,
        Boss,
        Schabb,
        Fake,
        MechaHitler,
        Mutant,
        Needle,
        Fireo,
        BJ,
        Ghost,
        RealHitler,
        Gretel,
        Gift,
        Fat,
        Rocket,

        Spectre,
        Angel,
        Trans,
        Uber,
        Will,
        Death,
        HRocket,
        Spark,
    }
}

#[repr(i16)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ActiveType {
    BadObject = -1,
    No = 0,
    Yes = 1,
    Always = 2,
}

impl TryFrom<i16> for ActiveType {
    type Error = ();

    fn try_from(v: i16) -> Result<Self, Self::Error> {
        match v {
            -1 => Ok(ActiveType::BadObject),
            0 => Ok(ActiveType::No),
            1 => Ok(ActiveType::Yes),
            2 => Ok(ActiveType::Always),
            _ => Err(()),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)] //XXX do not make this Clone, fix actor_at (also takes a ObjKey instead ObjType???)
pub struct ObjType {
    pub active: ActiveType,
    pub tic_count: u32,
    pub class: ClassType,
    pub state: Option<&'static StateType>,

    pub flags: u8,

    pub distance: i32,
    pub dir: DirType,

    pub x: i32, // TODO should be of Fixed type?
    pub y: i32, // TODO should be of Fixed type?
    pub tilex: usize,
    pub tiley: usize,
    pub area_number: usize,

    pub view_x: i32,
    pub view_height: i32,
    pub trans_x: Fixed, // in global coord
    pub trans_y: Fixed,

    pub angle: i32,
    pub hitpoints: i32,
    pub speed: i32,

    pub temp1: i32,
    pub temp2: i32,
    pub temp3: i32,

    pub pitch: u32, // TODO not in the original ObjTyp, will not be restored on save load. Ok?
}

derive_from! {
    #[repr(usize)]
    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    pub enum DoorAction {
        Open,
        Closed,
        Opening,
        Closing,
    }
}

derive_from! {
    #[repr(usize)]
    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    pub enum DoorLock {
        Normal,
        Lock1,
        Lock2,
        Lock3,
        Lock4,
        Elevator,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DoorType {
    pub num: usize,
    pub tile_x: usize,
    pub tile_y: usize,
    pub vertical: bool,
    pub lock: DoorLock,
    pub action: DoorAction,
    pub tic_count: u32,
    pub position: u16,
}

#[derive(Debug, PartialEq)]
pub struct StaticType {
    pub tile_x: usize,
    pub tile_y: usize,
    pub sprite: Sprite,
    pub flags: u8,
    pub item_number: StaticKind,
}

// iron-wolf specific configuration
#[derive(Deserialize, Debug)]
pub struct IWConfig {
    #[serde(default = "default_vanilla")]
    pub vanilla: bool,

    #[serde(default)]
    pub data: IWConfigData,
    #[serde(default)]
    pub options: IWConfigOptions,
}

#[derive(Deserialize, Debug, Default)]
pub struct IWConfigData {
    #[serde(default = "default_path")]
    pub wolf3d_data: PathBuf,
    pub patch_data: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Default)]
pub struct IWConfigOptions {
    #[serde(default)]
    pub no_wait: bool,
    #[serde(default)]
    pub fast_psyched: bool,
    #[serde(default)]
    pub enable_debug: bool,
    #[serde(default = "true_default")]
    pub fullscreen: bool,
}

fn true_default() -> bool {
    true
}

fn default_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("./");
    path
}

fn default_vanilla() -> bool {
    true
}

// All assets that need to be accessed in the game loop
pub struct Assets {
    pub map_headers: Vec<MapType>,
    pub map_offsets: MapFileType,
    pub textures: Vec<TextureData>,
    pub sprites: Vec<SpriteData>,
    pub gamedata_headers: GamedataHeaders,
    pub game_maps: Vec<u8>,
    pub audio_headers: Vec<u32>,
    pub audio_sounds: Vec<AdlSound>,
    pub digi_sounds: HashMap<SoundName, DigiSound>,
}

type Think = fn(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
);

type Action = fn(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
);

#[derive(Eq, PartialEq, Debug)]
pub struct StateType {
    pub id: u16, // non-changing ID of this state, will be used in save games
    pub rotate: usize,
    pub sprite: Option<Sprite>, // None means get from obj->temp1
    pub tic_time: u32,
    pub think: Option<Think>,
    pub action: Option<Action>,
    pub next: Option<&'static StateType>,
}

derive_from! {
    #[repr(usize)]
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub enum Sprite {
        None = usize::MAX,

        Demo = 0,
        DeathCam = 1,

        // static sprites
        Stat0 = 2, Stat1 = 3, Stat2 = 4, Stat3 = 5,
        Stat4 = 6, Stat5 = 7, Stat6 = 8, Stat7 = 9,
        Stat8 = 10, Stat9 = 11, Stat10 = 12, Stat11 = 13,
        Stat12 = 14, Stat13 = 15, Stat14 = 16, Stat15 = 17,
        Stat16 = 18, Stat17 = 19, Stat18 = 20, Stat19 = 21,
        Stat20 = 22, Stat21 = 23, Stat22 = 24, Stat23 = 25,
        Stat24 = 26, Stat25 = 27, Stat26 = 28, Stat27 = 29,
        Stat28 = 30, Stat29 = 31, Stat30 = 32, Stat31 = 33,
        Stat32 = 34, Stat33 = 35, Stat34 = 36, Stat35 = 37,
        Stat36 = 38, Stat37 = 39, Stat38 = 40, Stat39 = 41,
        Stat40 = 42, Stat41 = 43, Stat42 = 44, Stat43 = 45,
        Stat44 = 46, Stat45 = 47, Stat46 = 48, Stat47 = 49,

        // guard
        GuardS1 = 50, GuardS2 = 51, GuardS3 = 52, GuardS4 = 53,
        GuardS5 = 54, GuardS6 = 55, GuardS7 = 56, GuardS8 = 57,

        GuardW11 = 58, GuardW12 = 59, GuardW13 = 60, GuardW14 = 61,
        GuardW15 = 62, GuardW16 = 63, GuardW17 = 64, GuardW18 = 65,

        GuardW21 = 66, GuardW22 = 67, GuardW23 = 68, GuardW24 = 69,
        GuardW25 = 70, GuardW26 = 71, GuardW27 = 72, GuardW28 = 73,

        GuardW31 = 74 ,GuardW32 = 75, GuardW33 = 76, GuardW34 = 77,
        GuardW35 = 78, GuardW36 = 79, GuardW37 = 80, GuardW38 = 81,

        GuardW41 = 82, GuardW42 = 83, GuardW43 = 84, GuardW44 = 85,
        GuardW45 = 86, GuardW46 = 87, GuardW47 = 88, GuardW48 = 89,

        GuardPain1 = 90, GuardDie1 = 91, GuardDie2 = 92, GuardDie3 = 93,
        GuardPain2 = 94, GuardDead = 95,

        GuardShoot1 = 96, GuardShoot2 = 97, GuardShoot3 = 98,

        // dogs
        DogW11 = 99, DogW12 = 100, DogW13 = 101, DogW14 = 102,
        DogW15 = 103, DogW16 = 104, DogW17 = 105, DogW18 = 106,

        DogW21 = 107, DogW22 = 108, DogW23 = 109, DogW24 = 110,
        DogW25 = 111, DogW26 = 112, DogW27 = 113, DogW28 = 114,

        DogW31 = 115, DogW32 = 116, DogW33 = 117, DogW34 = 118,
        DogW35 = 119, DogW36 = 120, DogW37 = 121, DogW38 = 122,

        DogW41 = 123, DogW42 = 124, DogW43 = 125, DogW44 = 126,
        DogW45 = 127, DogW46 = 128, DogW47 = 129, DogW48 = 130,

        DogDie1 = 131, DogDie2 = 132, DogDie3 = 133, DogDead = 134,
        DogJump1 = 135, DogJump2 = 136, DogJump3 = 137,

        // SS
        SSS1 = 138, SSS2 = 139, SSS3 = 140, SSS4 = 141,
        SSS5 = 142, SSS6 = 143, SSS7 = 144, SSS8 = 145,

        SSW11 = 146, SSW12 = 147, SSW13 = 148, SSW14 = 149,
        SSW15 = 150, SSW16 = 151, SSW17 = 152, SSW18 = 153,

        SSW21 = 154, SSW22 = 155, SSW23 = 156, SSW24 = 157,
        SSW25 = 158, SSW26 = 159, SSW27 = 160, SSW28 = 161,

        SSW31 = 162, SSW32 = 163, SSW33 = 164, SSW34 = 165,
        SSW35 = 166, SSW36 = 167, SSW37 = 168, SSW38 = 169,

        SSW41 = 170, SSW42 = 171, SSW4 = 172, SSW44 = 173,
        SSW45 = 174, SSW46 = 175, SSW47 = 176, SSW48 = 177,

        SSPAIN1 = 178, SSDIE1 = 179, SSDIE2 = 180, SSDIE3 = 181,
        SSPAIN2 = 182, SSDEAD = 183,

        SSSHOOT1 = 184, SSSHOOT2 = 185, SSSHOOT3 = 186,

        // mutant
        MutantS1 = 187, MutantS2 = 188, MutantS3 = 189, MutantS4 = 190,
        MutantS5 = 191, MutantS6 = 192, MutantS7 = 193, MutantS8 = 194,

        MutantW11 = 195, MutantW12 = 196, MutantW13 = 197, MutantW14 = 198,
        MutantW15 = 199, MutantW16 = 200, MutantW17 = 201, MutantW18 = 202,

        MutantW21 = 203, MutantW22 = 204, MutantW23 = 205, MutantW24 = 206,
        MutantW25 = 207, MutantW26 = 208, MutantW27 = 209, MutantW28 = 210,

        MutantW31 = 211, MutantW32 = 212, MutantW33 = 213, MutantW34 = 214,
        MutantW35 = 215, MutantW36 = 216, MutantW37 = 217, MutantW38 = 218,

        MutantW41 = 219, MutantW42 = 220, MutantW43 = 221, MutantW44 = 222,
        MutantW45 = 223, MutantW46 = 224, MutantW47 = 225, MutantW48 = 226,

        MutantPain1 = 227, MutantDie1 = 228, MutantDie2 = 229, MutantDie3 = 230,
        MutantPain2 = 231, MutantDie4 = 232, MutantDead = 233,

        MutantShoot1 = 234, MutantShoot2 = 235, MutantShoot3 = 236, MutantShoot4 = 237,

        // officer
        OfficerS1 = 238, OfficerS2 = 239, OfficerS3 = 240, OfficerS4 = 241,
        OfficerS5 = 242, OfficerS6 = 243, OfficerS7 = 244, OfficerS8 = 245,

        OfficerW11 = 246, OfficerW12 = 247, OfficerW13 = 248, OfficerW14 = 249,
        OfficerW15 = 250, OfficerW16 = 251, OfficerW17 = 252, OfficerW18 = 253,

        OfficerW21 = 254, OfficerW22 = 255, OfficerW23 = 256, OfficerW24 = 257,
        OfficerW25 = 258, OfficerW26 = 259, OfficerW27 = 260, OfficerW28 = 261,

        OfficerW31 = 262, OfficerW32 = 263, OfficerW33 = 264, OfficerW34 = 265,
        OfficerW35 = 266, OfficerW36 = 267, OfficerW37 = 268, OfficerW38 = 269,

        OfficerW41 = 270, OfficerW42 = 271, OfficerW43 = 272, OfficerW44 = 273,
        OfficerW45 = 274, OfficerW46 = 275, OfficerW47 = 276, OfficerW48 = 277,

        OfficerPain1 = 278, OfficerDie1 = 279, OfficerDie2 = 280, OfficerDie3 = 281,
        OfficerPain2 = 282, OfficerDie4 = 283, OfficerDead = 284,

        OfficerShoot1 = 285, OfficerShoot2 = 286, OfficerShoot3 = 287,

        // ghosts
        BlinkyW1 = 288, BlinkyW2 = 289, PinkyW1 = 290, PinkyW2 = 291,
        ClydeW1 = 292, ClydeW2 = 293, InkyW1 = 294, InkyW2 = 295,

        // hans
        BossW1 = 296, BossW2 = 297, BossW3 = 298, BossW4 = 299,
        BossShoot1 = 300, BossShoot2 = 301, BossShoot3 = 302, BossDead = 303,

        BossDie1 = 304, BossDie2 = 305, BossDie3 = 306,

        // bj
        BJW1 = 408, BJW2 = 409, BJW3 = 410, BJW4 = 411,
        BJJump1 = 412, BJJump2 = 413, BJJump3 = 414, BJJump4 = 415,

        // player attack frames
        KnifeReady = 416, KnifeAtk1 = 417, KnifeAtk2 = 418, KnifeAtk3 = 419, KnifeAtk4 = 420,
        PistolReady = 421, PistolAtk1 = 422, PistolAtk2 = 423, PistolAtk3 = 424, PistolAtk4 = 425,
        MachinegunReady = 426, MachinegunAtk1 = 427, MachinegunAtk2 = 428, MachinegunAtk3 = 429, MachinegunAtk4 = 430,
        ChainReady = 431, ChainAtk1 = 432, ChainAtk2 = 433, ChainAtk3 = 434, ChainAtk4 = 435,
    }
}

pub struct StaticInfo {
    pub sprite: Sprite,
    pub kind: StaticKind,
}

derive_from! {
    #[repr(usize)]
    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    pub enum StaticKind {
        Dressing,
        Block,
        BoGibs,
        BoAlpo,
        BoFirstaid,
        BoKey1,
        BoKey2,
        BoKey3,
        BoKey4,
        BoCross,
        BoChalice,
        BoBible,
        BoCrown,
        BoClip,
        BoClip2,
        BoMachinegun,
        BoChaingun,
        BoFood,
        BoFullheal,
        Bo25clip,
        BoSpear,
    }
}
