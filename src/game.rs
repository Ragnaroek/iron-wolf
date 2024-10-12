use vga::VGA;

use crate::agent::{draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_weapon};
use crate::def::{ANGLES, MAX_STATS, MAX_DOORS, MAP_SIZE, PLAYER_KEY, PlayState, WeaponType, Sprite, StaticType, VisObj, ObjKey, Assets, ObjType, Level, LevelState, At, EnemyType, GameState, Difficulty, ControlState, AMBUSH_TILE, UserState};
use crate::assets::load_map_from_assets;
use crate::act1::{spawn_door, spawn_static};
use crate::act2::{dead_guard, stand};
use crate::agent::{spawn_player, thrust};
use crate::draw::{RayCast, init_ray_cast, three_d_refresh};
use crate::inter::check_highscore;
use crate::play::{draw_play_screen, finish_palette_shifts, play_loop, ProjectionConfig, new_game_state, new_control_state};
use crate::{map, time};
use crate::vga_render::VGARenderer;
use crate::input::Input;

pub const AREATILE : u16 = 107;

pub const NORTH : i32 = 0;
pub const EAST : i32 = 0;
pub const SOUTH : i32 = 0;
pub const WEST : i32 = 0;

pub const ANGLE_45 : u32 = 0x20000000;
pub const ANGLE_90 : u32 = ANGLE_45*2;
pub const ANGLE_180 : u32 = ANGLE_45*4;
pub const ANGLE_1 : u32 = ANGLE_45/45;

pub const DEATH_ROTATE : u64 = 2;

pub async fn game_loop(ticker: &time::Ticker, vga: &VGA, rdr: &VGARenderer, input: &Input, prj: &ProjectionConfig, assets: &Assets, user_state: &mut UserState) {
    let mut game_state = new_game_state();
    let mut control_state : ControlState = new_control_state();
    
    draw_play_screen(&game_state, rdr, prj).await;

	'game_loop:
	loop {
		let mut level_state = setup_game_level(prj, &game_state, assets).unwrap();
		let mut rc = init_ray_cast(prj.view_width);

		//TODO StartMusic
		//TODO PreloadGraphics
		
		game_state.fizzle_in = true;
		draw_level(&game_state, rdr);
		
		rdr.fade_in().await;

		play_loop(ticker, &mut level_state, &mut game_state, user_state, &mut control_state, vga, &mut rc, rdr, input, prj, assets).await;

		match game_state.play_state {
			PlayState::Died => {
				died(ticker, &mut level_state, &mut game_state, &mut rc, rdr, prj, input, assets).await;
				if game_state.lives > -1 {
					continue 'game_loop;
				}

				rdr.fade_out().await;
				
				check_highscore(rdr, input, game_state.score, game_state.map_on+1).await;

				return;
			},
			_ => panic!("not implemented end with state {:?}", game_state.play_state)
		}
	}
	//TODO Go to next level (gamestate.map_on+=1)
}

async fn died(ticker: &time::Ticker, level_state: &mut LevelState, game_state: &mut GameState, rc: &mut RayCast, rdr: &VGARenderer, prj: &ProjectionConfig, input: &Input, assets: &Assets) {
	game_state.weapon = WeaponType::None; // take away weapon
	//TODO SD_PlaySound(PLAYERDEATHSND)

	let player = level_state.player();
	let killer_obj = level_state.obj(game_state.killer_obj.expect("killer obj key be present"));

	// swing around to face attacker
	let dx = killer_obj.x - player.x;
	let dy = player.y - killer_obj.y; 

	let mut fangle = (dy as f64).atan2(dx as f64);
	if fangle < 0.0 {
		fangle = std::f64::consts::PI * 2.0 + fangle;
	}
	let iangle = (fangle/(std::f64::consts::PI * 2.0)) as i32 * ANGLES as i32;

	let counter;
	let clockwise;
	if player.angle > iangle {
		counter = player.angle - iangle;
		clockwise = ANGLES as i32 - player.angle + iangle;
	} else {
		clockwise = iangle - player.angle;
		counter = player.angle + ANGLES as i32 - iangle;
	}

	let mut curangle = player.angle;

	if clockwise < counter {
		// rotate clockwise

		if curangle > iangle {
			curangle -= ANGLES as i32;
		}
		loop {
			if curangle == iangle {
				break;
			}

			let tics = ticker.calc_tics();
			let mut change = (tics*DEATH_ROTATE) as i32;
			if curangle + change > iangle {
				change = iangle - curangle;
			}
			curangle += change;

			let player = level_state.mut_player();
			player.angle += change;
			if player.angle >= ANGLES as i32 {
				player.angle -= ANGLES as i32;
			}
			three_d_refresh(ticker, game_state, level_state, rc, rdr, prj, assets).await;
		}
	} else {
		// rotate counterclockwise
		if curangle < iangle {
			curangle += ANGLES as i32;
		}
		loop {
			if curangle == iangle {
				break;
			}

			let tics = ticker.calc_tics();
			let mut change = -((tics * DEATH_ROTATE) as i32);
			if curangle + change < iangle {
				change = iangle - curangle;
			}

			curangle += change;
			let player = level_state.mut_player();
			player.angle += change;
			if player.angle < 0 {
				player.angle += ANGLES as i32;
			}
			three_d_refresh(ticker, game_state, level_state, rc, rdr, prj, assets).await;
		}
	}

	// fade to red
	finish_palette_shifts(game_state, &rdr.vga).await;

	let source_buffer = rdr.buffer_offset()+prj.screenofs;
	rdr.set_buffer_offset(source_buffer);
	// fill source buffer with all red screen for the fizzle_fade
	rdr.bar(0, 0, prj.view_width, prj.view_height, 4);
	
	input.clear_keys_down();
	rdr.fizzle_fade(ticker, source_buffer, rdr.active_buffer()+prj.screenofs, prj.view_width, prj.view_height, 70, false).await;
	rdr.set_buffer_offset(rdr.buffer_offset()-prj.screenofs);
	input.wait_user_input(100).await;
	//TODO SD_WaitSoundDone

	// TODO editor support here (tedlevel)
	game_state.lives -= 1;

	if game_state.lives > -1 {
		game_state.health = 100;
		game_state.weapon = WeaponType::Pistol;
		game_state.best_weapon = WeaponType::Pistol;
		game_state.chosen_weapon = WeaponType::Pistol;
		game_state.keys = 0;
		game_state.attack_frame = 0;
		game_state.attack_count = 0;
		game_state.weapon_frame = 0;

		draw_keys(game_state, rdr);
		draw_weapon(game_state, rdr);
		draw_ammo(game_state, rdr);
		draw_health(game_state, rdr);
		draw_face(game_state, rdr);
		draw_lives(game_state, rdr);
	}
}

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

	let (actors, statics, info_map) = scan_info_plane(&map_data, &mut actor_at, game_state.difficulty);

	// take out the ambush markers
	map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[0][map_ptr];
			map_ptr += 1;

			if tile == AMBUSH_TILE {
				tile_map[x][y] = 0;
				if let At::Wall(tile) = actor_at[x][y] {
					if tile == AMBUSH_TILE {
						actor_at[x][y] = At::Nothing;
					}
				}

				// TODO something with AREATILEs has to happen here
			}
		}
	}
	
	let mut level_state = LevelState{
        level: Level {
			info_map,
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

	Ok(level_state)
}

// By convention the first element in the returned actors vec is the player
fn scan_info_plane(map_data: &map::MapData, actor_at : &mut Vec<Vec<At>>, difficulty: Difficulty) -> (Vec<ObjType>, Vec<StaticType>, Vec<Vec<u16>>) {
	let mut player = None;
	let mut statics = Vec::new();
	let mut actors = Vec::new();
	let mut info_plane = vec![vec![0; MAP_SIZE]; MAP_SIZE];

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_data.segs[1][map_ptr];
			map_ptr += 1;

			info_plane[x][y] = tile;

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

	(actors, statics, info_plane)
}

// spawns the obj into the map
pub fn spawn(actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, obj: ObjType) {
	actors.push(obj);
	let key = ObjKey(actors.len()); // +1 offset (not len()-1), since player will be later at position 0 and positions will shift
	actor_at[obj.tilex][obj.tiley] = At::Obj(key)
}