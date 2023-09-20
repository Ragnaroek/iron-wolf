use crate::{def::{ObjType, ClassType, FL_NEVERMARK, DirType, Dir, LevelState, MAP_SIZE, Level}, fixed::new_fixed_i32, agent::S_PLAYER, act2::stand};

use super::check_line;


#[test]
fn test_check_line_1() {
    let mut player = test_player();
    player.tilex = 30;
    player.tiley = 57;
    player.x = 1970056;
    player.y = 3768320;
    let level_state = mock_level_state(player);
    let obj = &stand(crate::def::EnemyType::Guard, 29, 30, 3);

    assert!(check_line(&level_state, obj));
}

#[test]
fn test_check_line_2() {
    let mut player = test_player();
    player.tilex = 32;
    player.tiley = 57;
    player.x = 2106529;
    player.y = 3768320;
    let level_state = mock_level_state(player);
    let obj = &stand(crate::def::EnemyType::Guard, 39, 61, 2);

    assert!(check_line(&level_state, obj));
}

// helper

fn mock_level_state(player: ObjType) -> LevelState {
    let tile_map = vec![vec![0; MAP_SIZE]; MAP_SIZE];     
    LevelState {
        level: Level {
            tile_map,
        },
        actors: vec![player],
        actor_at: Vec::with_capacity(0),
        doors: Vec::with_capacity(0),
        statics: Vec::with_capacity(0),
        spotvis: vec![vec![false; MAP_SIZE]; MAP_SIZE],
        vislist: Vec::with_capacity(0),
    }
}

fn test_player() -> ObjType {
    ObjType{
        class: ClassType::Player,
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
        state: &S_PLAYER,
    }
}