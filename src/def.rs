#[derive(Copy, Clone)]
pub enum WeaponType {
	Knife,
	Pistol,
	MachineGun,
	ChainGun,
}

pub struct GameState {
	pub mapon: usize,
	pub score: usize,
	pub lives: usize,
	pub health: usize,
	pub ammo: usize,
	pub keys: usize,
	pub weapon: WeaponType,

	pub face_frame: usize,
}