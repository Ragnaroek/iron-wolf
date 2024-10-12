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
}

pub struct MapType {
	pub plane_start: [i32; 3],
	pub plane_length: [u16; 3],
	pub width: u16,
	pub height: u16,
	pub name: String,
}

// All assets that need to be accessed in the game loop
pub struct Assets {
	pub map_headers: Vec<MapType>
}