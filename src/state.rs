#[cfg(test)]
#[path = "./state_test.rs"]
mod state_test;

use crate::{def::{ObjType, TILESHIFT, TILEGLOBAL, StateType, DirType, ClassType, FL_ATTACKMODE, FL_AMBUSH, LevelState, ObjKey, UNSIGNEDSHIFT}, fixed::new_fixed_i32, time, user::rnd_t};

pub const MIN_SIGHT: i32 = 0x18000;

pub fn spawn_new_obj(tile_x: usize, tile_y: usize, state: &'static StateType, class: ClassType) -> ObjType {
    // TODO set areanumber (what is it used for?)
    // TODO set ticcount (what is it used for?)
    ObjType { 
        class,
        active: true,
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
        state,
    }
}

/// Called by actors that ARE NOT chasing the player.  If the player
/// is detected (by sight, noise, or proximity), the actor is put into
/// it's combat frame and true is returned.
///
/// Incorporates a random reaction delay
pub fn sight_player(k: ObjKey, level_state: &mut LevelState, ticker: &time::Ticker) -> bool {
    let obj = level_state.obj(k);
    
    if obj.flags & FL_ATTACKMODE != 0 {
        panic!("An actor in ATTACKMODE called SightPlayer!")
    }

    if obj.temp2 != 0 {
        let tics = ticker.calc_tics();
        level_state.update_obj(k, |obj| obj.temp2 -= tics as i32);
        if level_state.obj(k).temp2 > 0 {
            return false;
        }
    } else  {
        // TODO check areabyplayer to optimise this

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
            },
            ClassType::Officer => {
                level_state.update_obj(k, |obj| obj.temp2 = 2);
            },
            ClassType::Mutant => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 6);
            },
            ClassType::SS => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 6);
            },
            ClassType::Dog => {
                level_state.update_obj(k, |obj| obj.temp2 = 1 + rnd_t() as i32 / 8);
            },
            ClassType::Boss|ClassType::Schabb|ClassType::Fake|ClassType::MechaHitler|ClassType::RealHitler
            |ClassType::Gretel|ClassType::Gift|ClassType::Fat|ClassType::Spectre|ClassType::Angel
            |ClassType::Trans|ClassType::Uber|ClassType::Will|ClassType::Death => {
                level_state.update_obj(k, |obj| obj.temp2 = 1);
            },
            _ => {/* do nothing for the other types */}
        }
        return false;
    }

    first_sighting(k, level_state);
    true
}


/// Checks a straight line between player and current object
/// If the sight is ok, check alertness and angle to see if they notice
/// returns true if the player has been spoted.
fn check_sight(k: ObjKey, level_state: &mut LevelState) -> bool {

    // TODO check areabyplayer here!

    let player = level_state.player();
    let obj = level_state.obj(k);

    // if the player is real close, sight is automatic
    let delta_x = player.x - obj.x;
    let delta_y = player.y - obj.y;
 
    if delta_x > -MIN_SIGHT && delta_x < MIN_SIGHT
       && delta_y > -MIN_SIGHT && delta_y < MIN_SIGHT {
        return true;
    }

    // see if they are looking in the right direction

    match obj.dir {
        DirType::North => if delta_y > 0 { return false },
        DirType::East => if delta_x < 0 { return false },
        DirType::South => if delta_y < 0 { return false },
        DirType::West => if delta_x > 0 { return false },
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

            let y = y_frac >> 8;
            y_frac += y_step;


            let value =  level_state.level.tile_map[x as usize][y as usize];
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

/// Puts an actor into attack mode and possibly reverses the direction
/// if the player is behind it 
pub fn first_sighting(k: ObjKey, level_state: &mut LevelState) {
    // TODO Impl sight reaction
    panic!("sight reaction!");
}