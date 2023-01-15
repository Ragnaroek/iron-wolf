use std::path::Path;

use libiw::map::{MapType, MapFileType};
use libiw::gamedata::Texture;

pub const MAP_SIZE : usize = 64;

pub const ANGLES : i32 = 360; //must be divisable by 4

#[derive(Copy, Clone)]
pub enum WeaponType {
	Knife,
	Pistol,
	MachineGun,
	ChainGun,
}

/// static level data (map and actors)
pub struct Level {
	pub tile_map: [[u8;MAP_SIZE]; MAP_SIZE],
	pub actor_at: [[Option<ObjType>;MAP_SIZE]; MAP_SIZE], //XXX should not contain ObjTyp (ObjKey????)
}

#[derive(Debug)]
pub struct Control {
    pub x: i32,
    pub y: i32,
}

/// State for one level
pub struct LevelState {
    pub level: Level,
    pub actors: Vec<ObjType>,
    
    /// Player stuff (TODO maybe move to own state?)
    
    /// Control diff from last frame for player
    pub control: Control,
    pub angle_frac: i32,
}

// This is the key of the actor in the LevelState
#[derive(Clone, Copy)]
pub struct ObjKey(pub usize);

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

/// State across the whole game
pub struct GameState {
	pub map_on: usize,
	pub score: usize,
	pub lives: usize,
	pub health: usize,
	pub ammo: usize,
	pub keys: usize,
	pub weapon: WeaponType,

	pub face_frame: usize,

	pub episode : usize,
}

#[derive(Clone, Copy)] //XXX do not make this Clone, fix actor_at (also takes a ObjKey instead ObjType???)
pub struct ObjType {
	pub angle: i32,
    pub pitch: u32,
	pub tilex: usize,
	pub tiley: usize,
	pub x: i32,
	pub y: i32,
    pub state: &'static StateType,
}

// iron-wolf specific configuration
pub struct IWConfig {
	pub wolf3d_data: &'static Path,
    pub no_wait: bool,
}

// All assets that need to be accessed in the game loop
pub struct Assets {
	pub iw_config: IWConfig, // put here for convenience (mabye only put assets path here?)
	pub map_headers: Vec<MapType>,
	pub map_offsets: MapFileType,
    pub textures: Vec<Texture>,
}

type Think = fn(k: ObjKey, level_state: &mut LevelState); 

pub struct StateType {
    pub think: Option<Think>,
    pub next: Option<&'static StateType>
}