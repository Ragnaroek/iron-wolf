
use super::play::ProjectionConfig;
use super::game::TILESHIFT;
use super::draw::fixed_by_frac;
use super::def::{StateType, ObjType, ObjKey, LevelState, At, new_fixed_i32, ANGLES, MIN_DIST, PLAYER_SIZE, TILEGLOBAL};

const ANGLE_SCALE : i32 = 20;
const MOVE_SCALE : i32 = 150;
const BACKMOVE_SCALE : i32 = 100;

pub const S_PLAYER : StateType = StateType{
    think: Some(t_player),
    next: None,
};

fn t_player(k: ObjKey, level_state: &mut LevelState, prj: &ProjectionConfig) {
    control_movement(k, level_state, prj);
}

pub fn spawn_player(tilex: usize, tiley: usize, dir: i32) -> ObjType {
	let r = ObjType{
		angle: (1-dir)*90, 
        pitch: 0,
		tilex,
		tiley,
		x: ((tilex as i32) << TILESHIFT) + TILEGLOBAL / 2,
		y: ((tiley as i32) << TILESHIFT) + TILEGLOBAL / 2,
        state: &S_PLAYER,
    };

    //TODO init_areas

    r
}

fn control_movement(k: ObjKey, level_state: &mut LevelState, prj: &ProjectionConfig) {
    // side to side move
    let control_x = level_state.control.x;
    let control_y = level_state.control.y;
    
    level_state.angle_frac += control_x;
    let angle_units = level_state.angle_frac / ANGLE_SCALE;
    level_state.angle_frac -= angle_units*ANGLE_SCALE;

    {
        let mut ob = level_state.mut_obj(k);
        //println!("ob.angle={}", ob.angle);
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
        println!("control_y={}", control_y);
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

    //println!("xmove={}, ymove={}, angle={}", x_move, y_move, angle);
    println!("xmove=({}, {})={};a={}", speed.to_i32(), prj.cos(angle as usize), x_move.to_i32(), angle);
    println!("ymove=({}, {})={};a={}", speed.to_i32(), prj.sin(angle as usize), y_move.to_i32(), angle);

    /*{
        let ob = level_state.obj(k);
        println!("before: ob.x={},ob.y={},ob.angle={}", ob.x, ob.y, ob.angle);
    }*/
    clip_move(k, level_state, x_move.to_i32(), y_move.to_i32());
    /*{
        let ob = level_state.obj(k);
        println!("after: ob.x={},ob.y={},ob.angle={}", ob.x, ob.y, ob.angle);
    }*/


    let mut ob = level_state.mut_obj(k);
    ob.tilex = ob.x as usize >> TILESHIFT;
    ob.tiley = ob.y as usize >> TILESHIFT;

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
    let mut obj = level_state.mut_obj(k);
    obj.x = dx;
    obj.y = dy;
}