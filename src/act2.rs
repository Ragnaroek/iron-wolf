use crate::act1::open_door;
use crate::agent::take_damage;
use crate::def::{ActiveType, At, ClassType, ControlState, Difficulty, DirType, DoorAction, EnemyType, GameState, LevelState, ObjKey, ObjType, Sprite, StateType, FL_SHOOTABLE, FL_VISABLE, ICON_ARROWS, MAP_SIZE, MIN_ACTOR_DIST, NUM_ENEMIES, RUN_SPEED, SPD_DOG, SPD_PATROL, TILEGLOBAL, TILESHIFT};
use crate::game;
use crate::state::{check_line, move_obj, new_state, select_chase_dir, select_dodge_dir, sight_player, spawn_new_obj, try_walk};
use crate::play::ProjectionConfig;
use crate::user::rnd_t;
use crate::vga_render::VGARenderer;

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

pub static S_GRDPATH1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH1S),
};

pub static S_GRDPATH1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_GRDPATH3),
};

pub static S_GRDPATH2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH3),
};

pub static S_GRDPATH3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH3S),
};

pub static S_GRDPATH3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_GRDPATH4),
};

pub static S_GRDPATH4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH1),
};

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

// ghosts

// TODO Impl ghosts

// dogs

pub static S_DOGPATH1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH1S),
};

pub static S_DOGPATH1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_DOGPATH2),
};

pub static S_DOGPATH2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH3),
};

pub static S_DOGPATH3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH3S),
};

pub static S_DOGPATH3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_DOGPATH4),
};

pub static S_DOGPATH4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH1),
};

pub static S_DOGJUMP1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP2),
};

pub static S_DOGJUMP2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogJump2),
    tic_time: 10,
    think: Some(t_bite),
    action: None,
    next: Some(&S_DOGJUMP3),
};

pub static S_DOGJUMP3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogJump3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP4),
};

pub static S_DOGJUMP4 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP5),
};

pub static S_DOGJUMP5 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGCHASE1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE1S),
};

pub static S_DOGCHASE1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE2),
};

pub static S_DOGCHASE2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE3),
};

pub static S_DOGCHASE3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE3S),
};

pub static S_DOGCHASE3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE4),
};

pub static S_DOGCHASE4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGDIE1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogDie1),
    tic_time: 15,
    think: None,
    action: None, // TODO A_DeathScream
    next: Some(&S_DOGDIE2),
};

pub static S_DOGDIE2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogDie2),
    tic_time: 15,
    think: None,
    action: None, 
    next: Some(&S_DOGDIE3),
};

pub static S_DOGDIE3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogDie3),
    tic_time: 15,
    think: None,
    action: None, 
    next: Some(&S_DOGDEAD),
};

pub static S_DOGDEAD : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::DogDead),
    tic_time: 15,
    think: None,
    action: None, 
    next: Some(&S_DOGDEAD),
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

pub static S_SSPATH1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH1S),
};

pub static S_SSPATH1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_SSPATH2),    
};

pub static S_SSPATH2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH3),
};

pub static S_SSPATH3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH3S),
};

pub static S_SSPATH3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_SSPATH4),
};

pub static S_SSPATH4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH1),
};

pub static S_SSPAIN : StateType = StateType {
    rotate: 2,
    sprite: Some(Sprite::SSPAIN1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSPAIN1 : StateType = StateType {
    rotate: 2,
    sprite: Some(Sprite::SSPAIN2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSSHOOT1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT1),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT2),    
};

pub static S_SSSHOOT2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT3),
};

pub static S_SSSHOOT3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT4),
};

pub static S_SSSHOOT4 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT5),
};

pub static S_SSSHOOT5 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT6),
};

pub static S_SSSHOOT6 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT7),
};

pub static S_SSSHOOT7 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT8),
};

pub static S_SSSHOOT8 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT9),
};

pub static S_SSSHOOT9 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSCHASE1 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE1S),
};

pub static S_SSCHASE1S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SSCHASE2),
};

pub static S_SSCHASE2 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE3),
};

pub static S_SSCHASE3 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE3S)
};

pub static S_SSCHASE3S : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SSCHASE4),
};

pub static S_SSCHASE4 : StateType = StateType {
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSDIE1 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSDIE1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_SSDIE2),
};

pub static S_SSDIE2 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSDIE2),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_SSDIE3)
};

pub static S_SSDIE3 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSDIE3),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_SSDIE4),
};

pub static S_SSDIE4 : StateType = StateType {
    rotate: 0,
    sprite: Some(Sprite::SSDEAD),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_SSDIE4),
};

fn t_path(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {
    if sight_player(k, level_state, tics) {
        return;
    }
    
    if level_state.obj(k).dir == DirType::NoDir {
        select_path_dir(k, level_state);
        if level_state.obj(k).dir == DirType::NoDir {
            return; // all movement is blocked
        }
    }

    let mut mov = level_state.obj(k).speed * tics as i32;
    while mov != 0 {
        let dist = level_state.obj(k).distance; 
        if dist < 0 { // waiting for a door to open
            let door = &mut level_state.doors[(-dist-1) as usize];
           open_door(door);
           if door.action != DoorAction::Open {
                return;
           }
           level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL);
        }

        let dist = level_state.obj(k).distance; 
        if mov < dist {
            let (x, y) = {
                let player = level_state.player();
                (player.x, player.y)
            };
            move_obj(k, level_state, game_state, rdr, x, y, mov, tics);
            break;
        }

        if level_state.obj(k).tilex > MAP_SIZE || level_state.obj(k).tiley > MAP_SIZE {
            panic!("T_Path hit a wall at {},{}, dir {:?}", level_state.obj(k).tilex, level_state.obj(k).tiley, level_state.obj(k).dir);
        }

        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL/2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL/2;
            mov -= obj.distance;
        });

        select_path_dir(k, level_state);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_dog_chase(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {
    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if level_state.obj(k).dir == DirType::NoDir {
        select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }

    let mut mov = level_state.obj(k).speed * tics as i32;
    while mov != 0 {
        // check for byte range

        let mut dx = level_state.player().x - level_state.obj(k).x;
        if dx < 0 {
            dx = -dx;
        }
        dx -= mov;
        if dx <= MIN_ACTOR_DIST {
            let mut dy = level_state.player().y - level_state.obj(k).y;
            if dy < 0 {
                dy = -dy;
            }
            dy -= mov;
            if dy <= MIN_ACTOR_DIST {
                level_state.update_obj(k, |obj| new_state(obj, &S_DOGJUMP1));
                return;
            }
        }

        let dist = level_state.obj(k).distance; 
        if mov < dist {
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
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL/2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL/2;
            mov -= obj.distance;
        });

        select_dodge_dir(k, level_state, level_state.obj(k).tilex, level_state.obj(k).tiley);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_bite(k: ObjKey, _: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {

    // TODO PlaySoundLocActor(DOGATTACKSND,ob) 

    let mut dx = level_state.player().x - level_state.obj(k).x;
    if dx < 0 {
        dx = -dx;
    }
    dx -= TILEGLOBAL;
    if dx <= MIN_ACTOR_DIST {
        let mut dy = level_state.player().y - level_state.obj(k).y;
        if dy < 0 {
            dy = -dy;
        }
        dy -= TILEGLOBAL;
        if dy <= MIN_ACTOR_DIST {
            if rnd_t() < 180 {
                take_damage(k, (rnd_t()>>4) as i32, level_state, game_state, rdr);
                return;
            }
        }
    }
}

fn select_path_dir(k: ObjKey, level_state: &mut LevelState) {    
    let spot = level_state.level.info_map[level_state.obj(k).tilex][level_state.obj(k).tiley].wrapping_sub(ICON_ARROWS);
    if spot < 8 {
        level_state.update_obj(k, |obj| obj.dir = dir_type(spot));
    }

    level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL);

    if !try_walk(k, level_state) {
        level_state.update_obj(k, |obj| obj.dir = DirType::NoDir);    
    } 
}

// supplied u16 must be within [0,8]. Everything outside this range is 
// mapped to DirType::NoDir.
fn dir_type(u: u16) -> DirType {
    match u {
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

fn t_stand(k: ObjKey, tics: u64, level_state: &mut LevelState, _: &mut GameState, _: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {
    sight_player(k, level_state, tics);
}

fn t_chase(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {
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
                ClassType::SS => Some(&S_SSSHOOT1),
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

pub fn spawn_dead_guard(actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, x_tile: usize, y_tile: usize) {
    let obj = spawn_new_obj(x_tile, y_tile, &S_GRDDIE4, ClassType::Inert);
    spawn(actors, actor_at, obj)
}

pub fn spawn_stand(which: EnemyType, actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, x_tile: usize, y_tile: usize, tile_dir: u16, difficulty: Difficulty) {
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
    stand.hitpoints = START_HITPOINTS[difficulty as usize][which as usize];
    stand.dir = dir_type(tile_dir*2);
    stand.flags |= FL_SHOOTABLE;
    
    spawn(actors, actor_at, stand);
}

pub fn spawn_patrol(which: EnemyType, actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, game_state: &mut GameState, x_tile: usize, y_tile: usize, tile_dir: u16, difficulty: Difficulty) {
    let mut patrol = match which {
        EnemyType::Guard => {
            let mut obj = spawn_new_obj(x_tile, y_tile, &S_GRDPATH1, ClassType::Guard);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        },
        EnemyType::Officer => {
            todo!("spawn with &S_OFCPATH1");
            /*
            let obj = spawn_new_obj(x_tile, y_tile, &S_OFCPATH1, ClassType::Officer);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
            */
        },
        EnemyType::SS => {
            let mut obj = spawn_new_obj(x_tile, y_tile, &S_SSPATH1, ClassType::SS);
            obj.speed = SPD_PATROL;
            // TODO check loadedgame
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        },
        EnemyType::Mutant => {
            todo!("spawn with &S_MUTPATH1");
            /*
            let obj = spawn_new_obj(x_tile, y_tile, &S_MUTPATH1, ClassType::Mutant);
            obj.speed = SPD_PATROL;
            // TODO check loadedgame
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
            */
        },
        EnemyType::Dog => {
            let mut obj = spawn_new_obj(x_tile, y_tile, &S_DOGPATH1, ClassType::Dog);
            obj.speed = SPD_DOG;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        },
        _ => {
            panic!("illegal stand enemy type: {:?}", which)
        }
    };

    patrol.dir = dir_type(tile_dir*2);
    patrol.hitpoints = START_HITPOINTS[difficulty as usize][which as usize];
    patrol.distance = TILEGLOBAL;
    patrol.flags |= FL_SHOOTABLE;
    patrol.active = ActiveType::Yes;

    actor_at[patrol.tilex][patrol.tiley] = At::Nothing;

    match tile_dir {
        0 => patrol.tilex += 1, 
        1 => patrol.tiley -= 1,
        2 => patrol.tilex -= 1,
        3 => patrol.tiley += 1,
        _ => {/* do nothing */}
    }
    spawn(actors, actor_at, patrol);
}

// spawns the obj into the map
fn spawn(actors: &mut Vec<ObjType>, actor_at: &mut Vec<Vec<At>>, obj: ObjType) {
	actors.push(obj);
	let key = ObjKey(actors.len()); // +1 offset (not len()-1), since player will be later at position 0 and positions will shift
	actor_at[obj.tilex][obj.tiley] = At::Obj(key)
}

/*
=============================================================================
								FIGHT
=============================================================================
*/

/// Try to damage the player, based on skill level and player's speed
fn t_shoot(k: ObjKey, _: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, _: &mut ControlState, _: &ProjectionConfig) {
    let obj = level_state.obj(k);
    if !level_state.area_by_player[obj.area_number] {
        return;
    }

    let player = level_state.player();
    if !check_line(&level_state, obj) { // player is behind a wall
        return;
    }
    
    let dx = obj.tilex.abs_diff(player.tilex);
    let dy = obj.tiley.abs_diff(player.tiley);
    
    let mut dist = if dx > dy { dx } else { dy } as i32;
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
    if hit_chance > 0 && rnd_t() < hit_chance as u8 {
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

fn a_death_scream(k: ObjKey, tics: u64, level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, control_state: &mut ControlState, prj: &ProjectionConfig) {
    // TODO play death sounds
}