use std::path::Path;

use libiw::map::{MapType, MapFileType};

#[derive(Copy, Clone)]
pub enum WeaponType {
	Knife,
	Pistol,
	MachineGun,
	ChainGun,
}

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
	pub angle: f64, //in radian
	pub tilex: usize,
	pub tiley: usize,
	pub x: u32,
	pub y: u32,
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
}