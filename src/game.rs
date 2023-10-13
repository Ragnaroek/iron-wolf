use crate::def::{Sprite, StaticType, VisObj, ObjKey, Assets, ObjType, Level, LevelState, At, MAX_STATS, MAX_DOORS, MAP_SIZE, PLAYER_KEY, DirType, EnemyType, GameState, Difficulty, };
use crate::assets::load_map_from_assets;
use crate::act1::{spawn_door, spawn_static};
use crate::act2::{dead_guard, stand};
use crate::agent::{spawn_player, thrust};
use crate::play::ProjectionConfig;

pub const AREATILE : u16 = 107;

pub const NORTH : i32 = 0;
pub const EAST : i32 = 0;
pub const SOUTH : i32 = 0;
pub const WEST : i32 = 0;

pub const ANGLE_45 : u32 = 0x20000000;
pub const ANGLE_90 : u32 = ANGLE_45*2;
pub const ANGLE_180 : u32 = ANGLE_45*4;
pub const ANGLE_1 : u32 = ANGLE_45/45;

pub fn setup_game_level(prj: &ProjectionConfig, game_state: &GameState, assets: &Assets) -> Result<LevelState, String> {
	let map = &assets.map_headers[game_state.map_on];
	if map.width != MAP_SIZE as u16 || map.height != MAP_SIZE as u16 {
		panic!("Map not 64*64!");
	}

	let map_data = load_map_from_assets(assets, game_state.map_on)?;

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

	let (actors, statics) = scan_info_plane(&map_data, &mut actor_at, game_state.difficulty);
    let mut level_state = LevelState{
        level: Level {
		    tile_map,
        },
        actors,
        actor_at,
		doors,
		statics,
		spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
		vislist: vec![VisObj{view_x: 0, view_height: 0, sprite: Sprite::None}; MAX_STATS],
		thrustspeed: 0,
		last_attacker: None,
	};

    thrust(PLAYER_KEY, &mut level_state, prj, 0, 0); // set some variables

	//TODO ambush markers
	Ok(level_state)
}

// By convention the first element in the returned actors vec is the player
fn scan_info_plane(map_data: &libiw::map::MapData, actor_at : &mut Vec<Vec<At>>, difficulty: Difficulty) -> (Vec<ObjType>, Vec<StaticType>) {
	let mut player = None;
	let mut statics = Vec::new();
	let mut actors = Vec::new();

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[1][map_ptr];
			map_ptr += 1;
			
			match tile {
				19..=22 => { // player start position
					player = Some(spawn_player(x, y, NORTH+(tile-19)as i32))
				}
				23..=74 => { // statics
					if statics.len() >= MAX_STATS {
						panic!("Too many static objects!")
					}
					statics.push(spawn_static(actor_at, x, y, (tile-23) as usize));

				},
				98 => { // P wall
					// TODO push wall
				},
				108..=111 => { // guard stand: normal mode
					spawn(&mut actors, actor_at, stand(EnemyType::Guard, x, y, tile-108, difficulty));
				},
				112..=115 => { // guard patrol: normal mode
				
				},
				116..=119 => { // officer stand: normal mode

				},
				120..=123 => { // officer patrol: normal mode

				},
				124 => { // guard: dead
					spawn(&mut actors, actor_at, dead_guard(x, y));
				},
				126..=129 => { // ss stand: normal mode

				},
				130..=133 => { // ss patrol: normal mode

				},
				134..=137 => { // dogs stand: normal mode

				},
				138..=141 => { // dogs patrol: normal mode

				},
				144..=147 => { // guard stand: medium mode
					if difficulty >= Difficulty::Medium {
						spawn(&mut actors, actor_at, stand(EnemyType::Guard, x, y, tile-144, difficulty));
					}
				},
				148..=151 => { // guard patrol: medium mode
				
				},
				152..=155 => { // officer stand: medium mode

				},
				156..=159 => { // officer patrol: medium mode

				},
				162..=165 => { // ss stand: medium mode

				},
				166..=169 => { // ss patrol: medium mode

				},
				170..=173 => { // dogs stand: medium mode

				},
				174..=177 => { // dogs patrol: medium mode

				},
				180..=183 => { // guard stand: hard mode
					if difficulty >= Difficulty::Hard {
						spawn(&mut actors, actor_at, stand(EnemyType::Guard, x, y, tile-180, difficulty));
					}
				},
				184..=187 => { // guard patrol: hard mode
	
				},
				188..=191 => { // officer stand: hard mode

				},
				192..=195 => { // officer patrol: hard mode

				},
				198..=201 => { // ss stand: hard mode

				},
				202..=205 => { // ss patrol: hard mode

				},
				206..=209 => { // dogs stand: hard mode

				},
				210..=213 => { // dogs patrol: hard mode

				}
				// TODO scan bosses, mutants and ghosts
				_ => {},
			}
		}
	}

	if player.is_none() {
		panic!("No player start position in map");
	}

	actors.insert(0, player.unwrap());

	(actors, statics)
}

// spawns the obj into the map
fn spawn(actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, obj: ObjType) {
	actors.push(obj);
	let key = ObjKey(actors.len()); // +1 offset (not len()-1), since player will be later at position 0 and positions will shift
	actor_at[obj.tilex][obj.tiley] = At::Obj(key)
}