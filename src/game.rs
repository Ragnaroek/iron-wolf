use std::vec;

use vga::VGA;

use crate::agent::{draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_weapon};
use crate::def::{Assets, At, ControlState, Difficulty, DoorLock, EnemyType, GameState, IWConfig, Level, LevelState, ObjType, PlayState, Sprite, StaticType, VisObj, WeaponType, WindowState, AMBUSH_TILE, ANGLES, MAP_SIZE, MAX_DOORS, MAX_STATS, NUM_AREAS, PLAYER_KEY};
use crate::assets::load_map_from_assets;
use crate::act1::{spawn_door, spawn_static};
use crate::act2::{spawn_dead_guard, spawn_patrol, spawn_stand};
use crate::agent::{spawn_player, thrust};
use crate::draw::{RayCast, init_ray_cast, three_d_refresh};
use crate::inter::{check_highscore, level_completed, preload_graphics};
use crate::loader::Loader;
use crate::menu::{MenuState, SaveLoadGame};
use crate::play::{draw_play_screen, finish_palette_shifts, play_loop, ProjectionConfig, new_control_state};
use crate::vh::vw_fade_out;
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

static ELEVATOR_BACK_TO : [usize; 6]= [1, 1, 7, 3, 5, 3];

pub async fn game_loop(
	ticker: &time::Ticker,
	iw_config: &IWConfig,
	game_state: &mut GameState,
	vga: &VGA,
	rdr: &VGARenderer,
	input: &Input,
	prj: &ProjectionConfig,
	assets: &Assets,
	win_state: &mut WindowState,
	menu_state: &mut MenuState,
	loader: &dyn Loader,
	save_load_param: Option<SaveLoadGame>) {
	let mut save_load = save_load_param;
    let mut control_state : ControlState = new_control_state();
    
    draw_play_screen(&game_state, rdr, prj).await;

	'game_loop:
	loop {
		let mut level_state = setup_game_level(prj, game_state, assets).unwrap();
		let mut rc = init_ray_cast(prj.view_width);

		game_state.in_game = true;

		//TODO StartMusic
		//TODO PreloadGraphics

		if !game_state.died {
			preload_graphics(ticker, iw_config, &game_state, prj, input, rdr).await;
		} else {
			game_state.died = false;
		}

		game_state.fizzle_in = true;
		draw_level(&game_state, rdr);
		
		rdr.fade_in().await;

		play_loop(ticker, &mut level_state, game_state, win_state, menu_state, &mut control_state, vga, &mut rc, rdr, input, prj, assets, loader, save_load).await;
		save_load = None;

		game_state.in_game = false;

		match game_state.play_state {
			PlayState::Completed|PlayState::SecretLevel => {
				game_state.keys = 0;
				draw_keys(&game_state, rdr);
				vw_fade_out(vga).await;

				level_completed(ticker, rdr, input, game_state, prj, win_state).await;

				game_state.old_score = game_state.score;

				// COMING BACK FROM SECRET LEVEL
				if game_state.map_on == 9 {
					game_state.map_on = ELEVATOR_BACK_TO[game_state.episode]; // back from secret
				}
				// GOING TO SECRET LEVEL
				if game_state.play_state == PlayState::SecretLevel {
					game_state.map_on = 9;
				} else {
					// GOING TO NEXT LEVEL
					game_state.map_on += 1;
				}
			},
			PlayState::Died => {
				died(ticker, &mut level_state, game_state, &mut rc, rdr, prj, input, assets).await;
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
}

async fn died(ticker: &time::Ticker, level_state: &mut LevelState, game_state: &mut GameState, rc: &mut RayCast, rdr: &VGARenderer, prj: &ProjectionConfig, input: &Input, assets: &Assets) {
	game_state.weapon = None; // take away weapon
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
		game_state.weapon = Some(WeaponType::Pistol);
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

pub fn setup_game_level(prj: &ProjectionConfig, game_state: &mut GameState, assets: &Assets) -> Result<LevelState, String> {
	if !game_state.loaded_game {
		game_state.time_count = 0;
		game_state.secret_total = 0;
		game_state.kill_total = 0;
		game_state.treasure_total = 0;
		game_state.secret_count = 0;
		game_state.kill_count = 0;
		game_state.treasure_count = 0;
	}
	
	let mapnum = game_state.map_on+game_state.episode*10;
	
	let map = &assets.map_headers[mapnum];
	if map.width != MAP_SIZE as u16 || map.height != MAP_SIZE as u16 {
		return Err("Map not 64*64!".to_string());
	}

	let map_segs = load_map_from_assets(assets, mapnum)?;

    let mut tile_map = vec![vec![0; MAP_SIZE]; MAP_SIZE];
    let mut actor_at = vec![vec![At::Nothing; MAP_SIZE]; MAP_SIZE];

	let mut map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_segs.segs[0][map_ptr];
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
			let tile = map_segs.segs[0][map_ptr];
			map_ptr += 1;
			if tile >= 90 && tile <= 101 {
				let door = match tile {
					90 | 92 | 94 | 96 | 98 | 100 => spawn_door(&mut tile_map, doornum, x, y, true, door_lock((tile-90)/2)),
					91 | 93 | 95 | 97 | 99 | 101 => spawn_door(&mut tile_map, doornum, x, y, false, door_lock((tile-91)/2)),
					_ => unreachable!("tile guaranteed to be in range through the if check")
				};
				doors.push(door);
				doornum += 1;
			}
		}
	}

	let (actors, statics, info_map) = scan_info_plane(&map_segs, game_state, &mut actor_at, game_state.difficulty);

	// take out the ambush markers
	map_ptr = 0;
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = map_segs.segs[0][map_ptr];
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
			map_segs,
			info_map,
		    tile_map,
        },
		map_width: map.width as usize,
        actors,
        actor_at,
		doors,
		area_connect: vec![vec![0; NUM_AREAS]; NUM_AREAS],
		area_by_player: vec![false; NUM_AREAS],
		statics,
		spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
		vislist: vec![VisObj{view_x: 0, view_height: 0, sprite: Sprite::None}; MAX_STATS],
		thrustspeed: 0,
		last_attacker: None,
	};

    thrust(PLAYER_KEY, &mut level_state, prj, 0, 0); // set some variables

	Ok(level_state)
}

fn door_lock(tile: u16) -> DoorLock {
	match tile {
		0 => DoorLock::Normal,
		1 => DoorLock::Lock1,
		2 => DoorLock::Lock2,
		3 => DoorLock::Lock3,
		4 => DoorLock::Lock4,
		5 => DoorLock::Elevator,
		_ => panic!("illegal door lock: {}", tile),
	}
}

// By convention the first element in the returned actors vec is the player
fn scan_info_plane(map_data: &map::MapSegs, game_state: &mut GameState, actor_at : &mut Vec<Vec<At>>, difficulty: Difficulty) -> (Vec<ObjType>, Vec<StaticType>, Vec<Vec<u16>>) {
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
					statics.push(spawn_static(actor_at, game_state, x, y, (tile-23) as usize));

				},
				98 => { // P wall
					// TODO check for loadedgame?
					game_state.secret_total += 1;
				},
				108..=111 => { // guard stand: normal mode
					spawn_stand(EnemyType::Guard, &mut actors, actor_at, x, y, tile-108, difficulty);
				},
				112..=115 => { // guard patrol: normal mode
					spawn_patrol(EnemyType::Guard, &mut actors, actor_at, game_state, x, y, tile-112, difficulty);
				},
				116..=119 => { // officer stand: normal mode
					todo!("officer stand");
				},
				120..=123 => { // officer patrol: normal mode
					todo!("office patrol");
				},
				124 => { // guard: dead
					spawn_dead_guard(&mut actors, actor_at, x, y);
				},
				125 => {
					todo!("trans");
				},
				126..=129 => { // ss stand: normal mode
					spawn_stand(EnemyType::SS, &mut actors, actor_at, x, y, tile-126, difficulty);
				},
				130..=133 => { // ss patrol: normal mode
					spawn_patrol(EnemyType::SS, &mut actors, actor_at, game_state, x, y, tile-130, difficulty);
				},
				134..=137 => { // dogs stand: normal mode
					spawn_stand(EnemyType::Dog, &mut actors, actor_at, x, y, tile-134, difficulty);
				},
				138..=141 => { // dogs patrol: normal mode
					spawn_patrol(EnemyType::Dog, &mut actors, actor_at, game_state, x, y, tile-138, difficulty);
				},
				142 => {
					todo!("uber");
				},
				143 => {
					todo!("will");
				},
				144..=147 => { // guard stand: medium mode
					if difficulty >= Difficulty::Medium {
						spawn_stand(EnemyType::Guard, &mut actors, actor_at, x, y, tile-144, difficulty);
					}
				},
				148..=151 => { // guard patrol: medium mode
					if difficulty >= Difficulty::Medium {
						spawn_patrol(EnemyType::Guard, &mut actors, actor_at, game_state, x, y, tile-148, difficulty);
					}
				},
				152..=155 => { // officer stand: medium mode
					todo!("officer stand");
				},
				156..=159 => { // officer patrol: medium mode
					todo!("officer patrol");
				},
				160 => {
					todo!("fake hitler");
				},
				161 => {
					todo!("death");
				},
				162..=165 => { // ss stand: medium mode
					if difficulty >= Difficulty::Medium {
						spawn_stand(EnemyType::SS, &mut actors, actor_at, x, y, tile-162, difficulty);
					}
				},
				166..=169 => { // ss patrol: medium mode
					if difficulty >= Difficulty::Medium {
						spawn_patrol(EnemyType::SS, &mut actors, actor_at, game_state, x, y, tile-166, difficulty);
					}
				},
				170..=173 => { // dogs stand: medium mode
					if difficulty >= Difficulty::Medium {
						todo!("spawn dog medium");
					}
				},
				174..=177 => { // dogs patrol: medium mode
					if difficulty >= Difficulty::Medium {
						spawn_patrol(EnemyType::Dog, &mut actors, actor_at, game_state, x, y, tile-174, difficulty);
					}
				},
				178 => {
					todo!("hitler");
				},
				179 => {
					todo!("fat");
				},
				180..=183 => { // guard stand: hard mode
					if difficulty >= Difficulty::Hard {
						spawn_stand(EnemyType::Guard, &mut actors, actor_at, x, y, tile-180, difficulty);
					}
				},
				184..=187 => { // guard patrol: hard mode
					if difficulty >= Difficulty::Hard {
						spawn_patrol(EnemyType::Guard, &mut actors, actor_at, game_state, x, y, tile-184, difficulty);
					}
				},
				188..=191 => { // officer stand: hard mode
					todo!("officer stand");
				},
				192..=195 => { // officer patrol: hard mode
					todo!("officer patrol");
				},
				196 => {
					todo!("schabbs");
				},
				197 => {
					todo!("gretel");
				},
				198..=201 => { // ss stand: hard mode
					if difficulty >= Difficulty::Hard {
						spawn_stand(EnemyType::SS, &mut actors, actor_at, x, y, tile-198, difficulty)
					}
				},
				202..=205 => { // ss patrol: hard mode
					if difficulty >= Difficulty::Hard {
						spawn_patrol(EnemyType::SS, &mut actors, actor_at, game_state, x, y, tile-202, difficulty);
					}
				},
				206..=209 => { // dogs stand: hard mode
					if difficulty >= Difficulty::Hard {
						todo!("spawn dog hard");
					}
				},
				210..=213 => { // dogs patrol: hard mode
					if difficulty >= Difficulty::Hard {
						spawn_patrol(EnemyType::Dog, &mut actors, actor_at, game_state, x, y, tile-210, difficulty);
					}
				}
				214 => {
					todo!("boss");
				},
				215 => {
					todo!("gift");
				},
				216..=219 => {
					todo!("stand mutant");
				}
				220..=223 => {
					todo!("patrol mutant");
				},
				224 => {
					todo!("ghost blinky");
				},
				225 => {
					todo!("ghost clyde");
				},
				226 => {
					todo!("ghost pinky");
				},
				227 => {
					todo!("ghost inky");
				},
				// nothing on 228 to 233
				234..=237 => {
					todo!("mutant");
				},
				238..=241 => {
					todo!("mutant");
				},
				//nothing on 242 to 251
				252..=255 => {
					todo!("mutant");
				},
				256..=259 => {
					todo!("mutant");
				}
				_ => {
					// nothing to do here
				},
			}
		}
	}

	if player.is_none() {
		panic!("No player start position in map");
	}

	actors.insert(0, player.unwrap());

	(actors, statics, info_plane)
}