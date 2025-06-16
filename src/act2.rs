#[cfg(test)]
#[path = "./act2_test.rs"]
mod act2_test;

use crate::act1::open_door;
use crate::agent::{S_ATTACK, S_PLAYER, take_damage};
use crate::assets::SoundName;
use crate::def::{
    AMBUSH_TILE, ANGLES, ActiveType, Actors, Assets, At, ClassType, ControlState, Difficulty,
    DirType, DoorAction, EnemyType, FL_AMBUSH, FL_NONMARK, FL_SHOOTABLE, FL_VISABLE, GameState,
    ICON_ARROWS, LevelState, MAP_SIZE, MIN_ACTOR_DIST, NUM_ENEMIES, ObjKey, ObjType, PLAYER_SIZE,
    PlayState, RUN_SPEED, SCREENLOC, SPD_DOG, SPD_PATROL, STATUS_LINES, Sprite, StateType,
    TILEGLOBAL, TILESHIFT,
};
use crate::draw::RayCastConsts;
use crate::fixed::{fixed_by_frac, new_fixed_i32, new_fixed_u32};
use crate::game::AREATILE;
use crate::input::Input;
use crate::inter::write;
use crate::map::MapSegs;
use crate::play::{ProjectionConfig, draw_play_border, finish_palette_shifts};
use crate::sd::{DigiMode, Sound};
use crate::start::quit;
use crate::state::{
    check_line, move_obj, new_state, select_chase_dir, select_dodge_dir, select_run_dir,
    sight_player, spawn_new_obj, try_walk,
};
use crate::time::Ticker;
use crate::user::rnd_t;
use crate::vga_render::{FizzleFadeAbortable, VGARenderer};

const BJ_RUN_SPEED: i32 = 2048;
const BJ_JUMP_SPEED: i32 = 680;

const PROJ_SIZE: i32 = 0x2000;
const PROJECTILE_SIZE: i32 = 0xc000;

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

// guards (1000)

pub static S_GRDSTAND: StateType = StateType {
    id: 1000,
    rotate: 1,
    sprite: Some(Sprite::GuardS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_GRDSTAND),
};

pub static S_GRDPATH1: StateType = StateType {
    id: 1001,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH1S),
};

pub static S_GRDPATH1S: StateType = StateType {
    id: 1002,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_GRDPATH3),
};

pub static S_GRDPATH2: StateType = StateType {
    id: 1003,
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH3),
};

pub static S_GRDPATH3: StateType = StateType {
    id: 1004,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH3S),
};

pub static S_GRDPATH3S: StateType = StateType {
    id: 1005,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_GRDPATH4),
};

pub static S_GRDPATH4: StateType = StateType {
    id: 1006,
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_GRDPATH1),
};

pub static S_GRDPAIN: StateType = StateType {
    id: 1007,
    rotate: 2,
    sprite: Some(Sprite::GuardPain1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDPAIN1: StateType = StateType {
    id: 1008,
    rotate: 2,
    sprite: Some(Sprite::GuardPain2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDSHOOT1: StateType = StateType {
    id: 1009,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot1),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_GRDSHOOT2),
};

pub static S_GRDSHOOT2: StateType = StateType {
    id: 1010,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_GRDSHOOT3),
};

pub static S_GRDSHOOT3: StateType = StateType {
    id: 1011,
    rotate: 0,
    sprite: Some(Sprite::GuardShoot3),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDCHASE1: StateType = StateType {
    id: 1012,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE1S),
};

pub static S_GRDCHASE1S: StateType = StateType {
    id: 1013,
    rotate: 1,
    sprite: Some(Sprite::GuardW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE2),
};

pub static S_GRDCHASE2: StateType = StateType {
    id: 1014,
    rotate: 1,
    sprite: Some(Sprite::GuardW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE3),
};

pub static S_GRDCHASE3: StateType = StateType {
    id: 1015,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE3S),
};

pub static S_GRDCHASE3S: StateType = StateType {
    id: 1016,
    rotate: 1,
    sprite: Some(Sprite::GuardW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_GRDCHASE4),
};

pub static S_GRDCHASE4: StateType = StateType {
    id: 1017,
    rotate: 1,
    sprite: Some(Sprite::GuardW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_GRDCHASE1),
};

pub static S_GRDDIE1: StateType = StateType {
    id: 1018,
    rotate: 0,
    sprite: Some(Sprite::GuardDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_GRDDIE2),
};

pub static S_GRDDIE2: StateType = StateType {
    id: 1019,
    rotate: 0,
    sprite: Some(Sprite::GuardDie2),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_GRDDIE3),
};

pub static S_GRDDIE3: StateType = StateType {
    id: 1020,
    rotate: 0,
    sprite: Some(Sprite::GuardDie3),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_GRDDIE4),
};

pub static S_GRDDIE4: StateType = StateType {
    id: 1021,
    rotate: 0,
    sprite: Some(Sprite::GuardDead),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_GRDDIE4),
};

// ghosts (1100)

pub static S_BLINKYCHASE1: StateType = StateType {
    id: 1200,
    rotate: 0,
    sprite: Some(Sprite::BlinkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_BLINKYCHASE2),
};

pub static S_BLINKYCHASE2: StateType = StateType {
    id: 1201,
    rotate: 0,
    sprite: Some(Sprite::BlinkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_BLINKYCHASE1),
};

pub static S_INKYCHASE1: StateType = StateType {
    id: 1202,
    rotate: 0,
    sprite: Some(Sprite::InkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_INKYCHASE2),
};

pub static S_INKYCHASE2: StateType = StateType {
    id: 1203,
    rotate: 0,
    sprite: Some(Sprite::InkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_INKYCHASE1),
};

pub static S_PINKYCHASE1: StateType = StateType {
    id: 1204,
    rotate: 0,
    sprite: Some(Sprite::PinkyW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_PINKYCHASE2),
};

pub static S_PINKYCHASE2: StateType = StateType {
    id: 1205,
    rotate: 0,
    sprite: Some(Sprite::PinkyW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_PINKYCHASE1),
};

pub static S_CLYDECHASE1: StateType = StateType {
    id: 1206,
    rotate: 0,
    sprite: Some(Sprite::ClydeW1),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_CLYDECHASE2),
};

pub static S_CLYDECHASE2: StateType = StateType {
    id: 1207,
    rotate: 0,
    sprite: Some(Sprite::ClydeW2),
    tic_time: 10,
    think: Some(t_ghosts),
    action: None,
    next: Some(&S_CLYDECHASE1),
};

// TODO Impl ghosts

// dogs (1200)

pub static S_DOGPATH1: StateType = StateType {
    id: 1022,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH1S),
};

pub static S_DOGPATH1S: StateType = StateType {
    id: 1023,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_DOGPATH2),
};

pub static S_DOGPATH2: StateType = StateType {
    id: 1024,
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH3),
};

pub static S_DOGPATH3: StateType = StateType {
    id: 1025,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH3S),
};

pub static S_DOGPATH3S: StateType = StateType {
    id: 1026,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_DOGPATH4),
};

pub static S_DOGPATH4: StateType = StateType {
    id: 1027,
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_DOGPATH1),
};

pub static S_DOGJUMP1: StateType = StateType {
    id: 1028,
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP2),
};

pub static S_DOGJUMP2: StateType = StateType {
    id: 1029,
    rotate: 0,
    sprite: Some(Sprite::DogJump2),
    tic_time: 10,
    think: Some(t_bite),
    action: None,
    next: Some(&S_DOGJUMP3),
};

pub static S_DOGJUMP3: StateType = StateType {
    id: 1030,
    rotate: 0,
    sprite: Some(Sprite::DogJump3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP4),
};

pub static S_DOGJUMP4: StateType = StateType {
    id: 1031,
    rotate: 0,
    sprite: Some(Sprite::DogJump1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGJUMP5),
};

pub static S_DOGJUMP5: StateType = StateType {
    id: 1032,
    rotate: 0,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGCHASE1: StateType = StateType {
    id: 1033,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE1S),
};

pub static S_DOGCHASE1S: StateType = StateType {
    id: 1034,
    rotate: 1,
    sprite: Some(Sprite::DogW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE2),
};

pub static S_DOGCHASE2: StateType = StateType {
    id: 1035,
    rotate: 1,
    sprite: Some(Sprite::DogW21),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE3),
};

pub static S_DOGCHASE3: StateType = StateType {
    id: 1036,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 10,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE3S),
};

pub static S_DOGCHASE3S: StateType = StateType {
    id: 1037,
    rotate: 1,
    sprite: Some(Sprite::DogW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_DOGCHASE4),
};

pub static S_DOGCHASE4: StateType = StateType {
    id: 1038,
    rotate: 1,
    sprite: Some(Sprite::DogW41),
    tic_time: 8,
    think: Some(t_dog_chase),
    action: None,
    next: Some(&S_DOGCHASE1),
};

pub static S_DOGDIE1: StateType = StateType {
    id: 1039,
    rotate: 0,
    sprite: Some(Sprite::DogDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_DOGDIE2),
};

pub static S_DOGDIE2: StateType = StateType {
    id: 1040,
    rotate: 0,
    sprite: Some(Sprite::DogDie2),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_DOGDIE3),
};

pub static S_DOGDIE3: StateType = StateType {
    id: 1041,
    rotate: 0,
    sprite: Some(Sprite::DogDie3),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_DOGDEAD),
};

pub static S_DOGDEAD: StateType = StateType {
    id: 1042,
    rotate: 0,
    sprite: Some(Sprite::DogDead),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_DOGDEAD),
};

// officers (1300)

pub static S_OFCSTAND: StateType = StateType {
    id: 1300,
    rotate: 1,
    sprite: Some(Sprite::OfficerS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_OFCSTAND),
};

pub static S_OFCPATH1: StateType = StateType {
    id: 1301,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_OFCPATH1S),
};

pub static S_OFCPATH1S: StateType = StateType {
    id: 1302,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_OFCPATH2),
};

pub static S_OFCPATH2: StateType = StateType {
    id: 1303,
    rotate: 1,
    sprite: Some(Sprite::OfficerW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_OFCPATH3),
};

pub static S_OFCPATH3: StateType = StateType {
    id: 1304,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_OFCPATH3S),
};

pub static S_OFCPATH3S: StateType = StateType {
    id: 1305,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_OFCPATH4),
};

pub static S_OFCPATH4: StateType = StateType {
    id: 1306,
    rotate: 1,
    sprite: Some(Sprite::OfficerW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_OFCPATH1),
};

pub static S_OFCPAIN: StateType = StateType {
    id: 1307,
    rotate: 2,
    sprite: Some(Sprite::OfficerPain1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCPAIN1: StateType = StateType {
    id: 1308,
    rotate: 2,
    sprite: Some(Sprite::OfficerPain2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCSHOOT1: StateType = StateType {
    id: 1309,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot1),
    tic_time: 6,
    think: None,
    action: None,
    next: Some(&S_OFCSHOOT2),
};

pub static S_OFCSHOOT2: StateType = StateType {
    id: 1310,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_OFCSHOOT3),
};

pub static S_OFCSHOOT3: StateType = StateType {
    id: 1311,
    rotate: 0,
    sprite: Some(Sprite::OfficerShoot3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCCHASE1: StateType = StateType {
    id: 1312,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_OFCCHASE1S),
};

pub static S_OFCCHASE1S: StateType = StateType {
    id: 1313,
    rotate: 1,
    sprite: Some(Sprite::OfficerW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_OFCCHASE2),
};

pub static S_OFCCHASE2: StateType = StateType {
    id: 1314,
    rotate: 1,
    sprite: Some(Sprite::OfficerW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_OFCCHASE3),
};

pub static S_OFCCHASE3: StateType = StateType {
    id: 1315,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_OFCCHASE3S),
};

pub static S_OFCCHASE3S: StateType = StateType {
    id: 1316,
    rotate: 1,
    sprite: Some(Sprite::OfficerW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_OFCCHASE4),
};

pub static S_OFCCHASE4: StateType = StateType {
    id: 1317,
    rotate: 1,
    sprite: Some(Sprite::OfficerW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_OFCCHASE1),
};

pub static S_OFCDIE1: StateType = StateType {
    id: 1318,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie1),
    tic_time: 11,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_OFCDIE2),
};

pub static S_OFCDIE2: StateType = StateType {
    id: 1319,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie2),
    tic_time: 11,
    think: None,
    action: None,
    next: Some(&S_OFCDIE3),
};

pub static S_OFCDIE3: StateType = StateType {
    id: 1320,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie3),
    tic_time: 11,
    think: None,
    action: None,
    next: Some(&S_OFCDIE4),
};

pub static S_OFCDIE4: StateType = StateType {
    id: 1321,
    rotate: 0,
    sprite: Some(Sprite::OfficerDie4),
    tic_time: 11,
    think: None,
    action: None,
    next: Some(&S_OFCDIE5),
};

pub static S_OFCDIE5: StateType = StateType {
    id: 1322,
    rotate: 0,
    sprite: Some(Sprite::OfficerDead),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_OFCDIE5),
};

// mutant (1400)

pub static S_MUTSTAND: StateType = StateType {
    id: 1400,
    rotate: 1,
    sprite: Some(Sprite::MutantS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_MUTSTAND),
};

pub static S_MUTPATH1: StateType = StateType {
    id: 1401,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_MUTPATH1S),
};

pub static S_MUTPATH1S: StateType = StateType {
    id: 1402,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_MUTPATH2),
};

pub static S_MUTPATH2: StateType = StateType {
    id: 1403,
    rotate: 1,
    sprite: Some(Sprite::MutantW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_MUTPATH3),
};

pub static S_MUTPATH3: StateType = StateType {
    id: 1404,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_MUTPATH3S),
};

pub static S_MUTPATH3S: StateType = StateType {
    id: 1405,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_MUTPATH4),
};

pub static S_MUTPATH4: StateType = StateType {
    id: 1406,
    rotate: 1,
    sprite: Some(Sprite::MutantW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_MUTPATH1),
};

pub static S_MUTPAIN: StateType = StateType {
    id: 1407,
    rotate: 2,
    sprite: Some(Sprite::MutantPain1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTPAIN1: StateType = StateType {
    id: 1408,
    rotate: 2,
    sprite: Some(Sprite::MutantPain2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTSHOOT1: StateType = StateType {
    id: 1409,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot1),
    tic_time: 6,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_MUTSHOOT2),
};

pub static S_MUTSHOOT2: StateType = StateType {
    id: 1410,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot2),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_MUTSHOOT3),
};

pub static S_MUTSHOOT3: StateType = StateType {
    id: 1411,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_MUTSHOOT4),
};

pub static S_MUTSHOOT4: StateType = StateType {
    id: 1412,
    rotate: 0,
    sprite: Some(Sprite::MutantShoot4),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTCHASE1: StateType = StateType {
    id: 1413,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_MUTCHASE1S),
};

pub static S_MUTCHASE1S: StateType = StateType {
    id: 1414,
    rotate: 1,
    sprite: Some(Sprite::MutantW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_MUTCHASE2),
};

pub static S_MUTCHASE2: StateType = StateType {
    id: 1415,
    rotate: 1,
    sprite: Some(Sprite::MutantW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_MUTCHASE3),
};

pub static S_MUTCHASE3: StateType = StateType {
    id: 1416,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_MUTCHASE3S),
};

pub static S_MUTCHASE3S: StateType = StateType {
    id: 1417,
    rotate: 1,
    sprite: Some(Sprite::MutantW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_MUTCHASE4),
};

pub static S_MUTCHASE4: StateType = StateType {
    id: 1418,
    rotate: 1,
    sprite: Some(Sprite::MutantW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_MUTCHASE1),
};

pub static S_MUTDIE1: StateType = StateType {
    id: 1419,
    rotate: 0,
    sprite: Some(Sprite::MutantDie1),
    tic_time: 7,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_MUTDIE2),
};

pub static S_MUTDIE2: StateType = StateType {
    id: 1420,
    rotate: 0,
    sprite: Some(Sprite::MutantDie2),
    tic_time: 7,
    think: None,
    action: None,
    next: Some(&S_MUTDIE3),
};

pub static S_MUTDIE3: StateType = StateType {
    id: 1421,
    rotate: 0,
    sprite: Some(Sprite::MutantDie3),
    tic_time: 7,
    think: None,
    action: None,
    next: Some(&S_MUTDIE4),
};

pub static S_MUTDIE4: StateType = StateType {
    id: 1422,
    rotate: 0,
    sprite: Some(Sprite::MutantDie4),
    tic_time: 7,
    think: None,
    action: None,
    next: Some(&S_MUTDIE5),
};

pub static S_MUTDIE5: StateType = StateType {
    id: 1423,
    rotate: 0,
    sprite: Some(Sprite::MutantDead),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_MUTDIE5),
};

// SS (1500)

pub static S_SSSTAND: StateType = StateType {
    id: 1045,
    rotate: 1,
    sprite: Some(Sprite::SSS1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_SSSTAND),
};

pub static S_SSPATH1: StateType = StateType {
    id: 1046,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH1S),
};

pub static S_SSPATH1S: StateType = StateType {
    id: 1047,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_SSPATH2),
};

pub static S_SSPATH2: StateType = StateType {
    id: 1048,
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH3),
};

pub static S_SSPATH3: StateType = StateType {
    id: 1049,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 20,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH3S),
};

pub static S_SSPATH3S: StateType = StateType {
    id: 1050,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 5,
    think: None,
    action: None,
    next: Some(&S_SSPATH4),
};

pub static S_SSPATH4: StateType = StateType {
    id: 1051,
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 15,
    think: Some(t_path),
    action: None,
    next: Some(&S_SSPATH1),
};

pub static S_SSPAIN: StateType = StateType {
    id: 1052,
    rotate: 2,
    sprite: Some(Sprite::SSPAIN1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSPAIN1: StateType = StateType {
    id: 1053,
    rotate: 2,
    sprite: Some(Sprite::SSPAIN2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSSHOOT1: StateType = StateType {
    id: 1054,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT1),
    tic_time: 20,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT2),
};

pub static S_SSSHOOT2: StateType = StateType {
    id: 1055,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 20,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT3),
};

pub static S_SSSHOOT3: StateType = StateType {
    id: 1056,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT4),
};

pub static S_SSSHOOT4: StateType = StateType {
    id: 1057,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT5),
};

pub static S_SSSHOOT5: StateType = StateType {
    id: 1058,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT6),
};

pub static S_SSSHOOT6: StateType = StateType {
    id: 1059,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT7),
};

pub static S_SSSHOOT7: StateType = StateType {
    id: 1060,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSSHOOT8),
};

pub static S_SSSHOOT8: StateType = StateType {
    id: 1061,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_SSSHOOT9),
};

pub static S_SSSHOOT9: StateType = StateType {
    id: 1062,
    rotate: 0,
    sprite: Some(Sprite::SSSHOOT3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSCHASE1: StateType = StateType {
    id: 1063,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE1S),
};

pub static S_SSCHASE1S: StateType = StateType {
    id: 1064,
    rotate: 1,
    sprite: Some(Sprite::SSW11),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SSCHASE2),
};

pub static S_SSCHASE2: StateType = StateType {
    id: 1065,
    rotate: 1,
    sprite: Some(Sprite::SSW21),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE3),
};

pub static S_SSCHASE3: StateType = StateType {
    id: 1066,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE3S),
};

pub static S_SSCHASE3S: StateType = StateType {
    id: 1067,
    rotate: 1,
    sprite: Some(Sprite::SSW31),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SSCHASE4),
};

pub static S_SSCHASE4: StateType = StateType {
    id: 1068,
    rotate: 1,
    sprite: Some(Sprite::SSW41),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_SSCHASE1),
};

pub static S_SSDIE1: StateType = StateType {
    id: 1069,
    rotate: 0,
    sprite: Some(Sprite::SSDIE1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_SSDIE2),
};

pub static S_SSDIE2: StateType = StateType {
    id: 1070,
    rotate: 0,
    sprite: Some(Sprite::SSDIE2),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_SSDIE3),
};

pub static S_SSDIE3: StateType = StateType {
    id: 1071,
    rotate: 0,
    sprite: Some(Sprite::SSDIE3),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_SSDIE4),
};

pub static S_SSDIE4: StateType = StateType {
    id: 1072,
    rotate: 0,
    sprite: Some(Sprite::SSDEAD),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_SSDIE4),
};

//
// hans (1600)
//
pub static S_BOSSSTAND: StateType = StateType {
    id: 1073,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 0,
    think: Some(t_stand),
    action: None,
    next: Some(&S_BOSSSTAND),
};

pub static S_BOSSCHASE1: StateType = StateType {
    id: 1074,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_BOSSCHASE1S),
};

pub static S_BOSSCHASE1S: StateType = StateType {
    id: 1075,
    rotate: 0,
    sprite: Some(Sprite::BossW1),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_BOSSCHASE2),
};

pub static S_BOSSCHASE2: StateType = StateType {
    id: 1076,
    rotate: 0,
    sprite: Some(Sprite::BossW2),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_BOSSCHASE3),
};

pub static S_BOSSCHASE3: StateType = StateType {
    id: 1077,
    rotate: 0,
    sprite: Some(Sprite::BossW3),
    tic_time: 10,
    think: Some(t_chase),
    action: None,
    next: Some(&S_BOSSCHASE3S),
};

pub static S_BOSSCHASE3S: StateType = StateType {
    id: 1078,
    rotate: 0,
    sprite: Some(Sprite::BossW3),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_BOSSCHASE4),
};

pub static S_BOSSCHASE4: StateType = StateType {
    id: 1079,
    rotate: 0,
    sprite: Some(Sprite::BossW4),
    tic_time: 8,
    think: Some(t_chase),
    action: None,
    next: Some(&S_BOSSCHASE1),
};

pub static S_BOSSDIE1: StateType = StateType {
    id: 1080,
    rotate: 0,
    sprite: Some(Sprite::BossDie1),
    tic_time: 15,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_BOSSDIE2),
};

pub static S_BOSSDIE2: StateType = StateType {
    id: 1081,
    rotate: 0,
    sprite: Some(Sprite::BossDie2),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_BOSSDIE3),
};

pub static S_BOSSDIE3: StateType = StateType {
    id: 1082,
    rotate: 0,
    sprite: Some(Sprite::BossDie3),
    tic_time: 15,
    think: None,
    action: None,
    next: Some(&S_BOSSDIE4),
};

pub static S_BOSSDIE4: StateType = StateType {
    id: 1083,
    rotate: 0,
    sprite: Some(Sprite::BossDead),
    tic_time: 0,
    think: None,
    action: None,
    next: Some(&S_BOSSDIE4),
};

pub static S_BOSSSHOOT1: StateType = StateType {
    id: 1084,
    rotate: 0,
    sprite: Some(Sprite::BossShoot1),
    tic_time: 30,
    think: None,
    action: None,
    next: Some(&S_BOSSSHOOT2),
};

pub static S_BOSSSHOOT2: StateType = StateType {
    id: 1085,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT3),
};

pub static S_BOSSSHOOT3: StateType = StateType {
    id: 1086,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT4),
};

pub static S_BOSSSHOOT4: StateType = StateType {
    id: 1087,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT5),
};

pub static S_BOSSSHOOT5: StateType = StateType {
    id: 1088,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT6),
};

pub static S_BOSSSHOOT6: StateType = StateType {
    id: 1089,
    rotate: 0,
    sprite: Some(Sprite::BossShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT7),
};

pub static S_BOSSSHOOT7: StateType = StateType {
    id: 1090,
    rotate: 0,
    sprite: Some(Sprite::BossShoot3),
    tic_time: 10,
    think: None,
    action: Some(t_shoot),
    next: Some(&S_BOSSSHOOT8),
};

pub static S_BOSSSHOOT8: StateType = StateType {
    id: 1091,
    rotate: 0,
    sprite: Some(Sprite::BossShoot1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_BOSSCHASE1),
};

//
// schabb
//
pub static S_SCHABBSTAND: StateType = StateType {
    id: 1092,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: Some(t_stand),
    action: None,
    next: Some(&S_SCHABBSTAND),
};

pub static S_SCHABBCHASE1: StateType = StateType {
    id: 1093,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: Some(t_schabb),
    action: None,
    next: Some(&S_SCHABBCHASE1S),
};

pub static S_SCHABBCHASE1S: StateType = StateType {
    id: 1094,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SCHABBCHASE2),
};
pub static S_SCHABBCHASE2: StateType = StateType {
    id: 1095,
    rotate: 0,
    sprite: Some(Sprite::SchabbW2),
    tic_time: 8,
    think: Some(t_schabb),
    action: None,
    next: Some(&S_SCHABBCHASE3),
};

pub static S_SCHABBCHASE3: StateType = StateType {
    id: 1096,
    rotate: 0,
    sprite: Some(Sprite::SchabbW3),
    tic_time: 10,
    think: Some(t_schabb),
    action: None,
    next: Some(&S_SCHABBCHASE3S),
};

pub static S_SCHABBCHASE3S: StateType = StateType {
    id: 1097,
    rotate: 0,
    sprite: Some(Sprite::SchabbW3),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_SCHABBCHASE4),
};

pub static S_SCHABBCHASE4: StateType = StateType {
    id: 1098,
    rotate: 0,
    sprite: Some(Sprite::SchabbW4),
    tic_time: 8,
    think: Some(t_schabb),
    action: None,
    next: Some(&S_SCHABBCHASE1),
};

pub static S_SCHABBDEATHCAM_140: StateType = StateType {
    id: 1099,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 1,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE1_140),
};

pub static S_SCHABBDEATHCAM_5: StateType = StateType {
    id: 1100,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 1,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE1_5),
};

pub static S_SCHABBDIE1_140: StateType = StateType {
    id: 1101,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_SCHABBDIE2_140),
};

pub static S_SCHABBDIE1_5: StateType = StateType {
    id: 1102,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 10,
    think: None,
    action: Some(a_death_scream),
    next: Some(&S_SCHABBDIE2_5),
};

pub static S_SCHABBDIE2_140: StateType = StateType {
    id: 1103,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 140,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE3),
};

pub static S_SCHABBDIE2_5: StateType = StateType {
    id: 1104,
    rotate: 0,
    sprite: Some(Sprite::SchabbW1),
    tic_time: 140,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE3),
};

pub static S_SCHABBDIE3: StateType = StateType {
    id: 1105,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie1),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE4),
};

pub static S_SCHABBDIE4: StateType = StateType {
    id: 1106,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie2),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE5),
};

pub static S_SCHABBDIE5: StateType = StateType {
    id: 1107,
    rotate: 0,
    sprite: Some(Sprite::SchabbDie3),
    tic_time: 10,
    think: None,
    action: None,
    next: Some(&S_SCHABBDIE6),
};

pub static S_SCHABBDIE6: StateType = StateType {
    id: 1108,
    rotate: 0,
    sprite: Some(Sprite::SchabbDead),
    tic_time: 20,
    think: None,
    action: Some(a_start_death_cam),
    next: Some(&S_SCHABBDIE6),
};

pub static S_SCHABBSHOOT1: StateType = StateType {
    id: 1109,
    rotate: 0,
    sprite: Some(Sprite::SchabbShoot1),
    tic_time: 30,
    think: None,
    action: None,
    next: Some(&S_SCHABBSHOOT2),
};

pub static S_SCHABBSHOOT2: StateType = StateType {
    id: 1110,
    rotate: 0,
    sprite: Some(Sprite::SchabbShoot2),
    tic_time: 10,
    think: None,
    action: Some(t_schabb_throw),
    next: Some(&S_SCHABBCHASE1),
};

pub static S_NEEDLE1: StateType = StateType {
    id: 1111,
    rotate: 0,
    sprite: Some(Sprite::Hypo1),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    next: Some(&S_NEEDLE2),
};

pub static S_NEEDLE2: StateType = StateType {
    id: 1112,
    rotate: 0,
    sprite: Some(Sprite::Hypo2),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    next: Some(&S_NEEDLE3),
};

pub static S_NEEDLE3: StateType = StateType {
    id: 1113,
    rotate: 0,
    sprite: Some(Sprite::Hypo3),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    next: Some(&S_NEEDLE4),
};

pub static S_NEEDLE4: StateType = StateType {
    id: 1114,
    rotate: 0,
    sprite: Some(Sprite::Hypo4),
    tic_time: 6,
    think: Some(t_projectile),
    action: None,
    next: Some(&S_NEEDLE1),
};

pub static S_BOOM1: StateType = StateType {
    id: 1115,
    rotate: 0,
    sprite: Some(Sprite::Boom1),
    tic_time: 6,
    think: None,
    action: None,
    next: Some(&S_BOOM2),
};

pub static S_BOOM2: StateType = StateType {
    id: 1116,
    rotate: 0,
    sprite: Some(Sprite::Boom2),
    tic_time: 6,
    think: None,
    action: None,
    next: Some(&S_BOOM3),
};

pub static S_BOOM3: StateType = StateType {
    id: 1117,
    rotate: 0,
    sprite: Some(Sprite::Boom3),
    tic_time: 6,
    think: None,
    action: None,
    next: None,
};

pub static STATES: [&'static StateType; 173] = [
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
    &S_SCHABBDEATHCAM_5,
    &S_SCHABBDIE1_140,
    &S_SCHABBDIE1_5,
    &S_SCHABBDIE2_140,
    &S_SCHABBDIE2_5,
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
    &S_BOOM1,
    &S_BOOM2,
    &S_BOOM3,
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    rc: &ProjectionConfig,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    let player_x = level_state.player().x;
    let player_y = level_state.player().y;

    let speed = new_fixed_i32(level_state.obj(k).speed * tics as i32);

    let mut delta_x = fixed_by_frac(speed, rc.cos(level_state.obj(k).angle as usize)).to_i32();
    let mut delta_y = fixed_by_frac(speed, rc.sin(level_state.obj(k).angle as usize)).to_i32();

    if delta_x > 0x10000 {
        delta_x = 0x10000;
    }
    if delta_y > 0x10000 {
        delta_y = 0x10000;
    }

    level_state.mut_obj(k).x += delta_x;
    level_state.mut_obj(k).y += delta_y;

    delta_x = level_state.obj(k).x - player_x;
    delta_y = level_state.obj(k).y - player_y;

    if !projectile_try_move(k, level_state) {
        if level_state.obj(k).class == ClassType::Rocket {
            sound.play_sound_loc_actor(
                SoundName::MISSILEHIT,
                assets,
                rc_consts,
                level_state.obj(k),
            );
            level_state.mut_obj(k).state = Some(&S_BOOM1);
        } else {
            level_state.mut_obj(k).state = None; // mark for removal
        }
        return;
    }

    if delta_x < PROJECTILE_SIZE && delta_y < PROJECTILE_SIZE {
        // hit the player
        let damage = match level_state.obj(k).class {
            ClassType::Needle => (rnd_t() >> 3) + 20,
            ClassType::Rocket | ClassType::HRocket | ClassType::Spark => (rnd_t() >> 3) + 30,
            ClassType::Fire => rnd_t() >> 3,
            _ => 0,
        } as i32;

        take_damage(k, damage, level_state, game_state, rdr);
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

fn t_path(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    if sight_player(k, level_state, game_state, sound, assets, rc_consts, tics) {
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
            move_obj(k, level_state, game_state, rdr, mov, tics);
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
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
            move_obj(k, level_state, game_state, rdr, mov, tics);
            break;
        }

        // reached goal tile, so select another one

        // fix position to account for round off during moving

        level_state.update_obj(k, |obj| {
            obj.x = ((obj.tilex as i32) << TILESHIFT) + TILEGLOBAL / 2;
            obj.y = ((obj.tiley as i32) << TILESHIFT) + TILEGLOBAL / 2;
            mov -= obj.distance;
        });

        select_dodge_dir(
            k,
            level_state,
            level_state.obj(k).tilex,
            level_state.obj(k).tiley,
        );

        if level_state.obj(k).dir == DirType::NoDir {
            return;
        }
    }
}

fn t_bite(
    k: ObjKey,
    _: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    _: &RayCastConsts,
) {
    sound.play_sound(SoundName::DOGATTACK, assets);

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
                take_damage(k, (rnd_t() >> 4) as i32, level_state, game_state, rdr);
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    _: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    sight_player(k, level_state, game_state, sound, assets, rc_consts, tics);
}

fn t_chase(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
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
            move_obj(k, level_state, game_state, rdr, mov, tics);
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
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
            move_obj(k, level_state, game_state, rdr, mov, tics);
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
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
            move_obj(k, level_state, game_state, rdr, mov, tics);
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
    k: ObjKey,
    _: u64,
    _: &Ticker,
    level_state: &mut LevelState,
    _: &mut GameState,
    sound: &mut Sound,
    _: &VGARenderer,
    _: &Input,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    rc: &RayCastConsts,
) {
    let player = level_state.player();
    let delta_x = player.x - level_state.obj(k).x;
    let delta_y = level_state.obj(k).y - player.y;

    let mut angle = (delta_y as f64).atan2(delta_x as f64);
    if angle < 0.0 {
        angle = std::f64::consts::PI * 2.0 + angle;
    }
    let iangle = (angle / (std::f64::consts::PI * 2.0)) as i32 * ANGLES as i32;

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

    sound.play_sound_loc_actor(SoundName::SCHABBSTHROW, assets, rc, &obj);
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
    k: ObjKey,
    _: u64,
    _: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    _: &Input,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    _: &RayCastConsts,
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

        take_damage(k, damage as i32, level_state, game_state, rdr)
    }

    let obj = level_state.obj(k);
    match obj.class {
        ClassType::SS => {
            sound.play_sound(SoundName::SSFIRE, assets);
        }
        ClassType::Gift | ClassType::Fat => {
            sound.play_sound(SoundName::MISSILEFIRE, assets);
        }
        ClassType::MechaHitler | ClassType::RealHitler | ClassType::Boss => {
            sound.play_sound(SoundName::BOSSFIRE, assets);
        }
        ClassType::Schabb => {
            sound.play_sound(SoundName::SCHABBSTHROW, assets);
        }
        ClassType::Fake => {
            sound.play_sound(SoundName::FLAMETHROWER, assets);
        }
        _ => {
            sound.play_sound(SoundName::NAZIFIRE, assets);
        }
    }
}

fn a_death_scream(
    k: ObjKey,
    _: u64,
    _: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    _: &VGARenderer,
    _: &Input,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    do_death_scream(k, level_state, game_state, sound, assets, rc_consts);
}

pub fn do_death_scream(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    let obj = level_state.obj(k);
    if game_state.map_on == 9 && rnd_t() == 0 {
        match obj.class {
            ClassType::Mutant
            | ClassType::Guard
            | ClassType::Officer
            | ClassType::SS
            | ClassType::Dog => {
                sound.play_sound_loc_actor(SoundName::DEATHSCREAM6, assets, rc_consts, obj);
                return;
            }
            _ => { /* play nothing */ }
        }
    }

    match obj.class {
        ClassType::Mutant => {
            sound.play_sound_loc_actor(SoundName::AHHHG, assets, rc_consts, obj);
        }
        ClassType::Guard => {
            sound.play_sound_loc_actor(
                GUARD_DEATH_SCREAMS[(rnd_t() % 8) as usize],
                assets,
                rc_consts,
                obj,
            );
        }
        ClassType::Officer => {
            sound.play_sound_loc_actor(SoundName::NEINSOVAS, assets, rc_consts, obj);
        }
        ClassType::SS => {
            sound.play_sound_loc_actor(SoundName::LEBEN, assets, rc_consts, obj);
        }
        ClassType::Dog => {
            sound.play_sound_loc_actor(SoundName::DOGDEATH, assets, rc_consts, obj);
        }
        ClassType::Boss => {
            sound.play_sound(SoundName::MUTTI, assets);
        }
        ClassType::Schabb => {
            sound.play_sound(SoundName::MEINGOTT, assets);
        }
        // TODO realhitlerobj EVASND
        // TODO mechahilterobj SCHEISTSND
        // TODO fakeobj HITLERHASND
        // TODO giftobj DONNERSND
        // TODO gretelobj MEINSND
        // TODO fatobj ROSESND
        _ => todo!("death scream missing: {:?}", obj.class),
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
    next: Some(&S_BJRUN1S),
};

pub static S_BJRUN1S: StateType = StateType {
    id: 9001,
    rotate: 0,
    sprite: Some(Sprite::BJW1),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_BJRUN2),
};

pub static S_BJRUN2: StateType = StateType {
    id: 9002,
    rotate: 0,
    sprite: Some(Sprite::BJW2),
    tic_time: 8,
    think: Some(t_bj_run),
    action: None,
    next: Some(&S_BJRUN3),
};

pub static S_BJRUN3: StateType = StateType {
    id: 9003,
    rotate: 0,
    sprite: Some(Sprite::BJW3),
    tic_time: 12,
    think: Some(t_bj_run),
    action: None,
    next: Some(&S_BJRUN3S),
};

pub static S_BJRUN3S: StateType = StateType {
    id: 9004,
    rotate: 0,
    sprite: Some(Sprite::BJW3),
    tic_time: 3,
    think: None,
    action: None,
    next: Some(&S_BJRUN4),
};

pub static S_BJRUN4: StateType = StateType {
    id: 9005,
    rotate: 0,
    sprite: Some(Sprite::BJW4),
    tic_time: 8,
    think: Some(t_bj_run),
    action: None,
    next: Some(&S_BJRUN1),
};

pub static S_BJ_JUMP1: StateType = StateType {
    id: 9006,
    rotate: 0,
    sprite: Some(Sprite::BJJump1),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: None,
    next: Some(&S_BJ_JUMP2),
};

pub static S_BJ_JUMP2: StateType = StateType {
    id: 9007,
    rotate: 0,
    sprite: Some(Sprite::BJJump2),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: Some(t_bj_yell),
    next: Some(&S_BJ_JUMP3),
};

pub static S_BJ_JUMP3: StateType = StateType {
    id: 9008,
    rotate: 0,
    sprite: Some(Sprite::BJJump3),
    tic_time: 14,
    think: Some(t_bj_jump),
    action: None,
    next: Some(&S_BJ_JUMP4),
};

pub static S_BJ_JUMP4: StateType = StateType {
    id: 9009,
    rotate: 0,
    sprite: Some(Sprite::BJJump4),
    tic_time: 300,
    think: None,
    action: Some(t_bj_done),
    next: Some(&S_BJ_JUMP4),
};

pub static S_DEATH_CAM: StateType = StateType {
    id: 9010,
    rotate: 0,
    sprite: None,
    tic_time: 0,
    think: None,
    action: None,
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
) {
    let mut mov = BJ_RUN_SPEED * tics as i32;
    while mov > 0 {
        if mov < level_state.obj(k).distance {
            move_obj(k, level_state, game_state, rdr, mov, tics);
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
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    rdr: &VGARenderer,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
) {
    let mov = BJ_JUMP_SPEED * tics as i32;
    move_obj(k, level_state, game_state, rdr, mov, tics);
}

fn t_bj_yell(
    k: ObjKey,
    _: u64,
    _: &Ticker,
    level_state: &mut LevelState,
    _: &mut GameState,
    sound: &mut Sound,
    _: &VGARenderer,
    _: &Input,
    _: &mut ControlState,
    _: &ProjectionConfig,
    assets: &Assets,
    rc_consts: &RayCastConsts,
) {
    let obj = level_state.obj(k);
    sound.play_sound_loc_actor(SoundName::YEAH, assets, rc_consts, obj);
}

fn t_bj_done(
    _: ObjKey,
    _: u64,
    _: &Ticker,
    _: &mut LevelState,
    game_state: &mut GameState,
    _: &mut Sound,
    _: &VGARenderer,
    _: &Input,
    _: &mut ControlState,
    _: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
) {
    game_state.play_state = PlayState::Victorious;
}

fn a_start_death_cam(
    k: ObjKey,
    _: u64,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    input: &Input,
    _: &mut ControlState,
    prj: &ProjectionConfig,
    _: &Assets,
    _: &RayCastConsts,
) {
    finish_palette_shifts(game_state, &rdr.vga);

    if game_state.victory_flag {
        game_state.play_state = PlayState::Victorious;
        return;
    }
    game_state.victory_flag = true;

    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);
    rdr.fizzle_fade(
        ticker,
        rdr.buffer_offset(),
        rdr.active_buffer(),
        320,
        200 - STATUS_LINES,
        70,
        FizzleFadeAbortable::No,
    );
    rdr.set_buffer_offset(rdr.active_buffer());

    write(rdr, 0, 7, "Let's see that again!");
    input.wait_user_input(300);

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
    let angle = (fangle / (std::f64::consts::PI * 2.0)) as i32 * ANGLES as i32;
    level_state.mut_player().angle = angle;
    // try to position as close as possible without being in a wall
    let mut dist = 0x14000;
    loop {
        let x_move = fixed_by_frac(new_fixed_u32(dist), prj.cos(angle as usize));
        let y_move = -fixed_by_frac(new_fixed_u32(dist), prj.sin(angle as usize));

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
    let offset_prev = rdr.buffer_offset();
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        draw_play_border(rdr, prj.view_width, prj.view_height);
    }
    rdr.set_buffer_offset(offset_prev);

    game_state.fizzle_in = true;
    let obj = level_state.mut_obj(k);
    match obj.class {
        ClassType::Schabb => {
            if sound.digi_mode() != DigiMode::Off {
                new_state(level_state.mut_obj(k), &S_SCHABBDEATHCAM_140);
            } else {
                new_state(level_state.mut_obj(k), &S_SCHABBDEATHCAM_5);
            }
        }
        // TODO realhitler
        // TODO giftobj
        // TODO fatobj
        _ => { /* ignore */ }
    }
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
