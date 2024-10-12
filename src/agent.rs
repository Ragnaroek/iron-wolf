use crate::def::Level;

use super::def::{StateType, ObjType, ObjKey, LevelState, ANGLES};

const ANGLE_SCALE : i32 = 20;

pub const S_PLAYER : StateType = StateType{
    think: Some(t_player),
    next: None,
};

fn t_player(k: ObjKey, level_state: &mut LevelState) {
    control_movement(k, level_state);

}

fn control_movement(k: ObjKey, level_state: &mut LevelState) {
    // side to side move

    level_state.angle_frac += level_state.control.x;
    let angle_units = level_state.angle_frac / ANGLE_SCALE;
    level_state.angle_frac -= angle_units*ANGLE_SCALE;


    let x = level_state.control.x;

    let mut ob = level_state.mut_obj(k);
    ob.angle -= angle_units;
    if ob.angle >= ANGLES {
        ob.angle -= ANGLES;
    }
    if ob.angle < 0 {
        ob.angle += ANGLES;
    }

    // forward/backwards move
}