#[cfg(test)]
#[path = "./act2_test.rs"]
mod act2_test;

use std::pin::Pin;

use crate::act1::open_door;
use crate::agent::{S_ATTACK, S_PLAYER, take_damage};
use crate::assets::{SoundName, W3D1};
use crate::def::{
    AMBUSH_TILE, ANGLES_F64, ActiveType, Actors, At, ClassType, ControlState, Difficulty, DirType,
    DoorAction, EnemyType, FL_AMBUSH, FL_NEVERMARK, FL_NONMARK, FL_SHOOTABLE, FL_VISABLE,
    GameState, ICON_ARROWS, LevelState, MAP_SIZE, MIN_ACTOR_DIST, NUM_ENEMIES, ObjKey, ObjType,
    PLAYER_SIZE, PlayState, RUN_SPEED, SCREENLOC, SPD_DOG, SPD_PATROL, STATUS_LINES, Sprite,
    StateType, TILEGLOBAL, TILESHIFT,
};
use crate::fixed::{Fixed, fixed_by_frac};
use crate::game::AREATILE;
use crate::inter::write;
use crate::map::MapSegs;
use crate::play::{draw_play_border, finish_palette_shifts};
use crate::rc::{FizzleFadeAbortable, RenderContext};
use crate::sd::DigiMode;
use crate::start::quit;
use crate::state::{
    check_line, move_obj, new_state, select_chase_dir, select_dodge_dir, select_run_dir,
    sight_player, spawn_new_obj, try_walk,
};
use crate::user::rnd_t;

const BJ_RUN_SPEED: i32 = 2048;
const BJ_JUMP_SPEED: i32 = 680;

const PROJ_SIZE: i32 = 0x2000;
const PROJECTILE_SIZE: u32 = 0xc000;

static START_HITPOINTS: [[i32; NUM_ENEMIES]; 4] = [
    [
        // BABY MODE
        25,   // guards
        50,   // officer
        100,  // SS
        1,    // dogs
        850,  // Hans
        850,  // Schabbs
        200,  // fake hitler
        800,  // mecha hitler
        45,   // mutants
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        850,  // Gretel
        850,  // Gift
        850,  // Fat
        5,    // en_spectre,
        1450, // en_angel,
        850,  // en_trans,
        1050, // en_uber,
        950,  // en_will,
        1250, // en_death
    ],
    [
        // DON'T HURT ME MODE
        25,   // guards
        50,   // officer
        100,  // SS
        1,    // dogs
        950,  // Hans
        950,  // Schabbs
        300,  // fake hitler
        950,  // mecha hitler
        55,   // mutants
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        950,  // Gretel
        950,  // Gift
        950,  // Fat
        10,   // en_spectre,
        1550, // en_angel,
        950,  // en_trans,
        1150, // en_uber,
        1050, // en_will,
        1350, // en_death
    ],
    [
        // BRING 'EM ON MODE
        25,   // guards
        50,   // officer
        100,  // SS
        1,    // dogs
        1050, // Hans
        1550, // Schabbs
        400,  // fake hitler
        1050, // mecha hitler
        55,   // mutants
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        1050, // Gretel
        1050, // Gift
        1050, // Fat
        15,   // en_spectre,
        1650, // en_angel,
        1050, // en_trans,
        1250, // en_uber,
        1150, // en_will,
        1450, // en_death
    ],
    [
        // DEATH INCARNATE MODE
        25,   // guards
        50,   // officer
        100,  // SS
        1,    // dogs
        1200, // Hans
        2400, // Schabbs
        500,  // fake hitler
        1200, // mecha hitler
        65,   // mutants
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        25,   // ghosts
        1200, // Gretel
        1200, // Gift
        1200, // Fat
        25,   // en_spectre,
        2000, // en_angel,
        1200, // en_trans,
        1400, // en_uber,
        1300, // en_will,
        1600, // en_death
    ],
];

static GUARD_DEATH_SCREAMS: [SoundName; 8] = [
    SoundName::DEATHSCREAM1,
    SoundName::DEATHSCREAM2,
    SoundName::DEATHSCREAM3,
    SoundName::DEATHSCREAM4,
    SoundName::DEATHSCREAM5,
    SoundName::DEATHSCREAM7,
    SoundName::DEATHSCREAM8,
    SoundName::DEATHSCREAM9,
];

// TODO consistently use id with groups and gaps :/

// guards (1000)

pub static S_GRDSTAND: StateType = StateType {
    id: 10000,
    rotate: 1,
    sprite: Some(Sprite::GuardS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_GRDSTAND),
};

pub static S_GRDPATH1: StateType = StateType {
    id: 10001,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH1S),
};

pub static S_GRDPATH1S: StateType = StateType {
    id: 1002,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH2),
};

pub static S_GRDPATH2: StateType = StateType {
    id: 10003,
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH3),
};

pub static S_GRDPATH3: StateType = StateType {
    id: 10004,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH3S),
};

pub static S_GRDPATH3S: StateType = StateType {
    id: 10005,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH4),
};

pub static S_GRDPATH4: StateType = StateType {
    id: 10006,
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_GRDPATH1),
};

pub static S_GRDPAIN: StateType = StateType {
    id: 10007,
    rotate: 2,
    sprite: Some(Sprite::GuardPain1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDPAIN1: StateType = StateType {
    id: 10008,
    rotate: 2,
    sprite: Some(Sprite::GuardPain2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDSHOOT1: StateType = StateType {
    id: 10009,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot1),
    tic_time: 20,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDSHOOT2),
};

pub static S_GRDSHOOT2: StateType = StateType {
    id: 10010,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRDSHOOT3),
};

pub static S_GRDSHOOT3: StateType = StateType {
    id: 10011,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot3),
    tic_time: 20,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDCHASE1: StateType = StateType {
    id: 10012,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE1S),
};

pub static S_GRDCHASE1S: StateType = StateType {
    id: 10013,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE2),
};

pub static S_GRDCHASE2: StateType = StateType {
    id: 10014,
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE3),
};

pub static S_GRDCHASE3: StateType = StateType {
    id: 10015,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE3S),
};

pub static S_GRDCHASE3S: StateType = StateType {
    id: 10016,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE4),
};

pub static S_GRDCHASE4: StateType = StateType {
    id: 10017,
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDDIE1: StateType = StateType {
    id: 10018,
    rotate: 0,
    sprite: Some(Sprite::GuardDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_GRDDIE2),
};

pub static S_GRDDIE2: StateType = StateType {
    id: 10019,
    rotate: 0,
    sprite: Some(Sprite::GuardDie2),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDDIE3),
};

pub static S_GRDDIE3: StateType = StateType {
    id: 10020,
    rotate: 0,
    sprite: Some(Sprite::GuardDie3),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDDIE4),
};

pub static S_GRDDIE4: StateType = StateType {
    id: 10021,
    rotate: 0,
    sprite: Some(Sprite::GuardDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRDDIE4),
};

// ghosts (1100)

pub static S_BLINKYCHASE1: StateType = StateType {
    id: 10100,
    rotate: 0,
    sprite: Some(Sprite::BlinkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_BLINKYCHASE2),
};

pub static S_BLINKYCHASE2: StateType = StateType {
    id: 10101,
    rotate: 0,
    sprite: Some(Sprite::BlinkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_BLINKYCHASE1),
};

pub static S_INKYCHASE1: StateType = StateType {
    id: 10102,
    rotate: 0,
    sprite: Some(Sprite::InkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_INKYCHASE2),
};

pub static S_INKYCHASE2: StateType = StateType {
    id: 10103,
    rotate: 0,
    sprite: Some(Sprite::InkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_INKYCHASE1),
};

pub static S_PINKYCHASE1: StateType = StateType {
    id: 10104,
    rotate: 0,
    sprite: Some(Sprite::PinkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_PINKYCHASE2),
};

pub static S_PINKYCHASE2: StateType = StateType {
    id: 10105,
    rotate: 0,
    sprite: Some(Sprite::PinkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_PINKYCHASE1),
};

pub static S_CLYDECHASE1: StateType = StateType {
    id: 10106,
    rotate: 0,
    sprite: Some(Sprite::ClydeW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_CLYDECHASE2),
};

pub static S_CLYDECHASE2: StateType = StateType {
    id: 10107,
    rotate: 0,
    sprite: Some(Sprite::ClydeW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    async_action: None,
    next: Some(&S_CLYDECHASE1),
};

// dogs

pub static S_DOGPATH1: StateType = StateType {
    id: 10200,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH1S),
};

pub static S_DOGPATH1S: StateType = StateType {
    id: 10201,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH2),
};

pub static S_DOGPATH2: StateType = StateType {
    id: 10202,
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH3),
};

pub static S_DOGPATH3: StateType = StateType {
    id: 10203,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH3S),
};

pub static S_DOGPATH3S: StateType = StateType {
    id: 10204,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH4),
};

pub static S_DOGPATH4: StateType = StateType {
    id: 10205,
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_DOGPATH1),
};

pub static S_DOGJUMP1: StateType = StateType {
    id: 10206,
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGJUMP2),
};

pub static S_DOGJUMP2: StateType = StateType {
    id: 10207,
    rotate: 0,
    sprite: Some(Sprite::DogJump2),
    tic_time: 10,
    think: Some(t_bite),
    action: None,
    async_action: None,
    next: Some(&S_DOGJUMP3),
};

pub static S_DOGJUMP3: StateType = StateType {
    id: 10208,
    rotate: 0,
    sprite: Some(Sprite::DogJump3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGJUMP4),
};

pub static S_DOGJUMP4: StateType = StateType {
    id: 10209,
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGJUMP5),
};

pub static S_DOGJUMP5: StateType = StateType {
    id: 10210,
    rotate: 0,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGCHASE1: StateType = StateType {
    id: 10211,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE1S),
};

pub static S_DOGCHASE1S: StateType = StateType {
    id: 10212,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE2),
};

pub static S_DOGCHASE2: StateType = StateType {
    id: 10213,
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE3),
};

pub static S_DOGCHASE3: StateType = StateType {
    id: 10214,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE3S),
};

pub static S_DOGCHASE3S: StateType = StateType {
    id: 10215,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE4),
};

pub static S_DOGCHASE4: StateType = StateType {
    id: 10216,
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    async_action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGDIE1: StateType = StateType {
    id: 10217,
    rotate: 0,
    sprite: Some(Sprite::DogDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_DOGDIE2),
};

pub static S_DOGDIE2: StateType = StateType {
    id: 10218,
    rotate: 0,
    sprite: Some(Sprite::DogDie2),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGDIE3),
};

pub static S_DOGDIE3: StateType = StateType {
    id: 10219,
    rotate: 0,
    sprite: Some(Sprite::DogDie3),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGDEAD),
};

pub static S_DOGDEAD: StateType = StateType {
    id: 10220,
    rotate: 0,
    sprite: Some(Sprite::DogDead),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_DOGDEAD),
};

// officers

pub static S_OFCSTAND: StateType = StateType {
    id: 10300,
    rotate: 1,
    sprite: Some(Sprite::OfficerS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_OFCSTAND),
};

pub static S_OFCPATH1: StateType = StateType {
    id: 10301,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH1S),
};

pub static S_OFCPATH1S: StateType = StateType {
    id: 10302,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH2),
};

pub static S_OFCPATH2: StateType = StateType {
    id: 10303,
    rotate: 1,
    sprite: Some(Sprite::OfficerW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH3),
};

pub static S_OFCPATH3: StateType = StateType {
    id: 10304,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH3S),
};

pub static S_OFCPATH3S: StateType = StateType {
    id: 10305,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH4),
};

pub static S_OFCPATH4: StateType = StateType {
    id: 10306,
    rotate: 1,
    sprite: Some(Sprite::OfficerW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_OFCPATH1),
};

pub static S_OFCPAIN: StateType = StateType {
    id: 10307,
    rotate: 2,
    sprite: Some(Sprite::OfficerPain1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCPAIN1: StateType = StateType {
    id: 10308,
    rotate: 2,
    sprite: Some(Sprite::OfficerPain2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCSHOOT1: StateType = StateType {
    id: 10309,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot1),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCSHOOT2),
};

pub static S_OFCSHOOT2: StateType = StateType {
    id: 10310,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_OFCSHOOT3),
};

pub static S_OFCSHOOT3: StateType = StateType {
    id: 10311,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCCHASE1: StateType = StateType {
    id: 10312,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE1S),
};

pub static S_OFCCHASE1S: StateType = StateType {
    id: 10313,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE2),
};

pub static S_OFCCHASE2: StateType = StateType {
    id: 10314,
    rotate: 1,
    sprite: Some(Sprite::OfficerW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE3),
};

pub static S_OFCCHASE3: StateType = StateType {
    id: 10315,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE3S),
};

pub static S_OFCCHASE3S: StateType = StateType {
    id: 10316,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE4),
};

pub static S_OFCCHASE4: StateType = StateType {
    id: 10317,
    rotate: 1,
    sprite: Some(Sprite::OfficerW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCDIE1: StateType = StateType {
    id: 10318,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie1),
    tic_time: 11,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_OFCDIE2),
};

pub static S_OFCDIE2: StateType = StateType {
    id: 10319,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie2),
    tic_time: 11,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCDIE3),
};

pub static S_OFCDIE3: StateType = StateType {
    id: 10320,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie3),
    tic_time: 11,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCDIE4),
};

pub static S_OFCDIE4: StateType = StateType {
    id: 10321,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie4),
    tic_time: 11,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCDIE5),
};

pub static S_OFCDIE5: StateType = StateType {
    id: 10322,
    rotate: 0,
    sprite: Some(Sprite::OfficerDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_OFCDIE5),
};

// mutant

pub static S_MUTSTAND: StateType = StateType {
    id: 10400,
    rotate: 1,
    sprite: Some(Sprite::MutantS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_MUTSTAND),
};

pub static S_MUTPATH1: StateType = StateType {
    id: 10401,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH1S),
};

pub static S_MUTPATH1S: StateType = StateType {
    id: 10402,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH2),
};

pub static S_MUTPATH2: StateType = StateType {
    id: 10403,
    rotate: 1,
    sprite: Some(Sprite::MutantW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH3),
};

pub static S_MUTPATH3: StateType = StateType {
    id: 10404,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH3S),
};

pub static S_MUTPATH3S: StateType = StateType {
    id: 10405,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH4),
};

pub static S_MUTPATH4: StateType = StateType {
    id: 10406,
    rotate: 1,
    sprite: Some(Sprite::MutantW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_MUTPATH1),
};

pub static S_MUTPAIN: StateType = StateType {
    id: 10407,
    rotate: 2,
    sprite: Some(Sprite::MutantPain1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTPAIN1: StateType = StateType {
    id: 10408,
    rotate: 2,
    sprite: Some(Sprite::MutantPain2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTSHOOT1: StateType = StateType {
    id: 10409,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot1),
    tic_time: 6,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MUTSHOOT2),
};

pub static S_MUTSHOOT2: StateType = StateType {
    id: 10410,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot2),
    tic_time: 20,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTSHOOT3),
};

pub static S_MUTSHOOT3: StateType = StateType {
    id: 10411,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MUTSHOOT4),
};

pub static S_MUTSHOOT4: StateType = StateType {
    id: 10412,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot4),
    tic_time: 20,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTCHASE1: StateType = StateType {
    id: 10413,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE1S),
};

pub static S_MUTCHASE1S: StateType = StateType {
    id: 10414,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE2),
};

pub static S_MUTCHASE2: StateType = StateType {
    id: 10415,
    rotate: 1,
    sprite: Some(Sprite::MutantW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE3),
};

pub static S_MUTCHASE3: StateType = StateType {
    id: 10416,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE3S),
};

pub static S_MUTCHASE3S: StateType = StateType {
    id: 10417,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE4),
};

pub static S_MUTCHASE4: StateType = StateType {
    id: 10418,
    rotate: 1,
    sprite: Some(Sprite::MutantW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTDIE1: StateType = StateType {
    id: 10419,
    rotate: 0,
    sprite: Some(Sprite::MutantDie1),
    tic_time: 7,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_MUTDIE2),
};

pub static S_MUTDIE2: StateType = StateType {
    id: 10420,
    rotate: 0,
    sprite: Some(Sprite::MutantDie2),
    tic_time: 7,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTDIE3),
};

pub static S_MUTDIE3: StateType = StateType {
    id: 10421,
    rotate: 0,
    sprite: Some(Sprite::MutantDie3),
    tic_time: 7,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTDIE4),
};

pub static S_MUTDIE4: StateType = StateType {
    id: 10422,
    rotate: 0,
    sprite: Some(Sprite::MutantDie4),
    tic_time: 7,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTDIE5),
};

pub static S_MUTDIE5: StateType = StateType {
    id: 10423,
    rotate: 0,
    sprite: Some(Sprite::MutantDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MUTDIE5),
};

// SS

pub static S_SSSTAND: StateType = StateType {
    id: 10500,
    rotate: 1,
    sprite: Some(Sprite::SSS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_SSSTAND),
};

pub static S_SSPATH1: StateType = StateType {
    id: 10501,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_SSPATH1S),
};

pub static S_SSPATH1S: StateType = StateType {
    id: 10502,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSPATH2),
};

pub static S_SSPATH2: StateType = StateType {
    id: 10503,
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_SSPATH3),
};

pub static S_SSPATH3: StateType = StateType {
    id: 10504,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_SSPATH3S),
};

pub static S_SSPATH3S: StateType = StateType {
    id: 10505,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSPATH4),
};

pub static S_SSPATH4: StateType = StateType {
    id: 10506,
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    async_action: None,
    next: Some(&S_SSPATH1),
};

pub static S_SSPAIN: StateType = StateType {
    id: 10507,
    rotate: 2,
    sprite: Some(Sprite::SSPAIN1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSPAIN1: StateType = StateType {
    id: 10508,
    rotate: 2,
    sprite: Some(Sprite::SSPAIN2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSSHOOT1: StateType = StateType {
    id: 10509,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT1),
    tic_time: 20,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSSHOOT2),
};

pub static S_SSSHOOT2: StateType = StateType {
    id: 10510,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_SSSHOOT3),
};

pub static S_SSSHOOT3: StateType = StateType {
    id: 10511,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSSHOOT4),
};

pub static S_SSSHOOT4: StateType = StateType {
    id: 10512,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_SSSHOOT5),
};

pub static S_SSSHOOT5: StateType = StateType {
    id: 10513,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSSHOOT6),
};

pub static S_SSSHOOT6: StateType = StateType {
    id: 10514,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_SSSHOOT7),
};

pub static S_SSSHOOT7: StateType = StateType {
    id: 10515,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSSHOOT8),
};

pub static S_SSSHOOT8: StateType = StateType {
    id: 10516,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_SSSHOOT9),
};

pub static S_SSSHOOT9: StateType = StateType {
    id: 10517,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSCHASE1: StateType = StateType {
    id: 10518,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE1S),
};

pub static S_SSCHASE1S: StateType = StateType {
    id: 10519,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE2),
};

pub static S_SSCHASE2: StateType = StateType {
    id: 10520,
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE3),
};

pub static S_SSCHASE3: StateType = StateType {
    id: 10521,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE3S),
};

pub static S_SSCHASE3S: StateType = StateType {
    id: 10522,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE4),
};

pub static S_SSCHASE4: StateType = StateType {
    id: 10523,
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSDIE1: StateType = StateType {
    id: 10524,
    rotate: 0,
    sprite: Some(Sprite::SSDIE1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_SSDIE2),
};

pub static S_SSDIE2: StateType = StateType {
    id: 10525,
    rotate: 0,
    sprite: Some(Sprite::SSDIE2),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSDIE3),
};

pub static S_SSDIE3: StateType = StateType {
    id: 10526,
    rotate: 0,
    sprite: Some(Sprite::SSDIE3),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSDIE4),
};

pub static S_SSDIE4: StateType = StateType {
    id: 10527,
    rotate: 0,
    sprite: Some(Sprite::SSDEAD),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SSDIE4),
};

//
// hans
//
pub static S_BOSSSTAND: StateType = StateType {
    id: 10600,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_BOSSSTAND),
};

pub static S_BOSSCHASE1: StateType = StateType {
    id: 10601,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE1S),
};

pub static S_BOSSCHASE1S: StateType = StateType {
    id: 10602,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE2),
};

pub static S_BOSSCHASE2: StateType = StateType {
    id: 10603,
    rotate: 0,
    sprite: Some(Sprite::BossW2),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE3),
};

pub static S_BOSSCHASE3: StateType = StateType {
    id: 10604,
    rotate: 0,
    sprite: Some(Sprite::BossW3),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE3S),
};

pub static S_BOSSCHASE3S: StateType = StateType {
    id: 10605,
    rotate: 0,
    sprite: Some(Sprite::BossW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE4),
};

pub static S_BOSSCHASE4: StateType = StateType {
    id: 10606,
    rotate: 0,
    sprite: Some(Sprite::BossW4),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE1),
};

pub static S_BOSSDIE1: StateType = StateType {
    id: 10607,
    rotate: 0,
    sprite: Some(Sprite::BossDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_BOSSDIE2),
};

pub static S_BOSSDIE2: StateType = StateType {
    id: 10608,
    rotate: 0,
    sprite: Some(Sprite::BossDie2),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSDIE3),
};

pub static S_BOSSDIE3: StateType = StateType {
    id: 10609,
    rotate: 0,
    sprite: Some(Sprite::BossDie3),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSDIE4),
};

pub static S_BOSSDIE4: StateType = StateType {
    id: 10610,
    rotate: 0,
    sprite: Some(Sprite::BossDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSDIE4),
};

pub static S_BOSSSHOOT1: StateType = StateType {
    id: 10611,
    rotate: 0,
    sprite: Some(Sprite::BossShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSSHOOT2),
};

pub static S_BOSSSHOOT2: StateType = StateType {
    id: 10612,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT3),
};

pub static S_BOSSSHOOT3: StateType = StateType {
    id: 10613,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT4),
};

pub static S_BOSSSHOOT4: StateType = StateType {
    id: 10614,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT5),
};

pub static S_BOSSSHOOT5: StateType = StateType {
    id: 10615,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT6),
};

pub static S_BOSSSHOOT6: StateType = StateType {
    id: 10616,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT7),
};

pub static S_BOSSSHOOT7: StateType = StateType {
    id: 10617,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_BOSSSHOOT8),
};

pub static S_BOSSSHOOT8: StateType = StateType {
    id: 10618,
    rotate: 0,
    sprite: Some(Sprite::BossShoot1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOSSCHASE1),
};

//
// schabb
//
pub static S_SCHABBSTAND: StateType = StateType {
    id: 10700,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_SCHABBSTAND),
};

pub static S_SCHABBCHASE1: StateType = StateType {
    id: 10701,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: Some(t_schabb),
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE1S),
};

pub static S_SCHABBCHASE1S: StateType = StateType {
    id: 10702,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE2),
};
pub static S_SCHABBCHASE2: StateType = StateType {
    id: 10703,
    rotate: 0,
    sprite: Some(Sprite::SchabbW2),
    tic_time: 8,
    think: Some(t_schabb),
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE3),
};

pub static S_SCHABBCHASE3: StateType = StateType {
    id: 10704,
    rotate: 0,
    sprite: Some(Sprite::SchabbW3),
    tic_time: 10,
    think: Some(t_schabb),
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE3S),
};

pub static S_SCHABBCHASE3S: StateType = StateType {
    id: 10705,
    rotate: 0,
    sprite: Some(Sprite::SchabbW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE4),
};

pub static S_SCHABBCHASE4: StateType = StateType {
    id: 10706,
    rotate: 0,
    sprite: Some(Sprite::SchabbW4),
    tic_time: 8,
    think: Some(t_schabb),
    action: None,
    async_action: None,
    next: Some(&S_SCHABBCHASE1),
};

pub static S_SCHABBDEATHCAM_140: StateType = StateType {
    id: 10707,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE1_140),
};

pub static S_SCHABBDEATHCAM_10: StateType = StateType {
    id: 10708,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE1_10),
};

pub static S_SCHABBDIE1_140: StateType = StateType {
    id: 10709,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_SCHABBDIE2_140),
};

pub static S_SCHABBDIE1_10: StateType = StateType {
    id: 10710,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_SCHABBDIE2_10),
};

pub static S_SCHABBDIE2_140: StateType = StateType {
    id: 10711,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 140,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE3),
};

pub static S_SCHABBDIE2_10: StateType = StateType {
    id: 10712,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE3),
};

pub static S_SCHABBDIE3: StateType = StateType {
    id: 10713,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE4),
};

pub static S_SCHABBDIE4: StateType = StateType {
    id: 10714,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE5),
};

pub static S_SCHABBDIE5: StateType = StateType {
    id: 10715,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBDIE6),
};

pub static S_SCHABBDIE6: StateType = StateType {
    id: 10716,
    rotate: 0,
    sprite: Some(Sprite::SchabbDead),
    tic_time: 20,
    think: None,
    action: None,
    async_action: Some(a_start_death_cam),
    next: Some(&S_SCHABBDIE6),
};

pub static S_SCHABBSHOOT1: StateType = StateType {
    id: 10717,
    rotate: 0,
    sprite: Some(Sprite::SchabbShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SCHABBSHOOT2),
};

pub static S_SCHABBSHOOT2: StateType = StateType {
    id: 10718,
    rotate: 0,
    sprite: Some(Sprite::SchabbShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_schabb_throw),
    async_action: None,
    next: Some(&S_SCHABBCHASE1),
};

pub static S_NEEDLE1: StateType = StateType {
    id: 10719,
    rotate: 0,
    sprite: Some(Sprite::Hypo1),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_NEEDLE2),
};

pub static S_NEEDLE2: StateType = StateType {
    id: 10720,
    rotate: 0,
    sprite: Some(Sprite::Hypo2),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_NEEDLE3),
};

pub static S_NEEDLE3: StateType = StateType {
    id: 10721,
    rotate: 0,
    sprite: Some(Sprite::Hypo3),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_NEEDLE4),
};

pub static S_NEEDLE4: StateType = StateType {
    id: 10722,
    rotate: 0,
    sprite: Some(Sprite::Hypo4),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_NEEDLE1),
};

//
// gift
//
pub static S_GIFTSTAND: StateType = StateType {
    id: 10800,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_GIFTSTAND),
};

pub static S_GIFTCHASE1: StateType = StateType {
    id: 10801,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 10,
    think: Some(t_gift),
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE1S),
};

pub static S_GIFTCHASE1S: StateType = StateType {
    id: 10802,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE2),
};

pub static S_GIFTCHASE2: StateType = StateType {
    id: 10803,
    rotate: 0,
    sprite: Some(Sprite::GiftW2),
    tic_time: 8,
    think: Some(t_gift),
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE3),
};

pub static S_GIFTCHASE3: StateType = StateType {
    id: 10804,
    rotate: 0,
    sprite: Some(Sprite::GiftW3),
    tic_time: 10,
    think: Some(t_gift),
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE3S),
};

pub static S_GIFTCHASE3S: StateType = StateType {
    id: 10805,
    rotate: 0,
    sprite: Some(Sprite::GiftW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE4),
};

pub static S_GIFTCHASE4: StateType = StateType {
    id: 10806,
    rotate: 0,
    sprite: Some(Sprite::GiftW4),
    tic_time: 8,
    think: Some(t_gift),
    action: None,
    async_action: None,
    next: Some(&S_GIFTCHASE1),
};

pub static S_GIFTDEATHCAM_140: StateType = StateType {
    id: 10807,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE1_140),
};

pub static S_GIFTDEATHCAM_5: StateType = StateType {
    id: 10808,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE1_5),
};

pub static S_GIFTDIE1_140: StateType = StateType {
    id: 10809,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_GIFTDIE2_140),
};

pub static S_GIFTDIE1_5: StateType = StateType {
    id: 10810,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_GIFTDIE2_5),
};

pub static S_GIFTDIE2_140: StateType = StateType {
    id: 10811,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 140,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE3),
};

pub static S_GIFTDIE2_5: StateType = StateType {
    id: 10812,
    rotate: 0,
    sprite: Some(Sprite::GiftW1),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE3),
};

pub static S_GIFTDIE3: StateType = StateType {
    id: 10813,
    rotate: 0,
    sprite: Some(Sprite::GiftDie1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE4),
};

pub static S_GIFTDIE4: StateType = StateType {
    id: 10814,
    rotate: 0,
    sprite: Some(Sprite::GiftDie2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE5),
};

pub static S_GIFTDIE5: StateType = StateType {
    id: 10815,
    rotate: 0,
    sprite: Some(Sprite::GiftDie3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTDIE6),
};

pub static S_GIFTDIE6: StateType = StateType {
    id: 10816,
    rotate: 0,
    sprite: Some(Sprite::GiftDead),
    tic_time: 20,
    think: None,
    action: None,
    async_action: Some(a_start_death_cam),
    next: Some(&S_GIFTDIE6),
};

pub static S_GIFTSHOOT1: StateType = StateType {
    id: 10817,
    rotate: 0,
    sprite: Some(Sprite::GiftShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GIFTSHOOT2),
};

pub static S_GIFTSHOOT2: StateType = StateType {
    id: 10818,
    rotate: 0,
    sprite: Some(Sprite::GiftShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_gift_throw),
    async_action: None,
    next: Some(&S_GIFTCHASE1),
};

pub static S_ROCKET: StateType = StateType {
    id: 10819,
    rotate: 1,
    sprite: Some(Sprite::Rocket1),
    tic_time: 3,
    think: Some(t_projectile),
    action: Some(a_smoke),
    async_action: None,
    next: Some(&S_ROCKET),
};

pub static S_SMOKE1: StateType = StateType {
    id: 10820,
    rotate: 0,
    sprite: Some(Sprite::Smoke1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SMOKE2),
};

pub static S_SMOKE2: StateType = StateType {
    id: 10821,
    rotate: 0,
    sprite: Some(Sprite::Smoke2),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SMOKE3),
};

pub static S_SMOKE3: StateType = StateType {
    id: 10822,
    rotate: 0,
    sprite: Some(Sprite::Smoke3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_SMOKE4),
};

pub static S_SMOKE4: StateType = StateType {
    id: 10823,
    rotate: 0,
    sprite: Some(Sprite::Smoke4),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: None,
};

pub static S_BOOM1: StateType = StateType {
    id: 10824,
    rotate: 0,
    sprite: Some(Sprite::Boom1),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOOM2),
};

pub static S_BOOM2: StateType = StateType {
    id: 10825,
    rotate: 0,
    sprite: Some(Sprite::Boom2),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BOOM3),
};

pub static S_BOOM3: StateType = StateType {
    id: 10826,
    rotate: 0,
    sprite: Some(Sprite::Boom3),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: None,
};

//
// fake hitler
//
pub static S_FAKESTAND: StateType = StateType {
    id: 10900,
    rotate: 0,
    sprite: Some(Sprite::FakeW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_FAKESTAND),
};

pub static S_FAKECHASE1: StateType = StateType {
    id: 10901,
    rotate: 0,
    sprite: Some(Sprite::FakeW1),
    tic_time: 10,
    think: Some(t_fake),
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE1S),
};

pub static S_FAKECHASE1S: StateType = StateType {
    id: 10902,
    rotate: 0,
    sprite: Some(Sprite::FakeW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE2),
};

pub static S_FAKECHASE2: StateType = StateType {
    id: 10903,
    rotate: 0,
    sprite: Some(Sprite::FakeW2),
    tic_time: 8,
    think: Some(t_fake),
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE3),
};

pub static S_FAKECHASE3: StateType = StateType {
    id: 10904,
    rotate: 0,
    sprite: Some(Sprite::FakeW3),
    tic_time: 10,
    think: Some(t_fake),
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE3S),
};

pub static S_FAKECHASE3S: StateType = StateType {
    id: 10905,
    rotate: 0,
    sprite: Some(Sprite::FakeW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE4),
};

pub static S_FAKECHASE4: StateType = StateType {
    id: 10906,
    rotate: 0,
    sprite: Some(Sprite::FakeW4),
    tic_time: 8,
    think: Some(t_fake),
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE1),
};

pub static S_FAKEDIE1: StateType = StateType {
    id: 10907,
    rotate: 0,
    sprite: Some(Sprite::FakeDie1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_FAKEDIE2),
};

pub static S_FAKEDIE2: StateType = StateType {
    id: 10908,
    rotate: 0,
    sprite: Some(Sprite::FakeDie2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKEDIE3),
};

pub static S_FAKEDIE3: StateType = StateType {
    id: 10909,
    rotate: 0,
    sprite: Some(Sprite::FakeDie3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKEDIE4),
};

pub static S_FAKEDIE4: StateType = StateType {
    id: 10910,
    rotate: 0,
    sprite: Some(Sprite::FakeDie4),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKEDIE5),
};

pub static S_FAKEDIE5: StateType = StateType {
    id: 10911,
    rotate: 0,
    sprite: Some(Sprite::FakeDie5),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKEDIE6),
};

pub static S_FAKEDIE6: StateType = StateType {
    id: 10912,
    rotate: 0,
    sprite: Some(Sprite::FakeDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKEDIE6),
};

pub static S_FAKESHOOT1: StateType = StateType {
    id: 10913,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT2),
};

pub static S_FAKESHOOT2: StateType = StateType {
    id: 10914,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT3),
};

pub static S_FAKESHOOT3: StateType = StateType {
    id: 10915,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT4),
};

pub static S_FAKESHOOT4: StateType = StateType {
    id: 10916,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT5),
};

pub static S_FAKESHOOT5: StateType = StateType {
    id: 10917,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT6),
};

pub static S_FAKESHOOT6: StateType = StateType {
    id: 10918,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT7),
};

pub static S_FAKESHOOT7: StateType = StateType {
    id: 10919,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT8),
};

pub static S_FAKESHOOT8: StateType = StateType {
    id: 10920,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: Some(t_fake_fire),
    async_action: None,
    next: Some(&S_FAKESHOOT9),
};

pub static S_FAKESHOOT9: StateType = StateType {
    id: 10921,
    rotate: 0,
    sprite: Some(Sprite::FakeShoot),
    tic_time: 8,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FAKECHASE1),
};

pub static S_FIRE1: StateType = StateType {
    id: 10922,
    rotate: 0,
    sprite: Some(Sprite::Fire1),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_FIRE2),
};

pub static S_FIRE2: StateType = StateType {
    id: 10923,
    rotate: 0,
    sprite: Some(Sprite::Fire2),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    async_action: None,
    next: Some(&S_FIRE1),
};

//
// mecha hitler
//
pub static S_MECHASTAND: StateType = StateType {
    id: 11000,
    rotate: 0,
    sprite: Some(Sprite::MechaW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_MECHASTAND),
};

pub static S_MECHACHASE1: StateType = StateType {
    id: 11001,
    rotate: 0,
    sprite: Some(Sprite::MechaW1),
    tic_time: 10,
    think: Some(t_chase),
    action: Some(a_mecha_sound),
    async_action: None,
    next: Some(&S_MECHACHASE1S),
};

pub static S_MECHACHASE1S: StateType = StateType {
    id: 11002,
    rotate: 0,
    sprite: Some(Sprite::MechaW1),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MECHACHASE2),
};

pub static S_MECHACHASE2: StateType = StateType {
    id: 11003,
    rotate: 0,
    sprite: Some(Sprite::MechaW2),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MECHACHASE3),
};

pub static S_MECHACHASE3: StateType = StateType {
    id: 11004,
    rotate: 0,
    sprite: Some(Sprite::MechaW3),
    tic_time: 10,
    think: Some(t_chase),
    action: Some(a_mecha_sound),
    async_action: None,
    next: Some(&S_MECHACHASE3S),
};

pub static S_MECHACHASE3S: StateType = StateType {
    id: 11005,
    rotate: 0,
    sprite: Some(Sprite::MechaW3),
    tic_time: 6,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MECHACHASE4),
};

pub static S_MECHACHASE4: StateType = StateType {
    id: 11006,
    rotate: 0,
    sprite: Some(Sprite::MechaW4),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_MECHACHASE1),
};

pub static S_MECHADIE1: StateType = StateType {
    id: 11007,
    rotate: 0,
    sprite: Some(Sprite::MechaDie1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_MECHADIE2),
};

pub static S_MECHADIE2: StateType = StateType {
    id: 11008,
    rotate: 0,
    sprite: Some(Sprite::MechaDie2),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MECHADIE3),
};

pub static S_MECHADIE3: StateType = StateType {
    id: 11009,
    rotate: 0,
    sprite: Some(Sprite::MechaDie3),
    tic_time: 10,
    think: None,
    action: Some(a_hitler_morph),
    async_action: None,
    next: Some(&S_MECHADIE4),
};

pub static S_MECHADIE4: StateType = StateType {
    id: 11010,
    rotate: 0,
    sprite: Some(Sprite::MechaDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MECHADIE4),
};

pub static S_MECHASHOOT1: StateType = StateType {
    id: 11011,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_MECHASHOOT2),
};

pub static S_MECHASHOOT2: StateType = StateType {
    id: 11012,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MECHASHOOT3),
};

pub static S_MECHASHOOT3: StateType = StateType {
    id: 11013,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MECHASHOOT4),
};

pub static S_MECHASHOOT4: StateType = StateType {
    id: 11014,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MECHASHOOT5),
};

pub static S_MECHASHOOT5: StateType = StateType {
    id: 11015,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MECHASHOOT6),
};

pub static S_MECHASHOOT6: StateType = StateType {
    id: 11016,
    rotate: 0,
    sprite: Some(Sprite::MechaShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_MECHACHASE1),
};

//
// real hitler
//
pub static S_HITLERCHASE1: StateType = StateType {
    id: 11100,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 6,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE1S),
};

pub static S_HITLERCHASE1S: StateType = StateType {
    id: 11101,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 4,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE2),
};

pub static S_HITLERCHASE2: StateType = StateType {
    id: 11102,
    rotate: 0,
    sprite: Some(Sprite::HitlerW2),
    tic_time: 2,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE3),
};

pub static S_HITLERCHASE3: StateType = StateType {
    id: 11103,
    rotate: 0,
    sprite: Some(Sprite::HitlerW3),
    tic_time: 6,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE3S),
};

pub static S_HITLERCHASE3S: StateType = StateType {
    id: 11104,
    rotate: 0,
    sprite: Some(Sprite::HitlerW3),
    tic_time: 4,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE4),
};

pub static S_HITLERCHASE4: StateType = StateType {
    id: 11105,
    rotate: 0,
    sprite: Some(Sprite::HitlerW4),
    tic_time: 2,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_HITLERCHASE1),
};

pub static S_HITLERDEATHCAM_140: StateType = StateType {
    id: 11106,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE1_140),
};

pub static S_HITLERDEATHCAM_5: StateType = StateType {
    id: 11107,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE1_5),
};

pub static S_HITLERDIE1_140: StateType = StateType {
    id: 11108,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_HITLERDIE2_140),
};

pub static S_HITLERDIE1_5: StateType = StateType {
    id: 11109,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_HITLERDIE2_5),
};

pub static S_HITLERDIE2_140: StateType = StateType {
    id: 11110,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 140,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE3),
};

pub static S_HITLERDIE2_5: StateType = StateType {
    id: 11111,
    rotate: 0,
    sprite: Some(Sprite::HitlerW1),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE3),
};

pub static S_HITLERDIE3: StateType = StateType {
    id: 11112,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie1),
    tic_time: 10,
    think: None,
    action: Some(a_slurpie),
    async_action: None,
    next: Some(&S_HITLERDIE4),
};

pub static S_HITLERDIE4: StateType = StateType {
    id: 11113,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE5),
};

pub static S_HITLERDIE5: StateType = StateType {
    id: 11114,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE6),
};

pub static S_HITLERDIE6: StateType = StateType {
    id: 11115,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie4),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE7),
};

pub static S_HITLERDIE7: StateType = StateType {
    id: 11116,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie5),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE8),
};

pub static S_HITLERDIE8: StateType = StateType {
    id: 11117,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie6),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE9),
};

pub static S_HITLERDIE9: StateType = StateType {
    id: 11118,
    rotate: 0,
    sprite: Some(Sprite::HitlerDie7),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERDIE10),
};

pub static S_HITLERDIE10: StateType = StateType {
    id: 11119,
    rotate: 0,
    sprite: Some(Sprite::HitlerDead),
    tic_time: 20,
    think: None,
    action: None,
    async_action: Some(a_start_death_cam),
    next: Some(&S_HITLERDIE10),
};

pub static S_HITLERSHOOT1: StateType = StateType {
    id: 11120,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_HITLERSHOOT2),
};

pub static S_HITLERSHOOT2: StateType = StateType {
    id: 11121,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_HITLERSHOOT3),
};

pub static S_HITLERSHOOT3: StateType = StateType {
    id: 11122,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_HITLERSHOOT4),
};

pub static S_HITLERSHOOT4: StateType = StateType {
    id: 11123,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_HITLERSHOOT5),
};

pub static S_HITLERSHOOT5: StateType = StateType {
    id: 11124,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_HITLERSHOOT6),
};

pub static S_HITLERSHOOT6: StateType = StateType {
    id: 11125,
    rotate: 0,
    sprite: Some(Sprite::HitlerShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_HITLERCHASE1),
};

//
// gretel
//
pub static S_GRETELSTAND: StateType = StateType {
    id: 11200,
    rotate: 0,
    sprite: Some(Sprite::GretelW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_GRETELSTAND),
};

pub static S_GRETELCHASE1: StateType = StateType {
    id: 11201,
    rotate: 0,
    sprite: Some(Sprite::GretelW1),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE1S),
};

pub static S_GRETELCHASE1S: StateType = StateType {
    id: 11202,
    rotate: 0,
    sprite: Some(Sprite::GretelW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE2),
};

pub static S_GRETELCHASE2: StateType = StateType {
    id: 11203,
    rotate: 0,
    sprite: Some(Sprite::GretelW2),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE3),
};

pub static S_GRETELCHASE3: StateType = StateType {
    id: 11204,
    rotate: 0,
    sprite: Some(Sprite::GretelW3),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE3S),
};

pub static S_GRETELCHASE3S: StateType = StateType {
    id: 11205,
    rotate: 0,
    sprite: Some(Sprite::GretelW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE4),
};

pub static S_GRETELCHASE4: StateType = StateType {
    id: 11206,
    rotate: 0,
    sprite: Some(Sprite::GretelW4),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE1),
};

pub static S_GRETELDIE1: StateType = StateType {
    id: 11207,
    rotate: 0,
    sprite: Some(Sprite::GretelDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_GRETELDIE2),
};

pub static S_GRETELDIE2: StateType = StateType {
    id: 11208,
    rotate: 0,
    sprite: Some(Sprite::GretelDie2),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELDIE3),
};

pub static S_GRETELDIE3: StateType = StateType {
    id: 11209,
    rotate: 0,
    sprite: Some(Sprite::GretelDie3),
    tic_time: 15,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELDIE4),
};

pub static S_GRETELDIE4: StateType = StateType {
    id: 11210,
    rotate: 0,
    sprite: Some(Sprite::GretelDead),
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELDIE4),
};

pub static S_GRETELSHOOT1: StateType = StateType {
    id: 11211,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELSHOOT2),
};

pub static S_GRETELSHOOT2: StateType = StateType {
    id: 11212,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT3),
};

pub static S_GRETELSHOOT3: StateType = StateType {
    id: 11213,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT4),
};

pub static S_GRETELSHOOT4: StateType = StateType {
    id: 11214,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT5),
};

pub static S_GRETELSHOOT5: StateType = StateType {
    id: 11215,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT6),
};

pub static S_GRETELSHOOT6: StateType = StateType {
    id: 11216,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT7),
};

pub static S_GRETELSHOOT7: StateType = StateType {
    id: 11217,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_GRETELSHOOT8),
};

pub static S_GRETELSHOOT8: StateType = StateType {
    id: 11218,
    rotate: 0,
    sprite: Some(Sprite::GretelShoot1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_GRETELCHASE1),
};

//
// fat
//
pub static S_FATSTAND: StateType = StateType {
    id: 11300,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    async_action: None,
    next: Some(&S_FATSTAND),
};

pub static S_FATCHASE1: StateType = StateType {
    id: 11301,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 10,
    think: Some(t_fat),
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE1S),
};

pub static S_FATCHASE1S: StateType = StateType {
    id: 11302,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE2),
};

pub static S_FATCHASE2: StateType = StateType {
    id: 11303,
    rotate: 0,
    sprite: Some(Sprite::FatW2),
    tic_time: 8,
    think: Some(t_fat),
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE3),
};

pub static S_FATCHASE3: StateType = StateType {
    id: 11304,
    rotate: 0,
    sprite: Some(Sprite::FatW3),
    tic_time: 10,
    think: Some(t_fat),
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE3S),
};

pub static S_FATCHASE3S: StateType = StateType {
    id: 11305,
    rotate: 0,
    sprite: Some(Sprite::FatW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE4),
};

pub static S_FATCHASE4: StateType = StateType {
    id: 11306,
    rotate: 0,
    sprite: Some(Sprite::FatW4),
    tic_time: 8,
    think: Some(t_fat),
    action: None,
    async_action: None,
    next: Some(&S_FATCHASE1),
};

pub static S_FATDEATHCAM_140: StateType = StateType {
    id: 11307,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE1_140),
};

pub static S_FATDEATHCAM_5: StateType = StateType {
    id: 11308,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 1,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE1_5),
};

pub static S_FATDIE1_140: StateType = StateType {
    id: 11309,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_FATDIE2_140),
};

pub static S_FATDIE1_5: StateType = StateType {
    id: 11310,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 1,
    think: None,
    action: Some(a_death_scream),
    async_action: None,
    next: Some(&S_FATDIE2_5),
};

pub static S_FATDIE2_140: StateType = StateType {
    id: 11311,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 140,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE3),
};

pub static S_FATDIE2_5: StateType = StateType {
    id: 11312,
    rotate: 0,
    sprite: Some(Sprite::FatW1),
    tic_time: 5,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE3),
};

pub static S_FATDIE3: StateType = StateType {
    id: 11313,
    rotate: 0,
    sprite: Some(Sprite::FatDie1),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE4),
};

pub static S_FATDIE4: StateType = StateType {
    id: 11314,
    rotate: 0,
    sprite: Some(Sprite::FatDie2),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE5),
};

pub static S_FATDIE5: StateType = StateType {
    id: 11315,
    rotate: 0,
    sprite: Some(Sprite::FatDie3),
    tic_time: 10,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATDIE6),
};

pub static S_FATDIE6: StateType = StateType {
    id: 11316,
    rotate: 0,
    sprite: Some(Sprite::FatDead),
    tic_time: 20,
    think: None,
    action: None,
    async_action: Some(a_start_death_cam),
    next: Some(&S_FATDIE6),
};

pub static S_FATSHOOT1: StateType = StateType {
    id: 11317,
    rotate: 0,
    sprite: Some(Sprite::FatShoot1),
    tic_time: 30,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_FATSHOOT2),
};

pub static S_FATSHOOT2: StateType = StateType {
    id: 11318,
    rotate: 0,
    sprite: Some(Sprite::FatShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_gift_throw), //same throw as gift
    async_action: None,
    next: Some(&S_FATSHOOT3),
};

pub static S_FATSHOOT3: StateType = StateType {
    id: 11319,
    rotate: 0,
    sprite: Some(Sprite::FatShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_FATSHOOT4),
};

pub static S_FATSHOOT4: StateType = StateType {
    id: 11320,
    rotate: 0,
    sprite: Some(Sprite::FatShoot4),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_FATSHOOT5),
};

pub static S_FATSHOOT5: StateType = StateType {
    id: 11321,
    rotate: 0,
    sprite: Some(Sprite::FatShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_FATSHOOT6),
};

pub static S_FATSHOOT6: StateType = StateType {
    id: 11322,
    rotate: 0,
    sprite: Some(Sprite::FatShoot4),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    async_action: None,
    next: Some(&S_FATCHASE1),
};

pub static STATES: [&'static StateType; 304] = [
    &S_PLAYER,
    &S_ATTACK,
    &S_GRDSTAND,
    &S_GRDPATH1,
    &S_GRDPATH1S,
    &S_GRDPATH2,
    &S_GRDPATH3,
    &S_GRDPATH3S,
    &S_GRDPATH4,
    &S_GRDPAIN,
    &S_GRDPAIN1,
    &S_GRDSHOOT1,
    &S_GRDSHOOT2,
    &S_GRDSHOOT3,
    &S_GRDCHASE1,
    &S_GRDCHASE1S,
    &S_GRDCHASE2,
    &S_GRDCHASE3,
    &S_GRDCHASE3S,
    &S_GRDCHASE4,
    &S_GRDDIE1,
    &S_GRDDIE2,
    &S_GRDDIE3,
    &S_GRDDIE4,
    &S_BLINKYCHASE1,
    &S_BLINKYCHASE2,
    &S_INKYCHASE1,
    &S_INKYCHASE2,
    &S_PINKYCHASE1,
    &S_PINKYCHASE2,
    &S_CLYDECHASE1,
    &S_CLYDECHASE2,
    &S_DOGPATH1,
    &S_DOGPATH1S,
    &S_DOGPATH2,
    &S_DOGPATH3,
    &S_DOGPATH3S,
    &S_DOGPATH4,
    &S_DOGJUMP1,
    &S_DOGJUMP2,
    &S_DOGJUMP3,
    &S_DOGJUMP4,
    &S_DOGJUMP5,
    &S_DOGCHASE1,
    &S_DOGCHASE1S,
    &S_DOGCHASE2,
    &S_DOGCHASE3,
    &S_DOGCHASE3S,
    &S_DOGCHASE4,
    &S_DOGDIE1,
    &S_DOGDIE2,
    &S_DOGDIE3,
    &S_DOGDEAD,
    &S_OFCSTAND,
    &S_OFCPATH1,
    &S_OFCPATH1S,
    &S_OFCPATH2,
    &S_OFCPATH3,
    &S_OFCPATH3S,
    &S_OFCPATH4,
    &S_OFCPAIN,
    &S_OFCPAIN1,
    &S_OFCSHOOT1,
    &S_OFCSHOOT2,
    &S_OFCSHOOT3,
    &S_OFCCHASE1,
    &S_OFCCHASE1S,
    &S_OFCCHASE2,
    &S_OFCCHASE3,
    &S_OFCCHASE3S,
    &S_OFCCHASE4,
    &S_OFCDIE1,
    &S_OFCDIE2,
    &S_OFCDIE3,
    &S_OFCDIE4,
    &S_OFCDIE5,
    &S_MUTSTAND,
    &S_MUTPATH1,
    &S_MUTPATH1S,
    &S_MUTPATH2,
    &S_MUTPATH3,
    &S_MUTPATH3S,
    &S_MUTPATH4,
    &S_MUTPAIN,
    &S_MUTPAIN1,
    &S_MUTSHOOT1,
    &S_MUTSHOOT2,
    &S_MUTSHOOT3,
    &S_MUTSHOOT4,
    &S_MUTCHASE1,
    &S_MUTCHASE1S,
    &S_MUTCHASE2,
    &S_MUTCHASE3,
    &S_MUTCHASE3S,
    &S_MUTCHASE4,
    &S_MUTDIE1,
    &S_MUTDIE2,
    &S_MUTDIE3,
    &S_MUTDIE4,
    &S_MUTDIE5,
    &S_SSSTAND,
    &S_SSPATH1,
    &S_SSPATH1S,
    &S_SSPATH2,
    &S_SSPATH3,
    &S_SSPATH3S,
    &S_SSPATH4,
    &S_SSPAIN,
    &S_SSPAIN1,
    &S_SSSHOOT1,
    &S_SSSHOOT2,
    &S_SSSHOOT3,
    &S_SSSHOOT4,
    &S_SSSHOOT5,
    &S_SSSHOOT6,
    &S_SSSHOOT7,
    &S_SSSHOOT8,
    &S_SSSHOOT9,
    &S_SSCHASE1,
    &S_SSCHASE1S,
    &S_SSCHASE2,
    &S_SSCHASE3,
    &S_SSCHASE3S,
    &S_SSCHASE4,
    &S_SSDIE1,
    &S_SSDIE2,
    &S_SSDIE3,
    &S_SSDIE4,
    &S_BOSSSTAND,
    &S_BOSSCHASE1,
    &S_BOSSCHASE1S,
    &S_BOSSCHASE2,
    &S_BOSSCHASE3,
    &S_BOSSCHASE3S,
    &S_BOSSCHASE4,
    &S_BOSSDIE1,
    &S_BOSSDIE2,
    &S_BOSSDIE3,
    &S_BOSSDIE4,
    &S_BOSSSHOOT1,
    &S_BOSSSHOOT2,
    &S_BOSSSHOOT3,
    &S_BOSSSHOOT4,
    &S_BOSSSHOOT5,
    &S_BOSSSHOOT6,
    &S_BOSSSHOOT7,
    &S_BOSSSHOOT8,
    &S_SCHABBSTAND,
    &S_SCHABBCHASE1,
    &S_SCHABBCHASE1S,
    &S_SCHABBCHASE2,
    &S_SCHABBCHASE3,
    &S_SCHABBCHASE3S,
    &S_SCHABBCHASE4,
    &S_SCHABBDEATHCAM_140,
    &S_SCHABBDEATHCAM_10,
    &S_SCHABBDIE1_140,
    &S_SCHABBDIE1_10,
    &S_SCHABBDIE2_140,
    &S_SCHABBDIE2_10,
    &S_SCHABBDIE3,
    &S_SCHABBDIE4,
    &S_SCHABBDIE5,
    &S_SCHABBDIE6,
    &S_SCHABBSHOOT1,
    &S_SCHABBSHOOT2,
    &S_NEEDLE1,
    &S_NEEDLE2,
    &S_NEEDLE3,
    &S_NEEDLE4,
    &S_GIFTSTAND,
    &S_GIFTCHASE1,
    &S_GIFTCHASE1S,
    &S_GIFTCHASE2,
    &S_GIFTCHASE3,
    &S_GIFTCHASE3S,
    &S_GIFTCHASE4,
    &S_GIFTDIE1_140,
    &S_GIFTDIE1_5,
    &S_GIFTDIE2_140,
    &S_GIFTDIE2_5,
    &S_GIFTDIE3,
    &S_GIFTDIE4,
    &S_GIFTDIE5,
    &S_GIFTDIE6,
    &S_GIFTSHOOT1,
    &S_GIFTSHOOT2,
    &S_ROCKET,
    &S_SMOKE1,
    &S_SMOKE2,
    &S_SMOKE3,
    &S_SMOKE4,
    &S_BOOM1,
    &S_BOOM2,
    &S_BOOM3,
    &S_FAKESTAND,
    &S_FAKECHASE1,
    &S_FAKECHASE1S,
    &S_FAKECHASE2,
    &S_FAKECHASE3,
    &S_FAKECHASE3S,
    &S_FAKECHASE4,
    &S_FAKEDIE1,
    &S_FAKEDIE2,
    &S_FAKEDIE3,
    &S_FAKEDIE4,
    &S_FAKEDIE5,
    &S_FAKEDIE6,
    &S_FAKESHOOT1,
    &S_FAKESHOOT2,
    &S_FAKESHOOT3,
    &S_FAKESHOOT4,
    &S_FAKESHOOT5,
    &S_FAKESHOOT6,
    &S_FAKESHOOT7,
    &S_FAKESHOOT8,
    &S_FAKESHOOT9,
    &S_FIRE1,
    &S_FIRE2,
    &S_MECHASTAND,
    &S_MECHACHASE1,
    &S_MECHACHASE1S,
    &S_MECHACHASE2,
    &S_MECHACHASE3,
    &S_MECHACHASE3S,
    &S_MECHACHASE4,
    &S_MECHADIE1,
    &S_MECHADIE2,
    &S_MECHADIE3,
    &S_MECHADIE4,
    &S_MECHASHOOT1,
    &S_MECHASHOOT2,
    &S_MECHASHOOT3,
    &S_MECHASHOOT4,
    &S_MECHASHOOT5,
    &S_MECHASHOOT6,
    &S_HITLERCHASE1,
    &S_HITLERCHASE1S,
    &S_HITLERCHASE2,
    &S_HITLERCHASE3,
    &S_HITLERCHASE3S,
    &S_HITLERCHASE4,
    &S_HITLERDEATHCAM_140,
    &S_HITLERDEATHCAM_5,
    &S_HITLERDIE1_140,
    &S_HITLERDIE1_5,
    &S_HITLERDIE2_140,
    &S_HITLERDIE2_5,
    &S_HITLERDIE3,
    &S_HITLERDIE4,
    &S_HITLERDIE5,
    &S_HITLERDIE6,
    &S_HITLERDIE7,
    &S_HITLERDIE8,
    &S_HITLERDIE9,
    &S_HITLERDIE10,
    &S_HITLERSHOOT1,
    &S_HITLERSHOOT2,
    &S_HITLERSHOOT3,
    &S_HITLERSHOOT4,
    &S_HITLERSHOOT5,
    &S_HITLERSHOOT6,
    &S_GRETELSTAND,
    &S_GRETELCHASE1,
    &S_GRETELCHASE1S,
    &S_GRETELCHASE2,
    &S_GRETELCHASE3,
    &S_GRETELCHASE3S,
    &S_GRETELCHASE4,
    &S_GRETELDIE1,
    &S_GRETELDIE2,
    &S_GRETELDIE3,
    &S_GRETELDIE4,
    &S_GRETELSHOOT1,
    &S_GRETELSHOOT2,
    &S_GRETELSHOOT3,
    &S_GRETELSHOOT4,
    &S_GRETELSHOOT5,
    &S_GRETELSHOOT6,
    &S_GRETELSHOOT7,
    &S_GRETELSHOOT8,
    &S_FATSTAND,
    &S_FATCHASE1,
    &S_FATCHASE1S,
    &S_FATCHASE2,
    &S_FATCHASE3,
    &S_FATCHASE3S,
    &S_FATCHASE4,
    &S_FATDEATHCAM_140,
    &S_FATDEATHCAM_5,
    &S_FATDIE1_140,
    &S_FATDIE1_5,
    &S_FATDIE2_140,
    &S_FATDIE2_5,
    &S_FATDIE3,
    &S_FATDIE4,
    &S_FATDIE5,
    &S_FATDIE6,
    &S_FATSHOOT1,
    &S_FATSHOOT2,
    &S_FATSHOOT3,
    &S_FATSHOOT4,
    &S_FATSHOOT5,
    &S_FATSHOOT6,
];

pub fn get_state_by_id(id: u16) -> Option<&'static StateType> {
    for s in &STATES {
        if s.id == id {
            return Some(&s);
        }
    }
    None
}

fn t_projectile(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let player_x = level_state.player().x;
    let player_y = level_state.player().y;

    let speed = Fixed::new_from_i32(level_state.obj(k).speed * tics as i32);
    {
        let mut delta_x =
            fixed_by_frac(speed, rc.projection.cos(level_state.obj(k).angle as usize)).to_i32();
        let mut delta_y =
            -fixed_by_frac(speed, rc.projection.sin(level_state.obj(k).angle as usize)).to_i32();

        if delta_x > 0x10000 {
            delta_x = 0x10000;
        }
        if delta_y > 0x10000 {
            delta_y = 0x10000;
        }

        level_state.mut_obj(k).x += delta_x;
        level_state.mut_obj(k).y += delta_y;
    }

    let delta_player_x = level_state.obj(k).x.abs_diff(player_x);
    let delta_player_y = level_state.obj(k).y.abs_diff(player_y);

    if !projectile_try_move(k, level_state) {
        if level_state.obj(k).class == ClassType::Rocket {
            rc.play_sound_loc_actor(SoundName::MISSILEHIT, level_state.obj(k));
            level_state.mut_obj(k).state = Some(&S_BOOM1);
        } else {
            level_state.mut_obj(k).state = None; // mark for removal
        }
        return;
    }

    if delta_player_x < PROJECTILE_SIZE && delta_player_y < PROJECTILE_SIZE {
        // hit the player
        let damage = match level_state.obj(k).class {
            ClassType::Needle => (rnd_t() >> 3) + 20,
            ClassType::Rocket | ClassType::HRocket | ClassType::Spark => (rnd_t() >> 3) + 30,
            ClassType::Fire => rnd_t() >> 3,
            _ => 0,
        } as i32;

        take_damage(rc, k, damage, level_state, game_state);
        level_state.mut_obj(k).state = None; // mark for removal
        return;
    }

    let obj = level_state.mut_obj(k);
    obj.tilex = (obj.x >> TILESHIFT) as usize;
    obj.tiley = (obj.y >> TILESHIFT) as usize;
}

fn projectile_try_move(k: ObjKey, level_state: &LevelState) -> bool {
    let ob = level_state.obj(k);

    let xl = (ob.x - PROJ_SIZE) >> TILESHIFT;
    let yl = (ob.y - PROJ_SIZE) >> TILESHIFT;
    let xh = (ob.x + PROJ_SIZE) >> TILESHIFT;
    let yh = (ob.y + PROJ_SIZE) >> TILESHIFT;

    // check for solid walls
    for y in yl..=yh {
        for x in xl..=xh {
            if let At::Wall(_) = level_state.actor_at[x as usize][y as usize] {
                return false;
            }
        }
    }

    true
}

fn a_smoke(
    _: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let tile_x = level_state.obj(k).tilex;
    let tile_y = level_state.obj(k).tiley;
    let mut obj = spawn_new_obj(
        &mut level_state.level.map_segs,
        tile_x,
        tile_y,
        &S_SMOKE1,
        ClassType::Inert,
    );
    obj.tic_count = 6;
    obj.x = level_state.obj(k).x;
    obj.y = level_state.obj(k).y;
    obj.active = ActiveType::Yes;
    obj.flags = FL_NEVERMARK;

    level_state.actors.add_obj(obj);
}

fn t_path(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    if sight_player(rc, k, level_state, game_state, tics) {
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
        if dist < 0 {
            // waiting for a door to open
            let door = &mut level_state.doors[(-dist - 1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return;
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL);
        }

        let dist = level_state.obj(k).distance;
        if mov < dist {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        if level_state.obj(k).tilex > MAP_SIZE || level_state.obj(k).tiley > MAP_SIZE {
            panic!(
                "T_Path hit a wall at {},{}, dir {:?}",
                level_state.obj(k).tilex,
                level_state.obj(k).tiley,
                level_state.obj(k).dir
            );
        }

        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
            mov -= obj.distance;
        });

        select_path_dir(k, level_state);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_dog_chase(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
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
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving

        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
            mov -= obj.distance;
        });

        let (player_tile_x, player_tile_y) = {
            let player = level_state.player();
            (player.tilex, player.tiley)
        };
        select_dodge_dir(k, level_state, player_tile_x, player_tile_y);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_bite(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    rc.play_sound(SoundName::DOGATTACK);

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
                take_damage(rc, k, (rnd_t() >> 4) as i32, level_state, game_state);
                return;
            }
        }
    }
}

fn select_path_dir(k: ObjKey, level_state: &mut LevelState) {
    let spot = level_state.level.info_map[level_state.obj(k).tilex][level_state.obj(k).tiley]
        .wrapping_sub(ICON_ARROWS);
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

fn t_stand(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    sight_player(rc, k, level_state, game_state, tics);
}

fn t_chase(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    if game_state.victory_flag {
        return;
    }

    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    let mut dodge = false;
    if check_line(level_state, level_state.obj(k)) {
        // got a shot at player?
        let obj = level_state.obj(k);
        let player = level_state.player();
        let dx = obj.tilex.abs_diff(player.tilex);
        let dy = obj.tiley.abs_diff(player.tiley);
        let dist = dx.max(dy);
        let chance = if dist == 0 || (dist == 1) && obj.distance < 0x4000 {
            300 // always hit
        } else {
            ((tics as usize) << 4) / dist
        };

        if (rnd_t() as usize) < chance {
            // go into attack frame
            let state_change = match obj.class {
                ClassType::Guard => Some(&S_GRDSHOOT1),
                ClassType::Officer => Some(&S_OFCSHOOT1),
                ClassType::Mutant => Some(&S_MUTSHOOT1),
                ClassType::SS => Some(&S_SSSHOOT1),
                ClassType::Boss => Some(&S_BOSSSHOOT1),
                ClassType::Gretel => Some(&S_GRETELSHOOT1),
                ClassType::MechaHitler => Some(&S_MECHASHOOT1),
                ClassType::RealHitler => Some(&S_HITLERSHOOT1),
                _ => panic!("impl state change for {:?}", obj.class),
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
            let door = &mut level_state.doors[(-distance - 1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return;
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL) // go ahead, the door is now opoen
        }

        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
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
            return; // object is blocked in
        }
    }
}

fn t_ghosts(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if level_state.obj(k).dir == DirType::NoDir {
        select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        if level_state.obj(k).dir == DirType::NoDir {
            return; // object is blocked in
        }
    }

    let mut mov = level_state.obj(k).speed * tics as i32;
    while mov != 0 {
        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        });

        mov -= level_state.obj(k).distance;

        select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        if level_state.obj(k).dir == DirType::NoDir {
            return; // object is blocked in
        }
    }
}

fn t_schabb(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let mut dodge = false;
    let dist = {
        let obj = level_state.obj(k);
        let player = level_state.player();
        let dx = obj.tilex.abs_diff(player.tilex);
        let dy = obj.tiley.abs_diff(player.tiley);
        let dist = if dx > dy { dx } else { dy };
        dist
    };

    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if check_line(level_state, level_state.obj(k)) {
        if (rnd_t() as u64) < (tics << 3) {
            // go into attack frame
            let mut_obj = level_state.mut_obj(k);
            new_state(mut_obj, &S_SCHABBSHOOT1);
            return;
        }
        dodge = true;
    }

    if level_state.obj(k).dir == DirType::NoDir {
        if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }

    let obj = level_state.obj(k);
    let mut mov = obj.speed * tics as i32;
    while mov != 0 {
        let distance = level_state.obj(k).distance;
        if distance < 0 {
            // waiting for a door to open
            let door = &mut level_state.doors[(-distance - 1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return;
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL) // go ahead, the door is now opoen
        }

        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        });

        mov -= level_state.obj(k).distance;

        if dist < 4 {
            select_run_dir(k, level_state, player_tile_x, player_tile_y);
        } else if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_schabb_throw(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let player = level_state.player();
    let delta_x = player.x - level_state.obj(k).x;
    let delta_y = level_state.obj(k).y - player.y;

    let mut angle = (delta_y as f64).atan2(delta_x as f64);
    if angle < 0.0 {
        angle = std::f64::consts::PI * 2.0 + angle;
    }
    let iangle = ((angle / (std::f64::consts::PI * 2.0)) * ANGLES_F64) as i32;

    let tile_x = level_state.obj(k).tilex;
    let tile_y = level_state.obj(k).tiley;
    let mut obj = spawn_new_obj(
        &mut level_state.level.map_segs,
        tile_x,
        tile_y,
        &S_NEEDLE1,
        ClassType::Needle,
    );

    obj.tic_count = 1;
    obj.x = level_state.obj(k).x;
    obj.y = level_state.obj(k).y;
    obj.dir = DirType::NoDir;
    obj.angle = iangle;
    obj.speed = 0x2000;
    obj.flags = FL_NONMARK;
    obj.active = ActiveType::Yes;

    level_state.actors.add_obj(obj);

    rc.play_sound_loc_actor(SoundName::SCHABBSTHROW, &obj);
}

fn t_gift(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let mut dodge = false;
    let dist = {
        let obj = level_state.obj(k);
        let player = level_state.player();
        let dx = obj.tilex.abs_diff(player.tilex);
        let dy = obj.tiley.abs_diff(player.tiley);
        let dist = if dx > dy { dx } else { dy };
        dist
    };

    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if check_line(level_state, level_state.obj(k)) {
        if (rnd_t() as u64) < (tics << 3) {
            // go into attack frame
            let mut_obj = level_state.mut_obj(k);
            new_state(mut_obj, &S_GIFTSHOOT1);
            return;
        }
        dodge = true;
    }

    if level_state.obj(k).dir == DirType::NoDir {
        if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }

    let obj = level_state.obj(k);
    let mut mov = obj.speed * tics as i32;
    while mov != 0 {
        let distance = level_state.obj(k).distance;
        if distance < 0 {
            // waiting for a door to open
            let door = &mut level_state.doors[(-distance - 1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return;
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL) // go ahead, the door is now opoen
        }

        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        });

        mov -= level_state.obj(k).distance;

        if dist < 4 {
            select_run_dir(k, level_state, player_tile_x, player_tile_y);
        } else if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_fat(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let mut dodge = false;
    let dist = {
        let obj = level_state.obj(k);
        let player = level_state.player();
        let dx = obj.tilex.abs_diff(player.tilex);
        let dy = obj.tiley.abs_diff(player.tiley);
        let dist = if dx > dy { dx } else { dy };
        dist
    };

    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if check_line(level_state, level_state.obj(k)) {
        if (rnd_t() as u64) < (tics << 3) {
            // go into attack frame
            let mut_obj = level_state.mut_obj(k);
            new_state(mut_obj, &S_FATSHOOT1);
            return;
        }
        dodge = true;
    }

    if level_state.obj(k).dir == DirType::NoDir {
        if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }

    let obj = level_state.obj(k);
    let mut mov = obj.speed * tics as i32;
    while mov != 0 {
        let distance = level_state.obj(k).distance;
        if distance < 0 {
            // waiting for a door to open
            let door = &mut level_state.doors[(-distance - 1) as usize];
            open_door(door);
            if door.action != DoorAction::Open {
                return;
            }
            level_state.update_obj(k, |obj| obj.distance = TILEGLOBAL) // go ahead, the door is now opoen
        }

        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        });

        mov -= level_state.obj(k).distance;

        if dist < 4 {
            select_run_dir(k, level_state, player_tile_x, player_tile_y);
        } else if dodge {
            select_dodge_dir(k, level_state, player_tile_x, player_tile_y);
        } else {
            select_chase_dir(k, level_state, player_tile_x, player_tile_y);
        }

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_gift_throw(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let player = level_state.player();
    let delta_x = player.x - level_state.obj(k).x;
    let delta_y = level_state.obj(k).y - player.y;

    let mut angle = (delta_y as f64).atan2(delta_x as f64);
    if angle < 0.0 {
        angle = std::f64::consts::PI * 2.0 + angle;
    }
    let iangle = ((angle / (std::f64::consts::PI * 2.0)) * ANGLES_F64) as i32;

    let tile_x = level_state.obj(k).tilex;
    let tile_y = level_state.obj(k).tiley;
    let mut obj = spawn_new_obj(
        &mut level_state.level.map_segs,
        tile_x,
        tile_y,
        &S_ROCKET,
        ClassType::Rocket,
    );

    obj.tic_count = 1;
    obj.x = level_state.obj(k).x;
    obj.y = level_state.obj(k).y;
    obj.dir = DirType::NoDir;
    obj.angle = iangle;
    obj.speed = 0x2000;
    obj.flags = FL_NONMARK;
    obj.active = ActiveType::Yes;

    level_state.actors.add_obj(obj);

    rc.play_sound_loc_actor(SoundName::MISSILEFIRE, &obj);
}

fn t_fake(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let (player_tile_x, player_tile_y) = {
        let player = level_state.player();
        (player.tilex, player.tiley)
    };

    if check_line(level_state, level_state.obj(k)) {
        if (rnd_t() as u64) < (tics << 1) {
            // go into attack frame
            let mut_obj = level_state.mut_obj(k);
            new_state(mut_obj, &S_FAKESHOOT1);
            return;
        }
    }

    if level_state.obj(k).dir == DirType::NoDir {
        select_dodge_dir(k, level_state, player_tile_x, player_tile_y);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }

    let obj = level_state.obj(k);
    let mut mov = obj.speed * tics as i32;
    while mov != 0 {
        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving
        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
        });

        mov -= level_state.obj(k).distance;

        select_dodge_dir(k, level_state, player_tile_x, player_tile_y);

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_fake_fire(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let player = level_state.player();
    let delta_x = player.x - level_state.obj(k).x;
    let delta_y = level_state.obj(k).y - player.y;

    let mut angle = (delta_y as f64).atan2(delta_x as f64);
    if angle < 0.0 {
        angle = std::f64::consts::PI * 2.0 + angle;
    }
    let iangle = ((angle / (std::f64::consts::PI * 2.0)) * ANGLES_F64) as i32;

    let tile_x = level_state.obj(k).tilex;
    let tile_y = level_state.obj(k).tiley;
    let mut obj = spawn_new_obj(
        &mut level_state.level.map_segs,
        tile_x,
        tile_y,
        &S_FIRE1,
        ClassType::Fire,
    );

    obj.tic_count = 1;
    obj.x = level_state.obj(k).x;
    obj.y = level_state.obj(k).y;
    obj.dir = DirType::NoDir;
    obj.angle = iangle;
    obj.speed = 0x1200;
    obj.flags = FL_NEVERMARK;
    obj.active = ActiveType::Yes;

    level_state.actors.add_obj(obj);

    rc.play_sound_loc_actor(SoundName::FLAMETHROWER, &obj);
}

fn a_hitler_morph(
    _: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let tile_x = level_state.obj(k).tilex;
    let tile_y = level_state.obj(k).tiley;
    let mut real_hitler = spawn_new_obj(
        &mut level_state.level.map_segs,
        tile_x,
        tile_y,
        &S_HITLERCHASE1,
        ClassType::RealHitler,
    );
    let obj = level_state.obj(k);
    real_hitler.speed = SPD_PATROL * 5;
    real_hitler.x = obj.x;
    real_hitler.y = obj.y;

    real_hitler.distance = obj.distance;
    real_hitler.dir = obj.dir;
    real_hitler.flags = obj.flags | FL_SHOOTABLE;
    real_hitler.hitpoints = match game_state.difficulty {
        Difficulty::Baby => 500,
        Difficulty::Easy => 700,
        Difficulty::Medium => 800,
        Difficulty::Hard => 900,
    };

    level_state.actors.add_obj(real_hitler);
}

fn a_mecha_sound(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let obj = level_state.obj(k);
    if level_state.area_by_player[obj.area_number] {
        rc.play_sound_loc_actor(SoundName::MECHSTEP, obj);
    }
}

fn a_slurpie(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    rc.play_sound_loc_actor(SoundName::SLURPIE, level_state.obj(k));
}

pub fn spawn_dead_guard(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    x_tile: usize,
    y_tile: usize,
) {
    let obj = spawn_new_obj(map_data, x_tile, y_tile, &S_GRDDIE4, ClassType::Inert);
    spawn(actors, actor_at, obj)
}

pub fn spawn_stand(
    tile_map: &mut Vec<Vec<u16>>,
    map_data: &mut MapSegs,
    which: EnemyType,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
    tile_dir: u16,
    difficulty: Difficulty,
) {
    let mut stand = match which {
        EnemyType::Guard => spawn_new_obj(map_data, x_tile, y_tile, &S_GRDSTAND, ClassType::Guard),
        EnemyType::Officer => {
            spawn_new_obj(map_data, x_tile, y_tile, &S_OFCSTAND, ClassType::Officer)
        }
        EnemyType::Mutant => {
            spawn_new_obj(map_data, x_tile, y_tile, &S_MUTSTAND, ClassType::Mutant)
        }
        EnemyType::SS => spawn_new_obj(map_data, x_tile, y_tile, &S_SSSTAND, ClassType::SS),
        _ => {
            panic!("illegal stand enemy type: {:?}", which)
        }
    };
    stand.speed = SPD_PATROL;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    let map_ptr = y_tile * MAP_SIZE + x_tile;
    let map = map_data.segs[0][map_ptr];
    if map == AMBUSH_TILE {
        tile_map[x_tile][y_tile] = 0;

        let mut tile = 0;
        if map_data.segs[0][map_ptr + 1] >= AREATILE {
            tile = map_data.segs[0][map_ptr + 1];
        }
        if map_data.segs[0][map_ptr - MAP_SIZE] >= AREATILE {
            tile = map_data.segs[0][map_ptr - MAP_SIZE];
        }
        if map_data.segs[0][map_ptr + MAP_SIZE] >= AREATILE {
            tile = map_data.segs[0][map_ptr + MAP_SIZE];
        }
        if map_data.segs[0][map_ptr - 1] >= AREATILE {
            tile = map_data.segs[0][map_ptr - 1];
        }
        map_data.segs[0][map_ptr] = tile;
        stand.area_number = (tile - AREATILE) as usize;

        stand.flags |= FL_AMBUSH;
    }

    stand.hitpoints = START_HITPOINTS[difficulty as usize][which as usize];
    stand.dir = dir_type(tile_dir * 2);
    stand.flags |= FL_SHOOTABLE;

    spawn(actors, actor_at, stand);
}

pub fn spawn_boss(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut boss = spawn_new_obj(map_data, x_tile, y_tile, &S_BOSSSTAND, ClassType::Boss);
    boss.speed = SPD_PATROL;
    boss.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Boss as usize];
    boss.dir = DirType::South;
    boss.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, boss);
}

pub fn spawn_gretel(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut gretel = spawn_new_obj(map_data, x_tile, y_tile, &S_GRETELSTAND, ClassType::Gretel);
    gretel.speed = SPD_PATROL;
    gretel.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Gretel as usize];
    gretel.dir = DirType::North;
    gretel.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, gretel);
}

pub fn spawn_schabbs(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut schabb = spawn_new_obj(map_data, x_tile, y_tile, &S_SCHABBSTAND, ClassType::Schabb);
    schabb.speed = SPD_PATROL;
    schabb.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Schabbs as usize];
    schabb.dir = DirType::South;
    schabb.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, schabb);
}

pub fn spawn_gift(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut gift = spawn_new_obj(map_data, x_tile, y_tile, &S_GIFTSTAND, ClassType::Gift);
    gift.speed = SPD_PATROL;
    gift.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Gift as usize];
    gift.dir = DirType::North;
    gift.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, gift);
}

pub fn spawn_fat(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut fat = spawn_new_obj(map_data, x_tile, y_tile, &S_FATSTAND, ClassType::Fat);
    fat.speed = SPD_PATROL;
    fat.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Fat as usize];
    fat.dir = DirType::South;
    fat.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, fat);
}

pub fn spawn_fake_hitler(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut fake = spawn_new_obj(map_data, x_tile, y_tile, &S_FAKESTAND, ClassType::Fake);
    fake.speed = SPD_PATROL;
    fake.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Fake as usize];
    fake.dir = DirType::North;
    fake.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, fake);
}

pub fn spawn_hitler(
    map_data: &MapSegs,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut hitler = spawn_new_obj(
        map_data,
        x_tile,
        y_tile,
        &S_MECHASTAND,
        ClassType::MechaHitler,
    );
    hitler.speed = SPD_PATROL;
    hitler.hitpoints = START_HITPOINTS[game_state.difficulty as usize][EnemyType::Hitler as usize];
    hitler.dir = DirType::South;
    hitler.flags = FL_SHOOTABLE | FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, hitler);
}

pub fn spawn_patrol(
    map_data: &MapSegs,
    which: EnemyType,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
    tile_dir: u16,
    difficulty: Difficulty,
) {
    let mut patrol = match which {
        EnemyType::Guard => {
            let mut obj = spawn_new_obj(map_data, x_tile, y_tile, &S_GRDPATH1, ClassType::Guard);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        }
        EnemyType::Officer => {
            let mut obj = spawn_new_obj(map_data, x_tile, y_tile, &S_OFCPATH1, ClassType::Officer);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        }
        EnemyType::SS => {
            let mut obj = spawn_new_obj(map_data, x_tile, y_tile, &S_SSPATH1, ClassType::SS);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        }
        EnemyType::Mutant => {
            let mut obj = spawn_new_obj(map_data, x_tile, y_tile, &S_MUTPATH1, ClassType::Mutant);
            obj.speed = SPD_PATROL;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        }
        EnemyType::Dog => {
            let mut obj = spawn_new_obj(map_data, x_tile, y_tile, &S_DOGPATH1, ClassType::Dog);
            obj.speed = SPD_DOG;
            if !game_state.loaded_game {
                game_state.kill_total += 1;
            }
            obj
        }
        _ => {
            panic!("illegal stand enemy type: {:?}", which)
        }
    };

    patrol.dir = dir_type(tile_dir * 2);
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
        _ => { /* do nothing */ }
    }
    spawn(actors, actor_at, patrol);
}

pub fn spawn_ghosts(
    map_data: &MapSegs,
    which: EnemyType,
    actors: &mut Actors,
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    x_tile: usize,
    y_tile: usize,
) {
    let mut ghost = match which {
        EnemyType::Blinky => {
            spawn_new_obj(map_data, x_tile, y_tile, &S_BLINKYCHASE1, ClassType::Ghost)
        }
        EnemyType::Clyde => {
            spawn_new_obj(map_data, x_tile, y_tile, &S_CLYDECHASE1, ClassType::Ghost)
        }
        EnemyType::Pinky => {
            spawn_new_obj(map_data, x_tile, y_tile, &S_PINKYCHASE1, ClassType::Ghost)
        }
        EnemyType::Inky => spawn_new_obj(map_data, x_tile, y_tile, &S_INKYCHASE1, ClassType::Ghost),
        _ => quit(Some("not a ghost")),
    };

    ghost.speed = SPD_DOG;
    ghost.dir = DirType::East;
    ghost.flags |= FL_AMBUSH;
    if !game_state.loaded_game {
        game_state.kill_total += 1;
    }

    spawn(actors, actor_at, ghost);
}

// spawns the obj into the map. At map load time
fn spawn(actors: &mut Actors, actor_at: &mut Vec<Vec<At>>, obj: ObjType) {
    let key = actors.add_obj(obj);
    actor_at[obj.tilex][obj.tiley] = At::Obj(key)
}

/*
=============================================================================
                                FIGHT
=============================================================================
*/

/// Try to damage the player, based on skill level and player's speed
fn t_shoot(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let obj = level_state.obj(k);
    if !level_state.area_by_player[obj.area_number] {
        return;
    }

    let player = level_state.player();
    if !check_line(&level_state, obj) {
        // player is behind a wall
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
    if (rnd_t() as i32) < hit_chance {
        let damage = if dist < 2 {
            rnd_t() >> 2
        } else if dist < 4 {
            rnd_t() >> 3
        } else {
            rnd_t() >> 4
        };

        take_damage(rc, k, damage as i32, level_state, game_state)
    }

    let obj = level_state.obj(k);
    match obj.class {
        ClassType::SS => {
            rc.play_sound(SoundName::SSFIRE);
        }
        ClassType::Gift | ClassType::Fat => {
            rc.play_sound(SoundName::MISSILEFIRE);
        }
        ClassType::MechaHitler | ClassType::RealHitler | ClassType::Boss => {
            rc.play_sound(SoundName::BOSSFIRE);
        }
        ClassType::Schabb => {
            rc.play_sound(SoundName::SCHABBSTHROW);
        }
        ClassType::Fake => {
            rc.play_sound(SoundName::FLAMETHROWER);
        }
        _ => {
            rc.play_sound(SoundName::NAZIFIRE);
        }
    }
}

fn a_death_scream(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    do_death_scream(rc, k, level_state, game_state);
}

pub fn do_death_scream(
    rc: &mut RenderContext,
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
) {
    let obj = level_state.obj(k);
    if game_state.map_on == 9 && rnd_t() == 0 {
        match obj.class {
            ClassType::Mutant
            | ClassType::Guard
            | ClassType::Officer
            | ClassType::SS
            | ClassType::Dog => {
                rc.play_sound_loc_actor(SoundName::DEATHSCREAM6, obj);
                return;
            }
            _ => { /* play nothing */ }
        }
    }

    match obj.class {
        ClassType::Mutant => {
            rc.play_sound_loc_actor(SoundName::AHHHG, obj);
        }
        ClassType::Guard => {
            let scream_ix = if rc.variant.id == W3D1.id {
                (rnd_t() % 2) as usize
            } else {
                (rnd_t() % 8) as usize
            };
            rc.play_sound_loc_actor(GUARD_DEATH_SCREAMS[scream_ix], obj);
        }
        ClassType::Officer => {
            rc.play_sound_loc_actor(SoundName::NEINSOVAS, obj);
        }
        ClassType::SS => {
            rc.play_sound_loc_actor(SoundName::LEBEN, obj);
        }
        ClassType::Dog => {
            rc.play_sound_loc_actor(SoundName::DOGDEATH, obj);
        }
        ClassType::Boss => {
            rc.play_sound(SoundName::MUTTI);
        }
        ClassType::Schabb => {
            rc.play_sound(SoundName::MEINGOTT);
        }
        ClassType::Fake => {
            rc.play_sound(SoundName::HITLERHA);
        }
        ClassType::MechaHitler => {
            rc.play_sound(SoundName::SCHEIST);
        }
        ClassType::RealHitler => {
            rc.play_sound(SoundName::EVA);
        }
        ClassType::Gretel => {
            rc.play_sound(SoundName::MEIN);
        }
        ClassType::Gift => {
            rc.play_sound(SoundName::DONNER);
        }
        ClassType::Fat => {
            rc.play_sound(SoundName::ROSE);
        }
        _ => { /* ignore */ }
    }
}

/*
============================================================================
                            BJ VICTORY
============================================================================
*/

pub static S_BJRUN1: StateType = StateType {
    id: 9000,
    rotate: 0,
    sprite: Some(Sprite::BJW1),
    tic_time: 12,
    think: Some(t_bj_run),
    action: None,
    async_action: None,
    next: Some(&S_BJRUN1S),
};

pub static S_BJRUN1S: StateType = StateType {
    id: 9001,
    rotate: 0,
    sprite: Some(Sprite::BJW1),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BJRUN2),
};

pub static S_BJRUN2: StateType = StateType {
    id: 9002,
    rotate: 0,
    sprite: Some(Sprite::BJW2),
    tic_time: 8,
    think: Some(t_bj_run),
    action: None,
    async_action: None,
    next: Some(&S_BJRUN3),
};

pub static S_BJRUN3: StateType = StateType {
    id: 9003,
    rotate: 0,
    sprite: Some(Sprite::BJW3),
    tic_time: 12,
    think: Some(t_bj_run),
    action: None,
    async_action: None,
    next: Some(&S_BJRUN3S),
};

pub static S_BJRUN3S: StateType = StateType {
    id: 9004,
    rotate: 0,
    sprite: Some(Sprite::BJW3),
    tic_time: 3,
    think: None,
    action: None,
    async_action: None,
    next: Some(&S_BJRUN4),
};

pub static S_BJRUN4: StateType = StateType {
    id: 9005,
    rotate: 0,
    sprite: Some(Sprite::BJW4),
    tic_time: 8,
    think: Some(t_bj_run),
    action: None,
    async_action: None,
    next: Some(&S_BJRUN1),
};

pub static S_BJ_JUMP1: StateType = StateType {
    id: 9006,
    rotate: 0,
    sprite: Some(Sprite::BJJump1),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: None,
    async_action: None,
    next: Some(&S_BJ_JUMP2),
};

pub static S_BJ_JUMP2: StateType = StateType {
    id: 9007,
    rotate: 0,
    sprite: Some(Sprite::BJJump2),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: Some(t_bj_yell),
    async_action: None,
    next: Some(&S_BJ_JUMP3),
};

pub static S_BJ_JUMP3: StateType = StateType {
    id: 9008,
    rotate: 0,
    sprite: Some(Sprite::BJJump3),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: None,
    async_action: None,
    next: Some(&S_BJ_JUMP4),
};

pub static S_BJ_JUMP4: StateType = StateType {
    id: 9009,
    rotate: 0,
    sprite: Some(Sprite::BJJump4),
    tic_time: 300,
    think: None,
    action: Some(t_bj_done),
    async_action: None,
    next: Some(&S_BJ_JUMP4),
};

pub static S_DEATH_CAM: StateType = StateType {
    id: 9010,
    rotate: 0,
    sprite: None,
    tic_time: 0,
    think: None,
    action: None,
    async_action: None,
    next: None,
};

pub fn spawn_bj_victory(level_state: &mut LevelState) {
    let player = level_state.player();
    let mut bj = spawn_new_obj(
        &level_state.level.map_segs,
        player.tilex,
        player.tiley + 1,
        &S_BJRUN1,
        ClassType::BJ,
    );
    bj.x = player.x;
    bj.y = player.y;
    bj.dir = DirType::North;
    bj.temp1 = 6; // tiles to run forward
    spawn(&mut level_state.actors, &mut level_state.actor_at, bj);
}

fn t_bj_run(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let mut mov = BJ_RUN_SPEED * tics as i32;
    while mov > 0 {
        if mov < level_state.obj(k).distance {
            move_obj(rc, k, level_state, game_state, mov, tics);
            break;
        }
        {
            let obj = level_state.mut_obj(k);
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
            mov -= obj.distance;
        }

        select_path_dir(k, level_state);

        let obj = level_state.mut_obj(k);
        obj.temp1 -= 1;
        if obj.temp1 <= 0 {
            new_state(obj, &S_BJ_JUMP1);
            return;
        }
    }
}

fn t_bj_jump(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    let mov = BJ_JUMP_SPEED * tics as i32;
    move_obj(rc, k, level_state, game_state, mov, tics);
}

fn t_bj_yell(
    rc: &mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    _: &mut GameState,
    _: &mut ControlState,
) {
    let obj = level_state.obj(k);
    rc.play_sound_loc_actor(SoundName::YEAH, obj);
}

fn t_bj_done(
    _: &mut RenderContext,
    _: ObjKey,
    _: u64,
    _: &mut LevelState,
    game_state: &mut GameState,
    _: &mut ControlState,
) {
    game_state.play_state = PlayState::Victorious;
}

fn a_start_death_cam<'a>(
    rc: &'a mut RenderContext,
    k: ObjKey,
    _: u64,
    level_state: &'a mut LevelState,
    game_state: &'a mut GameState,
    _: &'a mut ControlState,
) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
    Box::pin(async move {
        finish_palette_shifts(game_state, &mut rc.vga);

        if game_state.victory_flag {
            game_state.play_state = PlayState::Victorious;
            return;
        }
        game_state.victory_flag = true;

        rc.bar(0, 0, 320, 200 - STATUS_LINES, 127);
        rc.fizzle_fade(
            rc.buffer_offset(),
            rc.active_buffer(),
            320,
            200 - STATUS_LINES,
            70,
            FizzleFadeAbortable::No,
        );
        rc.set_buffer_offset(rc.active_buffer());

        write(rc, 0, 7, "Let's see that again!");
        rc.wait_user_input(300).await;

        // line angle up exactly
        new_state(level_state.mut_player(), &S_DEATH_CAM);
        level_state.mut_player().x = game_state.kill_x as i32;
        level_state.mut_player().y = game_state.kill_y as i32;

        let dx = level_state.obj(k).x - level_state.player().x;
        let dy = level_state.player().y - level_state.obj(k).y;

        let mut fangle = (dy as f64).atan2(dx as f64);
        if fangle < 0.0 {
            fangle = std::f64::consts::PI * 2.0 + fangle;
        }
        let angle = ((fangle / (std::f64::consts::PI * 2.0)) * ANGLES_F64) as i32;
        level_state.mut_player().angle = angle;
        // try to position as close as possible without being in a wall
        let mut dist = 0x14000;
        loop {
            let x_move =
                fixed_by_frac(Fixed::new_from_u32(dist), rc.projection.cos(angle as usize));
            let y_move =
                -fixed_by_frac(Fixed::new_from_u32(dist), rc.projection.sin(angle as usize));

            level_state.mut_player().x = level_state.obj(k).x - x_move.to_i32();
            level_state.mut_player().y = level_state.obj(k).y - y_move.to_i32();
            dist += 0x1000;

            if check_position_player(level_state) {
                break;
            }
        }
        level_state.mut_player().tilex = (level_state.player().x >> TILESHIFT) as usize;
        level_state.mut_player().tiley = (level_state.player().y >> TILESHIFT) as usize;

        // go back to the game
        let offset_prev = rc.buffer_offset();
        for i in 0..3 {
            rc.set_buffer_offset(SCREENLOC[i]);
            draw_play_border(rc, rc.projection.view_width, rc.projection.view_height);
        }
        rc.set_buffer_offset(offset_prev);

        game_state.fizzle_in = true;
        let obj = level_state.mut_obj(k);
        match obj.class {
            ClassType::Schabb => {
                if rc.sound.digi_mode() != DigiMode::Off {
                    new_state(level_state.mut_obj(k), &S_SCHABBDEATHCAM_140);
                } else {
                    new_state(level_state.mut_obj(k), &S_SCHABBDEATHCAM_10);
                }
            }
            ClassType::RealHitler => {
                if rc.sound.digi_mode() != DigiMode::Off {
                    new_state(level_state.mut_obj(k), &S_HITLERDEATHCAM_140);
                } else {
                    new_state(level_state.mut_obj(k), &S_HITLERDEATHCAM_5);
                }
            }
            ClassType::Gift => {
                if rc.sound.digi_mode() != DigiMode::Off {
                    new_state(level_state.mut_obj(k), &S_GIFTDEATHCAM_140);
                } else {
                    new_state(level_state.mut_obj(k), &S_GIFTDEATHCAM_5);
                }
            }
            ClassType::Fat => {
                if rc.sound.digi_mode() != DigiMode::Off {
                    new_state(level_state.mut_obj(k), &S_FATDEATHCAM_140);
                } else {
                    new_state(level_state.mut_obj(k), &S_FATDEATHCAM_5);
                }
            }
            _ => { /* ignore */ }
        }
    })
}

fn check_position_player(level_state: &LevelState) -> bool {
    let player = level_state.player();
    let xl = (player.x - PLAYER_SIZE) >> TILESHIFT;
    let yl = (player.y - PLAYER_SIZE) >> TILESHIFT;
    let xh = (player.x + PLAYER_SIZE) >> TILESHIFT;
    let yh = (player.y + PLAYER_SIZE) >> TILESHIFT;

    // check for solid walls
    for y in yl..=yh {
        for x in xl..=xh {
            if let At::Wall(_) = level_state.actor_at[x as usize][y as usize] {
                return false;
            }
        }
    }
    true
}
