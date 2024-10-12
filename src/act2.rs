use crate::act1::open_door;
use crate::agent::take_damage;
use crate::def::{ObjType, StateType, Sprite, DirType, EnemyType, SPD_PATROL, ObjKey, LevelState, ControlState, FL_SHOOTABLE, ClassType, DoorAction, TILEGLOBAL, TILESHIFT, RUN_SPEED, FL_VISABLE, GameState, NUM_ENEMIES, Difficulty};
use crate::state::{spawn_new_obj, new_state, sight_player, check_line, select_dodge_dir, select_chase_dir, move_obj};
use crate::play::ProjectionConfig;
use crate::user::rnd_t;
use crate::vga_render::Renderer;

static START_HITPOINTS : [[i32; NUM_ENEMIES]; 4] = [
     [ // BABY MODE	
        25,	// guards
        50,	// officer
        100,	// SS
        1,	// dogs
        850,	// Hans
        850,	// Schabbs
        200,	// fake hitler
        800,	// mecha hitler
        45,	// mutants
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
   
        850,	// Gretel
        850,	// Gift
        850,	// Fat
        5,	// en_spectre,
        1450,	// en_angel,
        850,	// en_trans,
        1050,	// en_uber,
        950,	// en_will,
        1250	// en_death
     ],
     [ // DON'T HURT ME MODE
        25,	// guards
        50,	// officer
        100,	// SS
        1,	// dogs
        950,	// Hans
        950,	// Schabbs
        300,	// fake hitler
        950,	// mecha hitler
        55,	// mutants
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
   
        950,	// Gretel
        950,	// Gift
        950,	// Fat
        10,	// en_spectre,
        1550,	// en_angel,
        950,	// en_trans,
        1150,	// en_uber,
        1050,	// en_will,
        1350	// en_death
    ],
    [ // BRING 'EM ON MODE
        25,	// guards
        50,	// officer
        100,	// SS
        1,	// dogs
   
        1050,	// Hans
        1550,	// Schabbs
        400,	// fake hitler
        1050,	// mecha hitler
   
        55,	// mutants
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
   
        1050,	// Gretel
        1050,	// Gift
        1050,	// Fat
        15,	// en_spectre,
        1650,	// en_angel,
        1050,	// en_trans,
        1250,	// en_uber,
        1150,	// en_will,
        1450	// en_death
    ],
    [ // DEATH INCARNATE MODE
        25,	// guards
        50,	// officer
        100,	// SS
        1,	// dogs
   
        1200,	// Hans
        2400,	// Schabbs
        500,	// fake hitler
        1200,	// mecha hitler
   
        65,	// mutants
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
        25,	// ghosts
   
        1200,	// Gretel
        1200,	// Gift
        1200,	// Fat
        25,	// en_spectre,
        2000,	// en_angel,
        1200,	// en_trans,
        1400,	// en_uber,
        1300,	// en_will,
        1600	// en_death
    ]
];

// guards

pub static S_GRDSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_GRDSTAND),
};

// TODO S_GRDPATH*

pub static S_GRDPAIN : StateType = StateType {
    rotate: 2,
    sprite: Some(Sprite::GuardPain1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDPAIN1 : StateType = StateType {
    rotate: 2,
    sprite: Some(Sprite::GuardPain2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDSHOOT1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot1),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_GRDSHOOT2),
};

pub static S_GRDSHOOT2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_GRDSHOOT3),
};

pub static S_GRDSHOOT3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::GuardShoot3),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDCHASE1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE1S),
};

pub static S_GRDCHASE1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE2),
};

pub static S_GRDCHASE2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE3),
};

pub static S_GRDCHASE3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE3S),
};

pub static S_GRDCHASE3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE4),
};

pub static S_GRDCHASE4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDDIE1 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDie1),
    tic_time: 15,
    think: None,
    action: None, //TODO A_DeathScream
    next: Some(&S_GRDDIE2),
};

pub static S_GRDDIE2 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDie2),
    tic_time: 15,
    think: None,
    action: None, 
    next: Some(&S_GRDDIE3),
};

pub static S_GRDDIE3 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDie3),
    tic_time: 15,
    think: None,
    action: None, 
    next: Some(&S_GRDDIE4),
};

pub static S_GRDDIE4 : StateType = StateType{
    rotate: 0,
    sprite: Some(Sprite::GuardDead),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_GRDDIE4),
};

// officers

pub static S_OFCSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::OfficerS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_OFCSTAND),
};

// mutant

pub static S_MUTSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::MutantS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_MUTSTAND),
};

// SS

pub static S_SSSTAND : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_SSSTAND),   
};

fn t_stand(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &dyn Renderer, control_state: &mut ControlState, prj: &ProjectionConfig) {
    sight_player(k, level_state, tics);
}

fn t_chase(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &dyn Renderer, control_state: &mut ControlState, prj: &ProjectionConfig) {
    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

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
            move_obj(k, level_state, game_state, rdr, x, y, mov, tics);
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

pub fn stand(which: EnemyType, x_tile: usize, y_tile: usize, tile_dir: u16, difficulty: Difficulty) -> ObjType {
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
    stand.hitpoints = START_HITPOINTS[difficulty as usize][which as usize];
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

/*
=============================================================================

								FIGHT

=============================================================================
*/

/// Try to damage the player, based on skill level and player's speed
fn t_shoot(k: ObjKey, _: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &dyn Renderer, _: &mut ControlState, _: &ProjectionConfig) {
    
    // TODO areabyplayer check!

    let obj = level_state.obj(k);
    let player = level_state.player();
    if !check_line(&level_state, obj) { // player is behind a wall
        return;
    }
    
    let dx = obj.tilex.abs_diff(player.tilex);
    let dy = obj.tiley.abs_diff(player.tiley);
    
    let mut dist = if dx > dy { dx } else { dy };
    if obj.class == ClassType::SS || obj.class == ClassType::Boss {
        dist = dist * 2 / 3; // ss are better shots
    }

    let hit_chance;
    if level_state.thrustspeed >= RUN_SPEED {
        if obj.flags & FL_VISABLE != 0 {
            hit_chance = 160 - dist * 16; // player can see to dodge
        } else {
            hit_chance = 160 - dist * 8;
        }
    } else {
        if obj.flags & FL_VISABLE != 0 {
            hit_chance = 256 - dist * 16; // player can see to dodge
        } else {
            hit_chance = 256 - dist * 8;
        }
    }

    // see if the shot was a hit

    if rnd_t() < hit_chance as u8 {
        let damage = if dist < 2 {
            rnd_t() >> 2   
        } else if dist < 4 {
            rnd_t() >> 3
        } else {
            rnd_t() >> 4
        };

        take_damage(k, damage as i32, level_state, game_state, rdr)
    }

    // TODO Play fire sounds!
}