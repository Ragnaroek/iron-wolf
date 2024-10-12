
use crate::def::MAX_DOORS;

use super::def::{Assets, ObjType, Level, LevelState, At, MAP_SIZE, PLAYER_KEY};
use super::assets::load_map_from_assets;
use super::act1::{spawn_door};
use super::agent::{spawn_player, thrust};
use super::play::ProjectionConfig;

pub const AREATILE : u16 = 107;

pub const NORTH : i32 = 0;
pub const EAST : i32 = 0;
pub const SOUTH : i32 = 0;
pub const WEST : i32 = 0;

pub const ANGLE_45 : u32 = 0x20000000;
pub const ANGLE_90 : u32 = ANGLE_45*2;
pub const ANGLE_180 : u32 = ANGLE_45*4;
pub const ANGLE_1 : u32 = ANGLE_45/45;

pub fn setup_game_level(prj: &ProjectionConfig, map_on: usize, assets: &Assets) -> Result<LevelState, String> {
	let map = &assets.map_headers[map_on];
	if map.width != MAP_SIZE as u16 || map.height != MAP_SIZE as u16 {
		panic!("Map not 64*64!");
	}

	let map_data = load_map_from_assets(assets, map_on)?;

    let mut tile_map = vec![vec![0; MAP_SIZE]; MAP_SIZE];
    let mut actor_at = vec![vec![At::Nothing; MAP_SIZE]; MAP_SIZE];

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[0][map_ptr];
			map_ptr += 1;
			if tile > 0 && tile < AREATILE {
				tile_map[x][y] = tile;
                actor_at[x][y] = At::Wall(tile);
			}
		}
	}

	// spawn doors
	map_ptr = 0;
	let mut doornum = 0;
	let mut doors = Vec::with_capacity(MAX_DOORS);
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[0][map_ptr];
			map_ptr += 1;
			if tile >= 90 && tile <= 101 {
				let door = match tile {
					90 | 92 | 94 | 96 | 98 | 100 => spawn_door(&mut tile_map, doornum, x, y, true, (tile-90)/2),
					91 | 93 | 95 | 97 | 99 | 101 => spawn_door(&mut tile_map, doornum, x, y, false, (tile-91)/2),
					_ => unreachable!("tile guaranteed to be in range through the if check")
				};
				doors.push(door);
				doornum += 1;
			}
		}
	}

	let player = scan_info_plane( &map_data);
    let actors = init_actors(player);

    let mut level_state = LevelState{
        level: Level {
		    tile_map,
        },
        actors,
        actor_at,
		doors,
	};

    thrust(PLAYER_KEY, &mut level_state, prj, 0, 0); // set some variables
	//TODO init_static_list?

	//TODO ambush markers
	Ok(level_state)
}

fn init_actors(player: ObjType) -> Vec<ObjType> {
    vec![player]
    //TODO init NP actors here
}

//Returns the player object
fn scan_info_plane(map_data: &libiw::map::MapData) -> ObjType {
	let mut player = None;

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[1][map_ptr];
			map_ptr += 1;
			match tile {
				19..=22 => player = Some(spawn_player(x, y, NORTH+(tile-19)as i32)),
				_ => {},
			}
		}
	}

	if player.is_none() {
		panic!("No player start position in map");
	}

	player.unwrap()
}

