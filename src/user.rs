
#[derive(Debug)]
pub struct HighScore {
	pub name: String, //58 bytes
	pub score: u32,
	pub completed: u16,
	pub episode: u16,
}