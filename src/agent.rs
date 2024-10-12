use crate::act1::operate_door;
use crate::play::ProjectionConfig;
use crate::def::{StateType, ObjType, ObjKey, LevelState, ControlState, Button, Dir, At, ANGLES, ANGLES_I32, MIN_DIST, PLAYER_SIZE, TILEGLOBAL, TILESHIFT, StateNext, FL_NEVERMARK, DirType, ClassType};
use crate::fixed::{new_fixed_i32, fixed_by_frac};
use crate::time;

const ANGLE_SCALE : i32 = 20;
const MOVE_SCALE : i32 = 150;
const BACKMOVE_SCALE : i32 = 100;

pub const S_PLAYER : StateType = StateType{
    rotate: 0,
    sprite: None,
    tic_count: 0,
    think: Some(t_player),
    action: None,
    next: StateNext::None,
};

fn t_player(k: ObjKey, level_state: &mut LevelState, _: &time::Ticker, control_state: &mut ControlState, prj: &ProjectionConfig) {
    if control_state.button_state[Button::Use as usize] {
        cmd_use(level_state, control_state);
    }

    control_movement(k, level_state, control_state, prj);
}

fn cmd_use(level_state: &mut LevelState, control_state: &mut ControlState) {

    //TODO pushable wall, elevator

    let check_x;
    let check_y;
    let dir;
    let mut elevator_ok = true;

    // find which cardinal direction the player is facing
    let player = level_state.player();
    if player.angle < ANGLES_I32/8 || player.angle > 7*ANGLES_I32/8 {
        check_x = player.tilex+1;
        check_y = player.tiley;
        dir = Dir::East;
        elevator_ok = true;
    } else if player.angle < 3*ANGLES_I32/8 {
        check_x = player.tilex;
        check_y = player.tiley-1;
        dir = Dir::North;
        elevator_ok = false;
    } else if player.angle < 5*ANGLES_I32/8 {
        check_x = player.tilex-1;
        check_y = player.tiley;
        dir = Dir::West;
        elevator_ok = true;
    } else {
        check_x = player.tilex;
        check_y = player.tiley+1;
        dir = Dir::South;
        elevator_ok = false;
    }

    let doornum = level_state.level.tile_map[check_x][check_y];
    if !control_state.button_held[Button::Use as usize] && doornum & 0x80 != 0 {
        control_state.button_held[Button::Use as usize] = true;
        operate_door(doornum & !0x80, level_state);
    }
}

pub fn spawn_player(tilex: usize, tiley: usize, dir: i32) -> ObjType {
	let r = ObjType{
        class: ClassType::Player,
        active: true,
		angle: (1-dir)*90,
        flags: FL_NEVERMARK, 
        pitch: 0,
		tilex,
		tiley,
        view_x: 0,
        view_height: 0,
        trans_x: new_fixed_i32(0),
        trans_y: new_fixed_i32(0),
		x: ((tilex as i32) << TILESHIFT) + TILEGLOBAL / 2,
		y: ((tiley as i32) << TILESHIFT) + TILEGLOBAL / 2,
        speed: 0,
        dir: DirType::NoDir,
        temp1: 0,
        temp2: 0,
        temp3: 0,
        state: &S_PLAYER,
    };

    //TODO init_areas

    r
}

fn control_movement(k: ObjKey, level_state: &mut LevelState, control_state: &mut ControlState, prj: &ProjectionConfig) {
    // side to side move
    let control_x = control_state.control.x;
    let control_y = control_state.control.y;
    
    control_state.angle_frac += control_x;
    let angle_units = control_state.angle_frac / ANGLE_SCALE;
    control_state.angle_frac -= angle_units*ANGLE_SCALE;

    {
        let ob = level_state.mut_obj(k);
        ob.angle -= angle_units;
        if ob.angle >= ANGLES as i32 {
            ob.angle -= ANGLES as i32;
        }
        if ob.angle < 0 {
            ob.angle += ANGLES as i32;
        }
    }

    // forward/backwards move
    let ob = level_state.obj(k);
    if control_y < 0 {
        thrust(k, level_state, prj, ob.angle, -control_y*MOVE_SCALE)
    } else if control_y > 0 {
        let mut angle = ob.angle + ANGLES as i32 /2;
        if angle >= ANGLES as i32 {
            angle -= ANGLES as i32;
        }
        thrust(k, level_state, prj, angle, control_y*BACKMOVE_SCALE);
    }
}

pub fn thrust(k: ObjKey, level_state: &mut LevelState, prj: &ProjectionConfig, angle: i32, speed_param: i32) {
    let speed = new_fixed_i32(if speed_param >= MIN_DIST*2 {
        MIN_DIST*2-1
    } else {
        speed_param
    });

    let x_move = fixed_by_frac(speed, prj.cos(angle as usize));
    let y_move = -fixed_by_frac(speed, prj.sin(angle as usize));

    clip_move(k, level_state, x_move.to_i32(), y_move.to_i32());

    let obj = level_state.mut_obj(k);
    obj.tilex = obj.x as usize >> TILESHIFT;
    obj.tiley = obj.y as usize >> TILESHIFT;

    //TODO update thrustspeed + reset funnyticount (only for Spear?)
}

fn clip_move(k : ObjKey, level_state: &mut LevelState, x_move: i32, y_move: i32) {
    let (base_x, base_y) = {
        let ob = level_state.obj(k);
        (ob.x, ob.y)
    };

    set_move(k, level_state, base_x+x_move, base_y+y_move);
    if try_move(k, level_state) {
        return;
    }

    // TODO add noclip check here (for cheats)

    // TODO Play HITWALLSND sound here

    set_move(k, level_state, base_x+x_move, base_y);
    if try_move(k, level_state) {
        return;
    }

    set_move(k, level_state, base_x, base_y+y_move);
    if try_move(k, level_state) {
        return;
    }

    set_move(k, level_state, base_x, base_y);
}

fn try_move(k : ObjKey, level_state: &mut LevelState) -> bool {
    let ob = level_state.obj(k);

    let xl = (ob.x - PLAYER_SIZE) >> TILESHIFT;
    let yl = (ob.y - PLAYER_SIZE) >> TILESHIFT;
    let xh = (ob.x + PLAYER_SIZE) >> TILESHIFT;
    let yh = (ob.y + PLAYER_SIZE) >> TILESHIFT;
    
    // check for solid walls
    for y in yl..=yh {
        for x in xl..=xh {
            if match level_state.actor_at[x as usize][y as usize] {
                At::Wall(_) => true,
                _ => false,
            } {
                return false;
            } 
        }
    }

    // TODO check for actors

    return true
}

fn set_move(k: ObjKey, level_state: &mut LevelState, dx: i32, dy: i32) {
    let obj = level_state.mut_obj(k);
    obj.x = dx;
    obj.y = dy;
}