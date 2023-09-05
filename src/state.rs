use crate::{def::{ObjType, TILESHIFT, TILEGLOBAL, StateType}, fixed::new_fixed_i32};


pub fn spawn_new_obj(tile_x: usize, tile_y: usize, state: &'static StateType) -> ObjType {
    // TODO set areanumber (what is it used for?)
    // TODO set ticcount (what is it used for?)
    // TODO set dir (what is it used for?)
    ObjType { 
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
        state,
    }
}