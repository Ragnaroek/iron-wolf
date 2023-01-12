use std::path::Path;

use libiw::map::{MapType, MapFileType};
use libiw::gamedata::Texture;

pub const MAP_SIZE : usize = 64;

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
	pub actor_at: [[Option<ObjType>;MAP_SIZE]; MAP_SIZE],
}

/// State for one level
pub struct LevelState {
    pub level: Level,
    pub actors: Vec<ObjType>,
}

impl LevelState {
    pub fn mut_player(&mut self) -> &mut ObjType {
        &mut self.actors[0]
    }

    pub fn player(&self) -> &ObjType {
        &self.actors[0]
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

#[derive(Clone, Copy)]
pub struct ObjType {
	pub angle: u32,
    pub pitch: u32,
	pub tilex: usize,
	pub tiley: usize,
	pub x: i32,
	pub y: i32,
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