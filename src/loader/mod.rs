#[cfg(any(feature = "sdl", feature = "egui"))]
pub mod loader_disk;
#[cfg(any(feature = "sdl", feature = "egui"))]
pub use loader_disk::Loader;

#[cfg(feature = "web")]
pub mod loader_web;
#[cfg(feature = "web")]
pub use loader_web::Loader;
