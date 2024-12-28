#![crate_name = "iw"]
#![crate_type = "lib"]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(type_alias_impl_trait)]
#![feature(stmt_expr_attributes)]

pub mod act1;
pub mod act2;
pub mod agent;
pub mod assets;
pub mod config;
pub mod debug;
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
pub mod patch;
pub mod play;
pub mod scale;
pub mod sd;
pub mod start;
pub mod state;
pub mod text;
pub mod time;
pub mod us1;
pub mod user;
pub mod util;
pub mod vga_render;
pub mod vh;
pub mod vl;

#[cfg(feature = "web")]
pub mod web;
