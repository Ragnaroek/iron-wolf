use std::path::PathBuf;

use libiw::map::{MapType, MapFileType};
use libiw::gamedata::{TextureData, SpriteData};

use crate::play::ProjectionConfig;

pub const GLOBAL1 : i32	= 1<<16;
pub const MAP_SIZE : usize = 64;
pub const MIN_DIST : i32 = 0x5800;
pub const PLAYER_SIZE : i32 = MIN_DIST;
pub const ANGLES : usize = 360; //must be divisable by 4
pub const ANGLES_I32 : i32 = ANGLES as i32;
pub const ANGLE_QUAD : usize = ANGLES/4;
pub const TILEGLOBAL : i32 = 1<<16;
pub const TILESHIFT : i32 = 16;
pub const FOCAL_LENGTH : i32 = 0x5700;
pub const FINE_ANGLES : usize = 3600;

pub const MAX_DOORS : usize = 64;

pub const NUM_BUTTONS : usize = 8;
pub const NUM_WEAPONS : usize = 5;

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum WeaponType {
    None,
	Knife,
	Pistol,
	MachineGun,
	ChainGun,
}

/// static level data (map and actors)
pub struct Level {
	pub tile_map: Vec<Vec<u16>>
}

#[derive(Debug)]
pub struct Control {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum At {
    Nothing,
    Wall(u16),
    Obj(ObjKey),
}

/// State for one level
pub struct LevelState {
    pub level: Level,
    /// Player stuff
    pub actor_at: Vec<Vec<At>>,
    pub actors: Vec<ObjType>,
    /// Door stuff
    pub doors: Vec<DoorType>, 
}

// This is the key of the actor in the LevelState actors[] array
#[derive(Debug, Clone, Copy)]
pub struct ObjKey(pub usize);

pub const PLAYER_KEY : ObjKey = ObjKey(0); // The player is always at position 0

impl LevelState {
    pub fn mut_player(&mut self) -> &mut ObjType {
        &mut self.actors[0]
    }

    pub fn player(&self) -> &ObjType {
        &self.actors[0]
    }

    pub fn obj(&self, k: ObjKey) -> &ObjType {
        &self.actors[k.0]
    }

    pub fn mut_obj(&mut self, k: ObjKey) -> &mut ObjType {
        &mut self.actors[k.0]
    }
}

pub enum Dir {
    North,
    East,
    South,
    West
}

/// State about the controls
pub struct ControlState {
    pub control: Control,
    pub angle_frac: i32,
    pub button_held : [bool; NUM_BUTTONS],
    pub button_state : [bool; NUM_BUTTONS],
}

// nums here are an index into ControlState::button_state
#[repr(usize)]
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

/// State across the whole game
pub struct GameState {
	pub map_on: usize,
	pub score: usize,
	pub lives: usize,
	pub health: usize,
	pub ammo: usize,
	pub keys: usize,
	pub weapon: WeaponType,
    pub weapon_frame: usize,

	pub face_frame: usize,

	pub episode : usize,
}

#[derive(Debug, Clone, Copy)] //XXX do not make this Clone, fix actor_at (also takes a ObjKey instead ObjType???)
pub struct ObjType {
	pub angle: i32,
    pub pitch: u32,
	pub tilex: usize,
	pub tiley: usize,
	pub x: i32,
	pub y: i32,
    pub state: &'static StateType,
}

#[derive(Eq, PartialEq)]
pub enum DoorAction {
    Open,
    Closed,
    Opening,
    Closing
}

pub struct DoorType {
    pub num: u16,
    pub tile_x: usize,
	pub tile_y: usize,
    pub vertical: bool,
    pub lock: u16,
    pub action: DoorAction,
    pub tic_count: u32,
    pub position: u16,
}

// iron-wolf specific configuration
pub struct IWConfig {
	pub wolf3d_data: PathBuf,
    pub no_wait: bool,
}

// All assets that need to be accessed in the game loop
pub struct Assets {
	pub iw_config: IWConfig, // put here for convenience (mabye only put assets path here?)
	pub map_headers: Vec<MapType>,
	pub map_offsets: MapFileType,
    pub textures: Vec<TextureData>,
    pub sprites: Vec<SpriteData>,
}

type Think = fn(k: ObjKey, level_state: &mut LevelState, &mut ControlState, prj: &ProjectionConfig); 

#[derive(Debug)]
pub struct StateType {
    pub think: Option<Think>,
    pub next: Option<&'static StateType>
}

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Sprite {

    None = 0,

    KnifeReady = 416, KnifeAtk1 = 417, KnifeAtk2 = 418, KnifeAtk3 = 419, KnifeAtk4 = 420, 
    PistolReady = 421, PistolAtk1 = 422, PistolAtk2 = 423, PistolAtk3 = 424, PistolAtk4 = 425,  
    MachinegunReady = 426, MachinegunAtk1 = 427, MachinegunAtk2 = 428, MachinegunAtk3 = 429, MachinegunAtk4 = 430,
    ChainReady = 431, ChainAtk1 = 432, ChainAtk2 = 433, ChainAtk3 = 434, ChainAtk4 = 435,  
}