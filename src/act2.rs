use crate::act1::open_door;
use crate::def::{ObjType, StateType, Sprite, StateNext, DirType, EnemyType, SPD_PATROL, ObjKey, LevelState, ControlState, FL_SHOOTABLE, ClassType, DoorAction, TILEGLOBAL, TILESHIFT};
use crate::state::{spawn_new_obj, new_state, sight_player, check_line, select_dodge_dir, select_chase_dir, move_obj};
use crate::play::ProjectionConfig;
use crate::time;
use crate::user::rnd_t;

// guards

pub static S_GRDSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Next(&S_GRDSTAND),
};

pub static S_GRDSHOOT1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot1),
    tic_time: 20,
    think: None,
    action: None,
    next: StateNext::Next(&S_GRDSHOOT2),
};

pub static S_GRDSHOOT2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: StateNext::Next(&S_GRDSHOOT3),
};

pub static S_GRDSHOOT3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot3),
    tic_time: 20,
    think: None,
    action: None,
    next: StateNext::Next(&S_GRDCHASE1),
};

pub static S_GRDCHASE1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: StateNext::Next(&S_GRDCHASE1S),
};

pub static S_GRDCHASE1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 3,
    think: None,
    action: None,
    next: StateNext::Next(&S_GRDCHASE2),
};

pub static S_GRDCHASE2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: StateNext::Next(&S_GRDCHASE3),
};

pub static S_GRDCHASE3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: StateNext::Next(&S_GRDCHASE3S),
};

pub static S_GRDCHASE3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 3,
    think: None,
    action: None,
    next: StateNext::Next(&S_GRDCHASE4),
};

pub static S_GRDCHASE4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: StateNext::Next(&S_GRDCHASE1),
};

// S_GRDCHASE4.next = StateNext::Next(&S_GRDCHASE1)

pub static S_GRDDIE4 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDead),
    tic_time: 0,
    think: None,
    action: None,
    next: StateNext::Next(&S_GRDDIE4),
};

// officers

pub static S_OFCSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::OfficerS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Next(&S_OFCSTAND),
};

// mutant

pub static S_MUTSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::MutantS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Next(&S_MUTSTAND),
};

// SS

pub static S_SSSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: StateNext::Next(&S_SSSTAND),   
};

fn t_stand(k: ObjKey, level_state: &mut LevelState, ticker: &time::Ticker, control_state: &mut ControlState, prj: &ProjectionConfig) {
    sight_player(k, level_state, ticker);
}

fn t_chase(k: ObjKey, level_state: &mut LevelState, ticker: &time::Ticker, control_state: &mut ControlState, prj: &ProjectionConfig) {
    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    let tics = ticker.calc_tics();

    // TODO check gamestate.victoryflag

    let mut dodge = false;
    if check_line(level_state, level_state.obj(k)) { // got a shot at player?
        let obj = level_state.obj(k);
        let player = level_state.player();
        let dx = obj.tilex.abs_diff(player.tilex);
        let dy = obj.tiley.abs_diff(player.tiley);
        let dist =  dx.max(dy);
        let chance = if dist == 0 || (dist == 1) && obj.distance < 0x4000 {
            300 // always hit
        } else {
            ((tics as usize) << 4) / dist
        };

        if (rnd_t() as usize) < chance {
            // go into attack frame
            let state_change = match obj.class {
                ClassType::Guard => Some(&S_GRDSHOOT1),
                _ => panic!("impl state change for {:?}", obj.class)
            };

            if let Some(state) = state_change {
                level_state.update_obj(k, |obj| new_state(obj, state))
            }
            return;
        }
        dodge = true;
    }

    if level_state.obj(k).dir == DirType::NoDir {
        if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y)
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y)
        } 

        if level_state.obj(k).dir == DirType::NoDir {
            return; // object is blocked in
        }
    }

    let mut mov = level_state.obj(k).speed * tics as i32; 
    while mov != 0 {
        let distance = level_state.obj(k).distance;
        if distance < 0 {
            // waiting for a door to open
            let door = &mut level_state.doors[(-distance-1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL) // go ahead, the door is now opoen
        }

        if mov < level_state.obj(k).distance {
            let (x, y) = {
                let player = level_state.player();
                (player.x, player.y)
            };
            level_state.update_obj(k, |obj| move_obj(x, y, obj, mov, tics));
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        }); 

        mov -= level_state.obj(k).distance;

        if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;  // object is blocked in
        }
    }
}

pub fn dead_guard(x_tile: usize, y_tile: usize) -> ObjType {
    spawn_new_obj(x_tile, y_tile, &S_GRDDIE4, ClassType::Inert)
}

pub fn stand(which: EnemyType, x_tile: usize, y_tile: usize, tile_dir: u16) -> ObjType {
    let mut stand = match which {
        EnemyType::Guard => spawn_new_obj(x_tile, y_tile, &S_GRDSTAND, ClassType::Guard),
        EnemyType::Officer => spawn_new_obj(x_tile, y_tile, &S_OFCSTAND, ClassType::Officer),
        EnemyType::Mutant => spawn_new_obj(x_tile, y_tile, &S_MUTSTAND, ClassType::Mutant),
        EnemyType::SS => spawn_new_obj(x_tile, y_tile, &S_SSSTAND, ClassType::SS),
        _ => {
            panic!("illegal stand enemy type: {:?}", which)
        }
    };
    stand.speed = SPD_PATROL;

    // TODO: update gamestate.killtotal

    // TODO: update ambush info

    // TODO: set hitpoints

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

// FIGHT

fn t_shoot(k: ObjKey) {
    panic!("impl t_shoot");
}