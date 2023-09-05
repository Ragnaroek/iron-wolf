use crate::{def::{ObjType, StateType, Sprite, StateNext}, state::spawn_new_obj};

pub const S_GRDDIE4 : StateType = StateType{
    rotate: false,
    sprite: Some(Sprite::GuardDead),
    tic_count: 0,
    think: None,
    next: StateNext::Cycle,
};

pub fn dead_guard(x_tile: usize, y_tile: usize) -> ObjType {
    let guard = spawn_new_obj(x_tile, y_tile, &S_GRDDIE4);
    // TODO: Set obclass here (what is it used for)?
    guard
}