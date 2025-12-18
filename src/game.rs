use std::vec;

use crate::act1::{spawn_door, spawn_static};
use crate::act2::{
    spawn_boss, spawn_dead_guard, spawn_fake_hitler, spawn_ghosts, spawn_gift, spawn_gretel,
    spawn_hitler, spawn_patrol, spawn_schabbs, spawn_stand,
};
use crate::agent::{
    DUMMY_PLAYER, draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_score,
    draw_weapon, spawn_player, thrust_player,
};
use crate::assets::{GraphicNum, SoundName, load_demo, load_map_from_assets};
use crate::config::WolfConfig;
use crate::def::{
    AMBUSH_TILE, ANGLES, ANGLES_F64, Actors, Assets, At, BenchmarkResult, ControlState, Difficulty,
    DoorLock, EnemyType, GameState, IWConfig, Level, LevelState, MAP_SIZE, MAX_ACTORS, MAX_DOORS,
    MAX_STATS, NUM_AREAS, ObjKey, PlayState, Sprite, StaticType, VisObj, WeaponType, WindowState,
    new_game_state,
};
use crate::draw::three_d_refresh;
use crate::inter::{check_highscore, level_completed, preload_graphics, victory};
use crate::loader::Loader;
use crate::map;
use crate::menu::MenuState;
use crate::play::{
    draw_play_screen, finish_palette_shifts, new_control_state, play_loop, start_music,
};
use crate::rc::{FizzleFadeAbortable, Input, InputMode, RenderContext};
use crate::user::{HighScore, init_rnd_t};
use crate::util::DataReader;
use crate::vh::vw_fade_out;

pub const AREATILE: u16 = 107;

pub const NORTH: i32 = 0;
pub const EAST: i32 = 0;
pub const SOUTH: i32 = 0;
pub const WEST: i32 = 0;

pub const ANGLE_45: u32 = 0x20000000;
pub const ANGLE_90: u32 = ANGLE_45 * 2;
pub const ANGLE_180: u32 = ANGLE_45 * 4;
pub const ANGLE_1: u32 = ANGLE_45 / 45;

pub const DEATH_ROTATE: u64 = 2;

// TODO Check whether indexes are correct
static ELEVATOR_BACK_TO: [usize; 6] = [1, 1, 7, 3, 4, 3];

pub async fn game_loop(
    rc: &mut RenderContext,
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) {
    let mut control_state: ControlState = new_control_state();

    draw_play_screen(rc, &game_state).await;

    let mut restart = false;
    'game_loop: loop {
        if restart {
            restart = false;
            draw_play_screen(rc, game_state).await;
            game_state.died = false;
        }

        if !game_state.loaded_game {
            game_state.score = game_state.old_score;
        }
        draw_score(rc, game_state);

        game_state.start_game = false;
        if game_state.loaded_game {
            game_state.loaded_game = false;
        } else {
            *level_state = setup_game_level(game_state, &rc.assets, false).unwrap();
        }

        win_state.in_game = true;

        start_music(rc, game_state, loader);

        if !game_state.died {
            preload_graphics(rc, iw_config, &game_state).await;
        } else {
            game_state.died = false;
        }

        game_state.fizzle_in = true;
        draw_level(rc, &game_state);

        rc.fade_in().await;

        let _ = play_loop(
            rc,
            wolf_config,
            iw_config,
            level_state,
            game_state,
            win_state,
            menu_state,
            &mut control_state,
            loader,
            false,
        )
        .await;

        win_state.in_game = false;

        if game_state.start_game || game_state.loaded_game {
            restart = true;
            continue;
        }

        match game_state.play_state {
            PlayState::Completed | PlayState::SecretLevel => {
                game_state.keys = 0;
                draw_keys(rc, &game_state);
                vw_fade_out(&mut rc.vga).await;

                level_completed(rc, game_state, win_state, loader).await;

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
            }
            PlayState::Died => {
                let player = level_state.player();
                rc.cast
                    .init_ray_cast_consts(&rc.projection, player, game_state.push_wall_pos);
                died(rc, level_state, game_state).await;
                if game_state.lives > -1 {
                    continue 'game_loop;
                }

                rc.fade_out().await;

                check_highscore(
                    rc,
                    win_state,
                    loader,
                    wolf_config,
                    new_high_score(game_state),
                )
                .await;

                menu_state.reset();
                return;
            }
            PlayState::Victorious => {
                rc.fade_out().await;

                victory(rc, game_state, win_state, loader).await;

                check_highscore(
                    rc,
                    win_state,
                    loader,
                    wolf_config,
                    new_high_score(game_state),
                )
                .await;

                menu_state.reset();
                return;
            }
            PlayState::Warped | PlayState::Abort | PlayState::ResetGame => {
                // do nothing and loop around the game loop
            }
            _ => panic!("not implemented end with state {:?}", game_state.play_state),
        }
    }
}

fn new_high_score(game_state: &GameState) -> HighScore {
    HighScore {
        name: "".to_string(),
        score: game_state.score,
        completed: game_state.map_on as u16 + 1,
        episode: game_state.episode as u16,
    }
}

async fn died(rc: &mut RenderContext, level_state: &mut LevelState, game_state: &mut GameState) {
    game_state.weapon = None; // take away weapon
    rc.play_sound(SoundName::PLAYERDEATH);

    let player = level_state.player();

    let (killer_x, killer_y) = if let Some(k) = game_state.killer_obj {
        let killer_obj = level_state.obj(k);
        (killer_obj.x, killer_obj.y)
    } else {
        // This happens if the player ends the game. The original
        // game uses random numbers in this game, but we just default this
        // to (0, 0) in iw.
        (0, 0)
    };

    // swing around to face attacker
    let dx = killer_x - player.x;
    let dy = player.y - killer_y;

    let mut fangle = (dy as f64).atan2(dx as f64);
    if fangle < 0.0 {
        fangle = std::f64::consts::PI * 2.0 + fangle;
    }
    let iangle = ((fangle / (std::f64::consts::PI * 2.0)) * ANGLES_F64) as i32;

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

            let tics = rc.ticker.wait_for_tic().await;
            let mut change = (tics * DEATH_ROTATE) as i32;
            if curangle + change > iangle {
                change = iangle - curangle;
            }
            curangle += change;

            let player = level_state.mut_player();
            player.angle += change;
            if player.angle >= ANGLES as i32 {
                player.angle -= ANGLES as i32;
            }
            three_d_refresh(
                rc,
                game_state,
                level_state,
                rc.input.mode == InputMode::DemoPlayback,
            )
            .await;
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

            let tics = rc.ticker.wait_for_tic().await;
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
            three_d_refresh(
                rc,
                game_state,
                level_state,
                rc.input.mode == InputMode::DemoPlayback,
            )
            .await;
        }
    }

    // fade to red
    finish_palette_shifts(game_state, &mut rc.vga);

    let source_buffer = rc.buffer_offset() + rc.projection.screenofs;
    rc.set_buffer_offset(source_buffer);
    // fill source buffer with all red screen for the fizzle_fade
    rc.bar(0, 0, rc.projection.view_width, rc.projection.view_height, 4);

    rc.clear_keys_down();
    rc.fizzle_fade(
        source_buffer,
        rc.active_buffer() + rc.projection.screenofs,
        rc.projection.view_width,
        rc.projection.view_height,
        70,
        FizzleFadeAbortable::No,
    );
    rc.set_buffer_offset(rc.buffer_offset() - rc.projection.screenofs);
    rc.wait_user_input(100).await;
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

        draw_keys(rc, game_state);
        draw_weapon(rc, game_state);
        draw_ammo(rc, game_state);
        draw_health(rc, game_state);
        draw_face(rc, game_state);
        draw_lives(rc, game_state);
    }
}

pub fn setup_game_level(
    game_state: &mut GameState,
    assets: &Assets,
    demo_playback: bool,
) -> Result<LevelState, String> {
    if !game_state.loaded_game {
        game_state.time_count = 0;
        game_state.secret_total = 0;
        game_state.kill_total = 0;
        game_state.treasure_total = 0;
        game_state.secret_count = 0;
        game_state.kill_count = 0;
        game_state.treasure_count = 0;
    }

    if demo_playback {
        init_rnd_t(false);
    } else {
        init_rnd_t(true);
    }

    let mapnum = game_state.map_on + game_state.episode * 10;

    let map = &assets.map_headers[mapnum];
    if map.width != MAP_SIZE as u16 || map.height != MAP_SIZE as u16 {
        return Err("Map not 64*64!".to_string());
    }

    let mut map_segs = load_map_from_assets(assets, mapnum)?;

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
                    90 | 92 | 94 | 96 | 98 | 100 => spawn_door(
                        &mut tile_map,
                        &mut map_segs,
                        &mut actor_at,
                        doornum,
                        x,
                        y,
                        true,
                        door_lock((tile - 90) / 2),
                    ),
                    91 | 93 | 95 | 97 | 99 | 101 => spawn_door(
                        &mut tile_map,
                        &mut map_segs,
                        &mut actor_at,
                        doornum,
                        x,
                        y,
                        false,
                        door_lock((tile - 91) / 2),
                    ),
                    _ => unreachable!("tile guaranteed to be in range through the if check"),
                };
                doors.push(door);
                doornum += 1;
            }
        }
    }

    let mut area_by_player = vec![false; NUM_AREAS];

    let (actors, statics, info_map) = scan_info_plane(
        &mut tile_map,
        &mut map_segs,
        game_state,
        &mut actor_at,
        &mut area_by_player,
        game_state.difficulty,
    );

    // take out the ambush markers
    map_ptr = 0;
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let mut tile = map_segs.segs[0][map_ptr];
            map_ptr += 1;

            if tile == AMBUSH_TILE {
                tile_map[x][y] = 0;
                if let At::Wall(tile) = actor_at[x][y] {
                    if tile == AMBUSH_TILE {
                        actor_at[x][y] = At::Nothing;
                    }
                }

                let map = map_segs.segs[0][map_ptr];
                if map >= AREATILE {
                    tile = map;
                }
                if map_segs.segs[0][map_ptr - 1 - MAP_SIZE] >= AREATILE {
                    tile = map_segs.segs[0][map_ptr - 1 - MAP_SIZE];
                }
                if map_segs.segs[0][map_ptr - 1 + MAP_SIZE] >= AREATILE {
                    tile = map_segs.segs[0][map_ptr - 1 + MAP_SIZE];
                }
                if map_segs.segs[0][map_ptr - 2] >= AREATILE {
                    tile = map_segs.segs[0][map_ptr - 2]
                }

                map_segs.segs[0][map_ptr - 1] = tile;
            }
        }
    }

    let mut level_state = LevelState {
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
        area_by_player,
        statics,
        spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
        vislist: vec![
            VisObj {
                view_x: 0,
                view_height: 0,
                sprite: Sprite::None
            };
            MAX_STATS
        ],
        thrustspeed: 0,
        last_attacker: None,
    };

    thrust_player(&mut level_state); // set some variables

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
fn scan_info_plane(
    tile_map: &mut Vec<Vec<u16>>,
    map_data: &mut map::MapSegs,
    game_state: &mut GameState,
    actor_at: &mut Vec<Vec<At>>,
    area_by_player: &mut Vec<bool>,
    difficulty: Difficulty,
) -> (Actors, Vec<StaticType>, Vec<Vec<u16>>) {
    let mut player = None;
    let mut statics = Vec::new();
    let mut actors = Actors::new(MAX_ACTORS);
    let player_key = actors.add_obj(DUMMY_PLAYER); //dummy player as a placeholder!
    if player_key != ObjKey(0) {
        // make sure player gets the 0 spot
        panic!("player not at position 0")
    }

    let mut info_plane = vec![vec![0; MAP_SIZE]; MAP_SIZE];

    let mut map_ptr = 0;
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile = map_data.segs[1][map_ptr];
            map_ptr += 1;

            info_plane[x][y] = tile;

            match tile {
                19..=22 => {
                    // player start position
                    player = Some(spawn_player(
                        x,
                        y,
                        map_data,
                        area_by_player,
                        NORTH + (tile - 19) as i32,
                    ))
                }
                23..=74 => {
                    // statics
                    if statics.len() >= MAX_STATS {
                        panic!("Too many static objects!")
                    }
                    statics.push(spawn_static(
                        actor_at,
                        game_state,
                        x,
                        y,
                        (tile - 23) as usize,
                    ));
                }
                98 => {
                    // P wall
                    if !game_state.loaded_game {
                        game_state.secret_total += 1;
                    }
                }
                108..=111 => {
                    // guard stand: normal mode
                    spawn_stand(
                        tile_map,
                        map_data,
                        EnemyType::Guard,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 108,
                        difficulty,
                    );
                }
                112..=115 => {
                    // guard patrol: normal mode
                    spawn_patrol(
                        map_data,
                        EnemyType::Guard,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 112,
                        difficulty,
                    );
                }
                116..=119 => {
                    // officer stand: normal mode
                    spawn_stand(
                        tile_map,
                        map_data,
                        EnemyType::Officer,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 116,
                        difficulty,
                    );
                }
                120..=123 => {
                    // officer patrol: normal mode
                    spawn_patrol(
                        map_data,
                        EnemyType::Officer,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 120,
                        difficulty,
                    );
                }
                124 => {
                    // guard: dead
                    spawn_dead_guard(map_data, &mut actors, actor_at, x, y);
                }
                125 => {
                    todo!("trans");
                }
                126..=129 => {
                    // ss stand: normal mode
                    spawn_stand(
                        tile_map,
                        map_data,
                        EnemyType::SS,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 126,
                        difficulty,
                    );
                }
                130..=133 => {
                    // ss patrol: normal mode
                    spawn_patrol(
                        map_data,
                        EnemyType::SS,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 130,
                        difficulty,
                    );
                }
                134..=137 => {
                    // dogs stand: normal mode
                    spawn_stand(
                        tile_map,
                        map_data,
                        EnemyType::Dog,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 134,
                        difficulty,
                    );
                }
                138..=141 => {
                    // dogs patrol: normal mode
                    spawn_patrol(
                        map_data,
                        EnemyType::Dog,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 138,
                        difficulty,
                    );
                }
                142 => {
                    todo!("uber");
                }
                143 => {
                    todo!("will");
                }
                144..=147 => {
                    // guard stand: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Guard,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 144,
                            difficulty,
                        );
                    }
                }
                148..=151 => {
                    // guard patrol: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_patrol(
                            map_data,
                            EnemyType::Guard,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 148,
                            difficulty,
                        );
                    }
                }
                152..=155 => {
                    if difficulty >= Difficulty::Medium {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Officer,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 152,
                            difficulty,
                        );
                    }
                }
                156..=159 => {
                    // officer patrol: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_patrol(
                            map_data,
                            EnemyType::Officer,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 156,
                            difficulty,
                        );
                    }
                }
                160 => {
                    spawn_fake_hitler(map_data, &mut actors, actor_at, game_state, x, y);
                }
                161 => {
                    todo!("death");
                }
                162..=165 => {
                    // ss stand: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::SS,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 162,
                            difficulty,
                        );
                    }
                }
                166..=169 => {
                    // ss patrol: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_patrol(
                            map_data,
                            EnemyType::SS,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 166,
                            difficulty,
                        );
                    }
                }
                170..=173 => {
                    // dogs stand: medium mode
                    if difficulty >= Difficulty::Medium {
                        todo!("spawn dog medium");
                    }
                }
                174..=177 => {
                    // dogs patrol: medium mode
                    if difficulty >= Difficulty::Medium {
                        spawn_patrol(
                            map_data,
                            EnemyType::Dog,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 174,
                            difficulty,
                        );
                    }
                }
                178 => {
                    spawn_hitler(map_data, &mut actors, actor_at, game_state, x, y);
                }
                179 => {
                    todo!("fat");
                }
                180..=183 => {
                    // guard stand: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Guard,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 180,
                            difficulty,
                        );
                    }
                }
                184..=187 => {
                    // guard patrol: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_patrol(
                            map_data,
                            EnemyType::Guard,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 184,
                            difficulty,
                        );
                    }
                }
                188..=191 => {
                    if difficulty >= Difficulty::Hard {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Officer,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 188,
                            difficulty,
                        );
                    }
                }
                192..=195 => {
                    // officer patrol: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_patrol(
                            map_data,
                            EnemyType::Officer,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 192,
                            difficulty,
                        );
                    }
                }
                196 => {
                    spawn_schabbs(map_data, &mut actors, actor_at, game_state, x, y);
                }
                197 => {
                    spawn_gretel(map_data, &mut actors, actor_at, game_state, x, y);
                }
                198..=201 => {
                    // ss stand: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::SS,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 198,
                            difficulty,
                        )
                    }
                }
                202..=205 => {
                    // ss patrol: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_patrol(
                            map_data,
                            EnemyType::SS,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 202,
                            difficulty,
                        );
                    }
                }
                206..=209 => {
                    // dogs stand: hard mode
                    if difficulty >= Difficulty::Hard {
                        todo!("spawn dog hard");
                    }
                }
                210..=213 => {
                    // dogs patrol: hard mode
                    if difficulty >= Difficulty::Hard {
                        spawn_patrol(
                            map_data,
                            EnemyType::Dog,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 210,
                            difficulty,
                        );
                    }
                }
                214 => {
                    spawn_boss(map_data, &mut actors, actor_at, game_state, x, y);
                }
                215 => {
                    spawn_gift(map_data, &mut actors, actor_at, game_state, x, y);
                }
                216..=219 => {
                    spawn_stand(
                        tile_map,
                        map_data,
                        EnemyType::Mutant,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 216,
                        difficulty,
                    );
                }
                220..=223 => {
                    spawn_patrol(
                        map_data,
                        EnemyType::Mutant,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                        tile - 220,
                        difficulty,
                    );
                }
                224 => {
                    spawn_ghosts(
                        map_data,
                        EnemyType::Blinky,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                    );
                }
                225 => {
                    spawn_ghosts(
                        map_data,
                        EnemyType::Clyde,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                    );
                }
                226 => {
                    spawn_ghosts(
                        map_data,
                        EnemyType::Pinky,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                    );
                }
                227 => {
                    spawn_ghosts(
                        map_data,
                        EnemyType::Inky,
                        &mut actors,
                        actor_at,
                        game_state,
                        x,
                        y,
                    );
                }
                // nothing on 228 to 233
                234..=237 => {
                    if difficulty >= Difficulty::Medium {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Mutant,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 234,
                            difficulty,
                        );
                    }
                }
                238..=241 => {
                    if difficulty >= Difficulty::Medium {
                        spawn_patrol(
                            map_data,
                            EnemyType::Mutant,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 238,
                            difficulty,
                        );
                    }
                }
                //nothing on 242 to 251
                252..=255 => {
                    if difficulty >= Difficulty::Hard {
                        spawn_stand(
                            tile_map,
                            map_data,
                            EnemyType::Mutant,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 252,
                            difficulty,
                        );
                    }
                }
                256..=259 => {
                    if difficulty >= Difficulty::Hard {
                        spawn_patrol(
                            map_data,
                            EnemyType::Mutant,
                            &mut actors,
                            actor_at,
                            game_state,
                            x,
                            y,
                            tile - 256,
                            difficulty,
                        );
                    }
                }
                _ => {
                    // nothing to do here
                }
            }
        }
    }

    let player = player.expect("No player start position in map");
    actors.put_obj(player_key, player);

    (actors, statics, info_plane)
}

// Fades the screen out, then starts a demo.  Exits with the screen faded
pub async fn play_demo(
    rc: &mut RenderContext,
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
    demo_num: usize,
    benchmark: bool,
) -> (bool, Option<BenchmarkResult>) {
    let demo_data = load_demo(loader, demo_graphic_num(demo_num)).expect("demo load");
    let mut demo_reader = DataReader::new(&demo_data);
    let mut game_state = new_game_state();
    game_state.map_on = demo_reader.read_u8() as usize;
    game_state.difficulty = Difficulty::Hard;
    demo_reader.skip(3); // length not needed (in Vec len)

    let demo_input = Input::init_demo_playback(demo_reader.unread_bytes().to_vec());
    rc.use_demo_input(demo_input);

    rc.fade_out().await;
    win_state.set_font_color(0, 15);
    draw_play_screen(rc, &game_state).await;
    rc.fade_in().await;

    let mut level_state =
        setup_game_level(&mut game_state, &rc.assets, true).expect("setup game level");
    start_music(rc, &mut game_state, loader);

    game_state.fizzle_in = true;
    let mut control_state = new_control_state();
    let benchmark_result = play_loop(
        rc,
        wolf_config,
        iw_config,
        &mut level_state,
        &mut game_state,
        win_state,
        menu_state,
        &mut control_state,
        loader,
        benchmark,
    )
    .await;

    rc.restore_player_input().expect("player input restored");

    (game_state.play_state == PlayState::Abort, benchmark_result)
}

fn demo_graphic_num(demo_num: usize) -> GraphicNum {
    match demo_num {
        0 => GraphicNum::DEMO0,
        1 => GraphicNum::DEMO1,
        2 => GraphicNum::DEMO2,
        3 => GraphicNum::DEMO3,
        _ => GraphicNum::DEMO3,
    }
}
