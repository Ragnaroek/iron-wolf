#![crate_name = "iw"]
#![crate_type = "lib"]

#![feature(ascii_char)]
#![feature(ascii_char_variants)]

pub mod act1;
pub mod act2;
pub mod agent;
pub mod assets;
pub mod config;
pub mod def;
pub mod draw;
pub mod fixed;
pub mod game;
pub mod gamedata;
pub mod input;
pub mod inter;
pub mod loader;
pub mod map;
pub mod menu;
pub mod play;
pub mod time;
pub mod scale;
pub mod start;
pub mod state;
pub mod us1;
pub mod user;
pub mod util;
pub mod vga_render;
pub mod vl;
pub mod vh;

#[cfg(feature = "web")]
pub mod web;