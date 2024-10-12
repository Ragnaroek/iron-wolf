use crate::{def::{ObjType, StateType, Sprite, StateNext, DirType, EnemyType, SPD_PATROL, ObjKey, LevelState, ControlState, At, FL_SHOOTABLE}, state::spawn_new_obj, play::ProjectionConfig};

// guards

pub const S_GRDSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardS1),
    tic_count: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Cycle,
};

pub const S_GRDDIE4 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDead),
    tic_count: 0,
    think: None,
    action: None,
    next: StateNext::Cycle,
};

// officers

pub const S_OFCSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::OfficerS1),
    tic_count: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Cycle,
};

// mutant

pub const S_MUTSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::MutantS1),
    tic_count: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Cycle,
};

// SS

pub const S_SSSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSS1),
    tic_count: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Cycle,   
};

fn t_stand(k: ObjKey, level_state: &mut LevelState, control_state: &mut ControlState, prj: &ProjectionConfig) {
    // TODO Impl (SightPlayer)
}

pub fn dead_guard(x_tile: usize, y_tile: usize) -> ObjType {
    let guard = spawn_new_obj(x_tile, y_tile, &S_GRDDIE4);
    // TODO: Set obclass here (what is it used for)?
    guard
}

pub fn stand(which: EnemyType, x_tile: usize, y_tile: usize, tile_dir: u16) -> ObjType {
    let mut stand = match which {
        EnemyType::Guard => spawn_new_obj(x_tile, y_tile, &S_GRDSTAND),
        EnemyType::Officer => spawn_new_obj(x_tile, y_tile, &S_OFCSTAND),
        EnemyType::Mutant => spawn_new_obj(x_tile, y_tile, &S_MUTSTAND),
        EnemyType::SS => spawn_new_obj(x_tile, y_tile, &S_SSSTAND),
        _ => {
            panic!("illegal stand enemy type: {:?}", which)
        }
    };
    stand.speed = SPD_PATROL;

    // TODO: update gamestate.killtotal

    // TODO: update ambush info

    stand.dir = dir_from_tile(tile_dir*2);
    stand.flags |= FL_SHOOTABLE;
    stand
}

fn dir_from_tile(tile_dir: u16) -> DirType {
	match tile_dir {
		0 => DirType::East,
        1 => DirType::NorthEast,
        2 => DirType::North,
        3 => DirType::NorthWest,
        4 => DirType::West,
        5 => DirType::SouthWest,
        6 => DirType::South,
        7 => DirType::SouthEast,
        8 => DirType::NoDir,
        _ => DirType::NoDir,
	}
}