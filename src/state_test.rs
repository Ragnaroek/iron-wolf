use crate::act2::spawn_stand;
use crate::def::{ObjType, ClassType, FL_NEVERMARK, DirType, LevelState, MAP_SIZE, Level, Difficulty, At, ObjKey, EnemyType, FL_SHOOTABLE};
use crate::fixed::new_fixed_i32;
use crate::agent::S_PLAYER;
use crate::state::check_side;

use super::{check_line, check_diag};

#[test]
fn test_check_line_1() {
    let mut player = test_player();
    player.tilex = 30;
    player.tiley = 57;
    player.x = 1970056;
    player.y = 3768320;
    let mut level_state = mock_level_state(player);
    spawn_stand(EnemyType::Guard, &mut level_state.actors, &mut level_state.actor_at, 29, 30, 3, Difficulty::Baby);
    let obj = level_state.obj(ObjKey(1));
    assert!(check_line(&level_state, obj));
}

#[test]
fn test_check_line_2() {
    let mut player = test_player();
    player.tilex = 32;
    player.tiley = 57;
    player.x = 2106529;
    player.y = 3768320;
    let mut level_state = mock_level_state(player);
    spawn_stand(EnemyType::Guard, &mut level_state.actors, &mut level_state.actor_at, 39, 61, 2, Difficulty::Baby);
    let obj = level_state.obj(ObjKey(1));
    assert!(check_line(&level_state, obj));
}

#[test]
fn test_check_diag() {
    let mut level_state = mock_level_state_with_actor_at();
    //level state contains a completely empty map without any walls or objects
    spawn_stand(EnemyType::Guard, &mut level_state.actors, &mut level_state.actor_at, 4, 3, 1, Difficulty::Baby); 
    // spawn uses wrong ObjKey since player already in the actors vec. Fix it up:
    level_state.actor_at[4][3] = At::Obj(ObjKey(1));

    assert!(check_diag(&level_state, 5, 10));

    level_state.actor_at[1][2] = At::Wall(42);
    assert!(!check_diag(&level_state, 1, 2));

    level_state.actor_at[1][3] = At::Blocked;
    assert!(!check_diag(&level_state, 1, 3)); 

    level_state.update_obj(ObjKey(1), |obj| obj.flags = 0);
    assert!(check_diag(&level_state, 4, 3));
    
    let obj = level_state.mut_obj(ObjKey(1));
    obj.flags = FL_SHOOTABLE;
    assert!(!check_diag(&level_state, 4, 3));
}

#[test]
fn test_check_side() {
    let mut level_state = mock_level_state_with_actor_at();
    spawn_stand(EnemyType::Guard, &mut level_state.actors, &mut level_state.actor_at, 4, 3, 1, Difficulty::Baby); 
    // spawn uses wrong ObjKey since player already in the actors vec. Fix it up:
    level_state.actor_at[4][3] = At::Obj(ObjKey(1));

    let (free, door) = check_side(&level_state, 5, 10);
    assert!(free);
    assert_eq!(door, -1);

    level_state.actor_at[1][2] = At::Wall(42);
    let (free, door) = check_side(&level_state, 1, 2);
    assert!(!free);
    assert_eq!(door, -1);

    level_state.actor_at[1][2] = At::Wall(255); // door
    let (free, door) = check_side(&level_state, 1, 2);
    assert!(free);
    assert_eq!(door, 255 & 63);

    level_state.actor_at[1][3] = At::Blocked;
    let (free, door) = check_side(&level_state, 1, 3);
    assert!(!free);
    assert_eq!(door, -1);

    level_state.update_obj(ObjKey(1), |obj| obj.flags = 0);
    let (free, door) = check_side(&level_state, 4, 3);
    assert!(free);
    assert_eq!(door, -1); 
}

// helper

fn mock_level_state_with_actor_at() -> LevelState {
    let mut state = mock_level_state(test_player());
    state.actor_at = vec![vec![At::Nothing; MAP_SIZE]; MAP_SIZE];
    state
}

fn mock_level_state(player: ObjType) -> LevelState {
    let tile_map = vec![vec![0; MAP_SIZE]; MAP_SIZE];     
    LevelState {
        level: Level {
            info_map: Vec::with_capacity(0),
            tile_map,
        },
        actors: vec![player],
        actor_at: vec![vec![At::Nothing; MAP_SIZE]; MAP_SIZE],
        doors: Vec::with_capacity(0),
        statics: Vec::with_capacity(0),
        spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
        vislist: Vec::with_capacity(0),
        thrustspeed: 0,
        last_attacker: None,
    }
}

fn test_player() -> ObjType {
    ObjType{
        class: ClassType::Player,
        tic_count: 0,
        distance: 0,
        area_number: 0,
        flags: FL_NEVERMARK,
        view_height: 0,
        view_x: 0,
        trans_x: new_fixed_i32(0),
        trans_y: new_fixed_i32(0),
        active: true,
        angle: 0,
        pitch: 0,
        x: 1933312,
        y: 3768320,
        tilex: 1904384,
        tiley: 1923201,
        dir: DirType::NoDir,
        speed: 0,
        temp1: 0,
        temp2: 0,
        temp3: 0,
        state: Some(&S_PLAYER),
        hitpoints: 0,
    }
}