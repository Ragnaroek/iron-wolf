#[cfg(test)]
#[path = "./state_test.rs"]
mod state_test;

use crate::act1::{open_door, place_item_type};
use crate::act2::{
    S_BOSSCHASE1, S_BOSSDIE1, S_DOGCHASE1, S_DOGDIE1, S_GRDCHASE1, S_GRDDIE1, S_GRDPAIN,
    S_GRDPAIN1, S_MUTCHASE1, S_MUTDIE1, S_MUTPAIN, S_MUTPAIN1, S_SSCHASE1, S_SSDIE1, S_SSPAIN,
    S_SSPAIN1,
};
use crate::agent::{give_points, take_damage};
use crate::assets::SoundName;
use crate::def::{
    Assets, At, ClassType, DirType, GameState, LevelState, ObjKey, ObjType, StateType, StaticKind,
    WeaponType, FL_AMBUSH, FL_ATTACKMODE, FL_FIRSTATTACK, FL_NONMARK, FL_SHOOTABLE, MAP_SIZE,
    MIN_ACTOR_DIST, SPD_PATROL, TILEGLOBAL, TILESHIFT, UNSIGNEDSHIFT,
};
use crate::fixed::new_fixed_i32;
use crate::game::AREATILE;
use crate::map::MapSegs;
use crate::sd::Sound;
use crate::user::rnd_t;
use crate::vga_render::VGARenderer;

static OPPOSITE: [DirType; 9] = [
    DirType::West,
    DirType::SouthWest,
    DirType::South,
    DirType::SouthEast,
    DirType::East,
    DirType::NorthEast,
    DirType::North,
    DirType::NorthWest,
    DirType::NoDir,
];

static DIAGONAL: [[DirType; 9]; 9] = [
    /* east */
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NorthEast,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::SouthEast,
        DirType::NoDir,
        DirType::NoDir,
    ],
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    /* north */
    [
        DirType::NorthEast,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NorthWest,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    /* west */
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NorthWest,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::SouthWest,
        DirType::NoDir,
        DirType::NoDir,
    ],
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    /* south */
    [
        DirType::SouthEast,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::SouthWest,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
    [
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
        DirType::NoDir,
    ],
];

pub const MIN_SIGHT: i32 = 0x18000;

pub fn spawn_new_obj(
    map_data: &MapSegs,
    tile_x: usize,
    tile_y: usize,
    state: &'static StateType,
    class: ClassType,
) -> ObjType {
    let tic_count = if state.tic_time > 0 {
        rnd_t() as u32 % state.tic_time
    } else {
        0
    };

    let tile = map_data.segs[0][tile_y * MAP_SIZE + tile_x];
    let area_number = if tile >= AREATILE {
        (tile - AREATILE) as usize
    } else {
        0
    };

    ObjType {
        class,
        active: crate::def::ActiveType::Yes,
        tic_count,
        distance: 0,
        area_number,
        flags: 0,
        angle: 0,
        pitch: 0,
        tilex: tile_x,
        tiley: tile_y,
        view_x: 0,
        view_height: 0,
        trans_x: new_fixed_i32(0),
        trans_y: new_fixed_i32(0),
        x: ((tile_x as i32) << TILESHIFT) + TILEGLOBAL / 2,
        y: ((tile_y as i32) << TILESHIFT) + TILEGLOBAL / 2,
        dir: DirType::NoDir,
        speed: 0,
        temp1: 0,
        temp2: 0,
        temp3: 0,
        state: Some(state),
        hitpoints: 0,
    }
}

/*
=============================================================================

                ENEMY TILE WORLD MOVEMENT CODE

=============================================================================
*/

pub fn try_walk(k: ObjKey, level_state: &mut LevelState) -> bool {
    let mut door_num: i32 = -1;
    if level_state.obj(k).class == ClassType::Inert {
        level_state.update_obj(k, |obj| {
            match obj.dir {
                DirType::North => {
                    obj.tiley -= 1;
                }
                DirType::NorthEast => {
                    obj.tilex += 1;
                    obj.tiley -= 1;
                }
                DirType::East => {
                    obj.tilex += 1;
                }
                DirType::SouthEast => {
                    obj.tilex += 1;
                    obj.tiley += 1;
                }
                DirType::South => {
                    obj.tiley += 1;
                }
                DirType::SouthWest => {
                    obj.tilex -= 1;
                    obj.tiley += 1;
                }
                DirType::West => {
                    obj.tilex -= 1;
                }
                DirType::NorthWest => {
                    obj.tilex -= 1;
                    obj.tiley -= 1;
                }
                DirType::NoDir => { /* do nothing */ }
            }
        });
    } else {
        let obj = level_state.obj(k);
        match obj.dir {
            DirType::North => {
                if obj.class == ClassType::Dog || obj.class == ClassType::Fake {
                    if !check_diag(level_state, obj.tilex, obj.tiley - 1) {
                        return false;
                    }
                } else {
                    let (check, door) = check_side(level_state, obj.tilex, obj.tiley - 1);
                    if !check {
                        return false;
                    }
                    door_num = door;
                }
                level_state.update_obj(k, |obj| obj.tiley -= 1);
            }
            DirType::NorthEast => {
                if !check_diag(level_state, obj.tilex + 1, obj.tiley - 1) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex + 1, obj.tiley) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex, obj.tiley - 1) {
                    return false;
                }
                level_state.update_obj(k, |obj| {
                    obj.tilex += 1;
                    obj.tiley -= 1;
                });
            }
            DirType::East => {
                if obj.class == ClassType::Dog || obj.class == ClassType::Fake {
                    if !check_diag(level_state, obj.tilex + 1, obj.tiley) {
                        return false;
                    }
                } else {
                    let (check, door) = check_side(level_state, obj.tilex + 1, obj.tiley);
                    if !check {
                        return false;
                    }
                    door_num = door;
                }
                level_state.update_obj(k, |obj| obj.tilex += 1);
            }
            DirType::SouthEast => {
                if !check_diag(level_state, obj.tilex + 1, obj.tiley + 1) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex + 1, obj.tiley) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex, obj.tiley + 1) {
                    return false;
                }
                level_state.update_obj(k, |obj| {
                    obj.tilex += 1;
                    obj.tiley += 1;
                });
            }
            DirType::South => {
                if obj.class == ClassType::Dog || obj.class == ClassType::Fake {
                    if !check_diag(level_state, obj.tilex, obj.tiley + 1) {
                        return false;
                    }
                } else {
                    let (check, door) = check_side(level_state, obj.tilex, obj.tiley + 1);
                    if !check {
                        return false;
                    }
                    door_num = door;
                }
                level_state.update_obj(k, |obj| obj.tiley += 1);
            }
            DirType::SouthWest => {
                if !check_diag(level_state, obj.tilex - 1, obj.tiley + 1) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex - 1, obj.tiley) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex, obj.tiley + 1) {
                    return false;
                }
                level_state.update_obj(k, |obj| {
                    obj.tilex -= 1;
                    obj.tiley += 1;
                });
            }
            DirType::West => {
                if obj.class == ClassType::Dog || obj.class == ClassType::Fake {
                    if !check_diag(level_state, obj.tilex - 1, obj.tiley) {
                        return false;
                    }
                } else {
                    let (check, door) = check_side(level_state, obj.tilex - 1, obj.tiley);
                    if !check {
                        return false;
                    }
                    door_num = door;
                }
                level_state.update_obj(k, |obj| obj.tilex -= 1);
            }
            DirType::NorthWest => {
                if !check_diag(level_state, obj.tilex - 1, obj.tiley - 1) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex - 1, obj.tiley) {
                    return false;
                }
                if !check_diag(level_state, obj.tilex, obj.tiley - 1) {
                    return false;
                }
                level_state.update_obj(k, |obj| {
                    obj.tilex -= 1;
                    obj.tiley -= 1;
                });
            }
            DirType::NoDir => {
                return false;
            }
        }
    }

    if door_num >= 0 {
        {
            let door = &mut level_state.doors[door_num as usize];
            open_door(door);
        }
        level_state.update_obj(k, |obj| {
            obj.distance = -door_num - 1;
        });
        return true;
    }
    let area = {
        let obj = level_state.obj(k);
        (level_state.level.map_segs.segs[0][obj.tiley * MAP_SIZE + obj.tilex] - AREATILE) as usize
    };
    let obj = level_state.mut_obj(k);
    obj.area_number = area;
    obj.distance = TILEGLOBAL;
    return true;
}

fn check_diag(level_state: &LevelState, x: usize, y: usize) -> bool {
    let actor = level_state.actor_at[x][y];
    match actor {
        At::Obj(k) => level_state.obj(k).flags & FL_SHOOTABLE == 0,
        At::Wall(_) => false,
        At::Nothing => true,
    }
}

fn check_side(level_state: &LevelState, x: usize, y: usize) -> (bool, i32) {
    let actor = level_state.actor_at[x][y];
    match actor {
        At::Obj(k) => (level_state.obj(k).flags & FL_SHOOTABLE == 0, -1),
        At::Wall(tile) => {
            if tile >= 128 && tile < 256 {
                return (true, (tile & 63) as i32);
            }
            return (false, -1);
        }
        At::Nothing => (true, -1),
    }
}

pub fn select_dodge_dir(
    k: ObjKey,
    level_state: &mut LevelState,
    player_tile_x: usize,
    player_tile_y: usize,
) {
    let mut dir_try: [DirType; 5] = [DirType::NoDir; 5];
    let turn_around = if level_state.obj(k).flags & FL_FIRSTATTACK != 0 {
        // turning around is only ok the very first time after noticing the
        // player
        level_state.update_obj(k, |obj| {
            obj.flags &= !FL_FIRSTATTACK;
        });

        DirType::NoDir
    } else {
        OPPOSITE[level_state.obj(k).dir as usize]
    };

    let delta_x = player_tile_x as i32 - level_state.obj(k).tilex as i32;
    let delta_y = player_tile_y as i32 - level_state.obj(k).tiley as i32;

    // arrange 5 direction choices in order of preference
    // the four cardinal directions plus the diagonal straight towards
    // the player
    if delta_x > 0 {
        dir_try[1] = DirType::East;
        dir_try[3] = DirType::West;
    } else {
        dir_try[1] = DirType::West;
        dir_try[3] = DirType::East;
    }

    if delta_y > 0 {
        dir_try[2] = DirType::South;
        dir_try[4] = DirType::North;
    } else {
        dir_try[2] = DirType::North;
        dir_try[4] = DirType::South;
    }

    // randomize a bit for dodging

    let abs_dx = delta_x.abs();
    let abs_dy = delta_y.abs();

    if abs_dx > abs_dy {
        let t_dir = dir_try[1];
        dir_try[1] = dir_try[2];
        dir_try[2] = t_dir;
        let t_dir = dir_try[3];
        dir_try[3] = dir_try[4];
        dir_try[4] = t_dir;
    }

    if rnd_t() < 128 {
        let t_dir = dir_try[1];
        dir_try[1] = dir_try[2];
        dir_try[2] = t_dir;
        let t_dir = dir_try[3];
        dir_try[3] = dir_try[4];
        dir_try[4] = t_dir;
    }

    dir_try[0] = DIAGONAL[dir_try[1] as usize][dir_try[2] as usize];

    // try the directions util one works

    for i in 0..5 {
        if dir_try[i] == DirType::NoDir || dir_try[i] == turn_around {
            continue;
        }
        level_state.update_obj(k, |obj| obj.dir = dir_try[i]);
        if try_walk(k, level_state) {
            return;
        }
    }

    // turn around only as a last resort

    if turn_around != DirType::NoDir {
        level_state.update_obj(k, |obj| obj.dir = turn_around);
        if try_walk(k, level_state) {
            return;
        }
    }

    let obj = level_state.mut_obj(k);
    obj.dir = DirType::NoDir;
}

pub fn select_chase_dir(
    k: ObjKey,
    level_state: &mut LevelState,
    player_tile_x: usize,
    player_tile_y: usize,
) {
    let mut d: [DirType; 3] = [DirType::NoDir; 3];

    let old_dir = level_state.obj(k).dir;
    let turn_around = OPPOSITE[old_dir as usize];

    let delta_x = player_tile_x as i32 - level_state.obj(k).tilex as i32;
    let delta_y = player_tile_y as i32 - level_state.obj(k).tiley as i32;

    if delta_x > 0 {
        d[1] = DirType::East;
    } else if delta_x < 0 {
        d[1] = DirType::West;
    }

    if delta_y > 0 {
        d[2] = DirType::South;
    } else if delta_y < 0 {
        d[2] = DirType::North;
    }

    if delta_y.abs() > delta_x.abs() {
        let t_dir = d[1];
        d[1] = d[2];
        d[2] = t_dir;
    }

    if d[1] == turn_around {
        d[1] = DirType::NoDir;
    }
    if d[2] == turn_around {
        d[2] = DirType::NoDir;
    }

    if d[1] != DirType::NoDir {
        level_state.update_obj(k, |obj| obj.dir = d[1]);
        if try_walk(k, level_state) {
            return; /*either moved forward or attacked*/
        }
    }

    if d[2] != DirType::NoDir {
        level_state.update_obj(k, |obj| obj.dir = d[2]);
        if try_walk(k, level_state) {
            return;
        }
    }

    /* there is no direct path to the player, so pick another direction */

    if old_dir != DirType::NoDir {
        level_state.update_obj(k, |obj| obj.dir = old_dir);
        if try_walk(k, level_state) {
            return;
        }
    }

    if rnd_t() > 128 {
        /*randomly determine direction of search*/
        for t_dir in [DirType::North, DirType::NorthWest, DirType::West] {
            if t_dir != turn_around {
                level_state.update_obj(k, |obj| obj.dir = t_dir);
                if try_walk(k, level_state) {
                    return;
                }
            }
        }
    } else {
        for t_dir in [DirType::West, DirType::NorthWest, DirType::North] {
            if t_dir != turn_around {
                level_state.update_obj(k, |obj| obj.dir = t_dir);
                if try_walk(k, level_state) {
                    return;
                }
            }
        }
    }

    if turn_around != DirType::NoDir {
        level_state.update_obj(k, |obj| obj.dir = turn_around);
        if level_state.obj(k).dir != DirType::NoDir {
            if try_walk(k, level_state) {
                return;
            }
        }
    }

    level_state.update_obj(k, |obj| obj.dir = DirType::NoDir); // can't move
}

/// Moves ob be move global units in ob->dir direction
/// Actors are not allowed to move inside the player
/// Does NOT check to see if the move is tile map valid
///
/// ob->x			= adjusted for new position
/// ob->y
pub fn move_obj(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    mov: i32,
    tics: u64,
) {
    level_state.update_obj(k, |obj| {
        match obj.dir {
            DirType::North => obj.y -= mov,
            DirType::NorthEast => {
                obj.x += mov;
                obj.y -= mov;
            }
            DirType::East => {
                obj.x += mov;
            }
            DirType::SouthEast => {
                obj.x += mov;
                obj.y += mov;
            }
            DirType::South => {
                obj.y += mov;
            }
            DirType::SouthWest => {
                obj.x -= mov;
                obj.y += mov;
            }
            DirType::West => {
                obj.x -= mov;
            }
            DirType::NorthWest => {
                obj.x -= mov;
                obj.y -= mov;
            }
            DirType::NoDir => {
                // do nothing
            }
        }
    });

    // check to make sure it's not on top of player

    if level_state.area_by_player[level_state.obj(k).area_number] {
        let delta_x = level_state.obj(k).x - level_state.player().x;
        if delta_x < -MIN_ACTOR_DIST || delta_x > MIN_ACTOR_DIST {
            level_state.update_obj(k, |obj| obj.distance -= mov);
            return;
        }
        let delta_y = level_state.obj(k).y - level_state.player().y;
        if delta_y < -MIN_ACTOR_DIST || delta_y > MIN_ACTOR_DIST {
            level_state.update_obj(k, |obj| obj.distance -= mov);
            return;
        }

        let class = level_state.obj(k).class;
        if class == ClassType::Ghost || class == ClassType::Spectre {
            take_damage(k, (tics * 2) as i32, level_state, game_state, rdr)
        }

        // back up
        let obj = level_state.mut_obj(k);
        match obj.dir {
            DirType::North => obj.y += mov,
            DirType::NorthEast => {
                obj.x -= mov;
                obj.y += mov;
            }
            DirType::East => {
                obj.x -= mov;
            }
            DirType::SouthEast => {
                obj.x -= mov;
                obj.y -= mov;
            }
            DirType::South => {
                obj.y -= mov;
            }
            DirType::SouthWest => {
                obj.x += mov;
                obj.y -= mov;
            }
            DirType::West => {
                obj.x += mov;
            }
            DirType::NorthWest => {
                obj.x += mov;
                obj.y += mov;
            }
            DirType::NoDir => { /* do nothing */ }
        }
        obj.distance -= mov;
    }
}

/// Called by actors that ARE NOT chasing the player.  If the player
/// is detected (by sight, noise, or proximity), the actor is put into
/// it's combat frame and true is returned.
///
/// Incorporates a random reaction delay
pub fn sight_player(
    k: ObjKey,
    level_state: &mut LevelState,
    sound: &mut Sound,
    assets: &Assets,
    tics: u64,
) -> bool {
    let obj = level_state.obj(k);

    if obj.flags & FL_ATTACKMODE != 0 {
        panic!("An actor in ATTACKMODE called SightPlayer!")
    }

    if obj.temp2 != 0 {
        level_state.update_obj(k, |obj| obj.temp2 -= tics as i32);
        if level_state.obj(k).temp2 > 0 {
            return false;
        }
    } else {
        if !level_state.area_by_player[obj.area_number] {
            return false;
        }

        if obj.flags & FL_AMBUSH != 0 {
            if !check_sight(k, level_state) {
                return false;
            }
            level_state.update_obj(k, |obj| obj.flags &= !FL_AMBUSH);
        } else {
            // TODO impl noise check!
            if !check_sight(k, level_state) {
                return false;
            }
        }

        match level_state.obj(k).class {
            ClassType::Guard => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 4);
            }
            ClassType::Officer => {
                level_state.update_obj(k, |obj| obj.temp2 = 2);
            }
            ClassType::Mutant => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 6);
            }
            ClassType::SS => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 6);
            }
            ClassType::Dog => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 8);
            }
            ClassType::Boss
            | ClassType::Schabb
            | ClassType::Fake
            | ClassType::MechaHitler
            | ClassType::RealHitler
            | ClassType::Gretel
            | ClassType::Gift
            | ClassType::Fat
            | ClassType::Spectre
            | ClassType::Angel
            | ClassType::Trans
            | ClassType::Uber
            | ClassType::Will
            | ClassType::Death => {
                level_state.update_obj(k, |obj| obj.temp2 = 1);
            }
            _ => { /* do nothing for the other types */ }
        }
        return false;
    }

    first_sighting(k, level_state, sound, assets);
    true
}

/// Puts an actor into attack mode and possibly reverses the direction
/// if the player is behind it
pub fn first_sighting(k: ObjKey, level_state: &mut LevelState, sound: &mut Sound, assets: &Assets) {
    // react to the player
    let obj = level_state.mut_obj(k);
    match obj.class {
        ClassType::Guard => {
            sound.play_sound_loc_actor(SoundName::HALT, assets, obj);
            new_state(obj, &S_GRDCHASE1);
            obj.speed *= 3; // go faster when chasing player
        }
        ClassType::Mutant => {
            new_state(obj, &S_MUTCHASE1);
            obj.speed *= 3;
        }
        ClassType::Dog => {
            sound.play_sound_loc_actor(SoundName::SPION, assets, obj);
            new_state(obj, &S_DOGCHASE1);
            obj.speed *= 2; // go faster when chasing player
        }
        ClassType::SS => {
            sound.play_sound_loc_actor(SoundName::SCHUTZAD, assets, obj);
            new_state(obj, &S_SSCHASE1);
            obj.speed *= 4;
        }
        ClassType::Boss => {
            sound.play_sound_loc_actor(SoundName::GUTENTAG, assets, obj);
            new_state(obj, &S_BOSSCHASE1);
            obj.speed = SPD_PATROL * 3;
        }
        _ => panic!(
            "first sight for class type not implemented: {:?}",
            obj.class
        ),
    }

    if obj.distance < 0 {
        obj.distance = 0; // ignore the door opening command
    }

    obj.flags |= FL_ATTACKMODE | FL_FIRSTATTACK;
}

pub fn new_state(obj: &mut ObjType, state: &'static StateType) {
    obj.state = Some(state);
    obj.tic_count = state.tic_time;
}

/// Checks a straight line between player and current object
/// If the sight is ok, check alertness and angle to see if they notice
/// returns true if the player has been spoted.
fn check_sight(k: ObjKey, level_state: &mut LevelState) -> bool {
    let player = level_state.player();
    let obj = level_state.obj(k);

    // don't bother tracing a line if the area isn't connected to the player's
    if !level_state.area_by_player[obj.area_number] {
        return false;
    }

    // if the player is real close, sight is automatic
    let delta_x = player.x - obj.x;
    let delta_y = player.y - obj.y;

    if delta_x > -MIN_SIGHT && delta_x < MIN_SIGHT && delta_y > -MIN_SIGHT && delta_y < MIN_SIGHT {
        return true;
    }

    // see if they are looking in the right direction

    match obj.dir {
        DirType::North => {
            if delta_y > 0 {
                return false;
            }
        }
        DirType::East => {
            if delta_x < 0 {
                return false;
            }
        }
        DirType::South => {
            if delta_y < 0 {
                return false;
            }
        }
        DirType::West => {
            if delta_x > 0 {
                return false;
            }
        }
        _ => {}
    }

    // trace a line to check for blocking tiles (corners)
    check_line(level_state, obj)
}

/// Returns true if a straight line between the player and ob is unobstructed
pub fn check_line(level_state: &LevelState, obj: &ObjType) -> bool {
    let player = level_state.player();

    let x1 = obj.x >> UNSIGNEDSHIFT; // 1/256 tile precision
    let y1 = obj.y >> UNSIGNEDSHIFT;
    let xt1 = x1 >> 8;
    let yt1 = y1 >> 8;

    let x2 = player.x >> UNSIGNEDSHIFT;
    let y2 = player.y >> UNSIGNEDSHIFT;
    let mut xt2 = player.tilex as i32;
    let mut yt2 = player.tiley as i32;

    let mut x_step;
    let mut y_step;
    let mut partial;

    let x_dist = xt2.abs_diff(xt1);
    if x_dist > 0 {
        if xt2 > xt1 {
            partial = 256 - (x1 & 0xFF);
            x_step = 1;
        } else {
            partial = x1 & 0xFF;
            x_step = -1;
        }

        let delta_frac = x2.abs_diff(x1) as i32;
        let delta = y2 - y1;
        let ltemp = (delta << 8) / delta_frac;
        if ltemp > 0x7fff {
            y_step = 0x7fff;
        } else if ltemp < -0x7fff {
            y_step = -0x7fff;
        } else {
            y_step = ltemp;
        }

        let mut y_frac = y1 + ((y_step * partial) >> 8);
        let mut x = xt1 + x_step;
        xt2 += x_step;
        loop {
            if x == xt2 {
                break;
            }

            let y: i32 = y_frac >> 8;
            y_frac += y_step;

            let value = level_state.level.tile_map[x as usize][y as usize];
            x += x_step;

            if value == 0 {
                continue;
            }
            if value < 128 || value > 256 {
                return false;
            }

            // see if the door is open enough
            let door = value & !0x80;
            let intercept = y_frac - y_step / 2;

            if intercept > level_state.doors[door as usize].position as i32 {
                return false;
            }
        }
    }

    let y_dist = yt2.abs_diff(yt1);
    if y_dist > 0 {
        if yt2 > yt1 {
            partial = 256 - (y1 & 0xFF);
            y_step = 1;
        } else {
            partial = y1 & 0xFF;
            y_step = -1;
        }

        let delta_frac = y2.abs_diff(y1);
        let delta = x2 - x1;
        let ltemp = (delta << 8) / delta_frac as i32;
        if ltemp > 0x7fff {
            x_step = 0x7fff;
        } else if ltemp < -0x7fff {
            x_step = -0x7fff;
        } else {
            x_step = ltemp;
        }

        let mut x_frac = x1 + ((x_step * partial) >> 8);
        let mut y = yt1 + y_step;
        yt2 += y_step;
        loop {
            if y == yt2 {
                break;
            }

            let x = x_frac >> 8;
            x_frac += x_step;
            let value = level_state.level.tile_map[x as usize][y as usize];
            y += y_step;

            if value == 0 {
                continue;
            }

            if value < 128 || value > 256 {
                return false;
            }

            let door = value & !0x80;
            let intercept = x_frac - x_step / 2;

            if intercept > level_state.doors[door as usize].position as i32 {
                return false;
            }
        }
    }
    true
}

pub fn damage_actor(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    damage_param: usize,
) {
    game_state.made_noise = true;

    let mut damage = damage_param;
    // do double damage if shooting a non attack mode actor
    if level_state.obj(k).flags & FL_ATTACKMODE == 0 {
        damage <<= 1;
    }

    level_state.update_obj(k, |obj| obj.hitpoints -= damage as i32);
    if level_state.obj(k).hitpoints <= 0 {
        kill_actor(k, level_state, game_state, rdr, sound, assets);
    } else {
        if level_state.obj(k).flags & FL_ATTACKMODE == 0 {
            first_sighting(k, level_state, sound, assets); // put into combat mode
        }

        let obj = level_state.mut_obj(k);
        match obj.class {
            ClassType::Guard => {
                if obj.hitpoints & 1 != 0 {
                    new_state(obj, &S_GRDPAIN);
                } else {
                    new_state(obj, &S_GRDPAIN1);
                }
            }
            ClassType::Officer => {
                panic!("damage officer");
            }
            ClassType::Mutant => {
                if obj.hitpoints & 1 != 0 {
                    new_state(obj, &S_MUTPAIN);
                } else {
                    new_state(obj, &S_MUTPAIN1);
                }
            }
            ClassType::SS => {
                if obj.hitpoints & 1 != 0 {
                    new_state(obj, &S_SSPAIN);
                } else {
                    new_state(obj, &S_SSPAIN1);
                }
            }
            _ => { /* do nothing */ }
        }
    }
}

fn kill_actor(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
) {
    {
        let obj = level_state.mut_obj(k);
        let tile_x = (obj.x >> TILESHIFT) as usize;
        let tile_y = (obj.y >> TILESHIFT) as usize;
        obj.tilex = tile_x;
        obj.tiley = tile_y;

        match obj.class {
            ClassType::Guard => {
                give_points(game_state, rdr, sound, assets, 100);
                new_state(obj, &S_GRDDIE1);
                place_item_type(level_state, StaticKind::BoClip2, tile_x, tile_y);
            }
            ClassType::Officer => {
                panic!("kill officer");
            }
            ClassType::Mutant => {
                give_points(game_state, rdr, sound, assets, 700);
                new_state(obj, &S_MUTDIE1);
                place_item_type(level_state, StaticKind::BoClip2, tile_x, tile_y);
            }
            ClassType::SS => {
                give_points(game_state, rdr, sound, assets, 500);
                new_state(obj, &S_SSDIE1);
                if game_state.best_weapon < WeaponType::MachineGun {
                    place_item_type(level_state, StaticKind::BoMachinegun, tile_x, tile_y);
                } else {
                    place_item_type(level_state, StaticKind::BoClip2, tile_x, tile_y);
                }
            }
            ClassType::Dog => {
                give_points(game_state, rdr, sound, assets, 200);
                new_state(obj, &S_DOGDIE1);
            }
            ClassType::Boss => {
                give_points(game_state, rdr, sound, assets, 5000);
                new_state(obj, &S_BOSSDIE1);
                place_item_type(level_state, StaticKind::BoKey1, tile_x, tile_y);
            }
            ClassType::Gretel => {
                panic!("kill gretel");
            }
            ClassType::Gift => {
                panic!("kill gift");
            }
            ClassType::Fat => {
                panic!("kill fat");
            }
            ClassType::Schabb => {
                panic!("kill schabb");
            }
            ClassType::Fake => {
                panic!("kill fake");
            }
            ClassType::MechaHitler => {
                panic!("kill mecha hitler");
            }
            ClassType::RealHitler => {
                panic!("kill real hitler");
            }
            _ => { /* ignore kill on this class of obj */ }
        }
    }

    game_state.kill_count += 1;
    let (tile_x, tile_y) = {
        let obj = level_state.mut_obj(k);
        obj.flags &= !FL_SHOOTABLE;
        obj.flags |= FL_NONMARK;
        (obj.tilex, obj.tiley)
    };

    level_state.actor_at[tile_x][tile_y] = At::Nothing;
}
