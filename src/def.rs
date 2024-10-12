#[cfg(test)]
#[path = "./def_test.rs"]
mod draw_test;

use std::path::Path;
use std::fmt;

use libiw::map::{MapType, MapFileType};
use libiw::gamedata::Texture;

use crate::play::ProjectionConfig;

pub const GLOBAL1 : i32	= 1<<16;

pub const MAP_SIZE : usize = 64;

pub const MIN_DIST : i32 = 0x5800;
pub const PLAYER_SIZE : i32 = MIN_DIST;

pub const ANGLES : usize = 360; //must be divisable by 4
pub const ANGLE_QUAD : usize = ANGLES/4;

pub const TILEGLOBAL : i32 = 1<<16;

#[derive(PartialEq, Clone, Copy)]
pub struct Fixed(i32); //16:16 fixed point

pub fn new_fixed_u16(int_part: u16, frac_part: u16) -> Fixed {
    new_fixed_u32((int_part as u32) << 16 | frac_part as u32)
}

pub fn new_fixed_i16(int_part: i16, frac_part: i16) -> Fixed {
    new_fixed(int_part as i32, frac_part as i32)
}

pub fn new_fixed(int_part: i32, frac_part: i32) -> Fixed {
    Fixed(int_part << 16 | frac_part)
}

pub fn new_fixed_i32(raw: i32) -> Fixed {
    Fixed(raw)
}

pub fn new_fixed_u32(raw: u32) -> Fixed {
    Fixed(raw as i32)
}

impl Fixed {
    pub fn to_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0 >> 16, self.0 & 0xFFFF)
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i = self.0 >> 16;
        let frac = self.0 & 0xFFFF;
        write!(f, "{:#04x}.{:#04x}({}.{})", i, frac, i, frac)
    }
}

impl std::ops::Neg for Fixed {
    type Output = Self;

    fn neg(self) -> Self::Output {
        new_fixed_i32(-self.0)
    }
}

#[derive(Copy, Clone)]
pub enum WeaponType {
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
    pub actor_at: Vec<Vec<At>>,
    pub actors: Vec<ObjType>,
    
    /// Player stuff (TODO maybe move to own state?)
    
    /// Control diff from last frame for player
    pub control: Control,
    pub angle_frac: i32,
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

type Think = fn(k: ObjKey, level_state: &mut LevelState, prj: &ProjectionConfig); 

pub struct StateType {
    pub think: Option<Think>,
    pub next: Option<&'static StateType>
}