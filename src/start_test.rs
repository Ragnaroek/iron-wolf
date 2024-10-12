use std::{path::PathBuf, sync::Arc};

use vga::SCReg;

use crate::{assets, config, def::{new_game_state, Assets, Difficulty, WeaponType, MAP_SIZE}, game::setup_game_level, loader::{DiskLoader, Loader}, play::{self, ProjectionConfig}, vga_render::{self, VGARenderer}};

use super::load_the_game;

#[test]
fn test_load_save_game() {
    let mut data_path = PathBuf::new();
    data_path.push("./testdata/shareware_data");

    let loader = DiskLoader{
        variant: &assets::W3D1,
        data_path,
        patch_path: None,
    };

    let (prj, rdr, assets) = start_test_iw(&loader);

    let mut game_state = new_game_state();
    let level_state_init = setup_game_level(&prj, &mut game_state, &assets).expect("level state");
    let mut level_state = setup_game_level(&prj, &mut game_state, &assets).expect("level state");
    // change some default values to check if it is overwritten
    game_state.best_weapon = WeaponType::Knife;
    game_state.weapon = None;
    game_state.chosen_weapon = WeaponType::Knife;

    game_state.loaded_game = true;
    load_the_game(&mut level_state, &mut game_state, &rdr, &prj, &assets, &loader, 0, 0, 0);

    // check game_state
    assert_eq!(game_state.difficulty, Difficulty::Baby);
    assert_eq!(game_state.map_on, 1);
    assert_eq!(game_state.old_score, 3700);
    assert_eq!(game_state.score, 6400);
    assert_eq!(game_state.next_extra, 40000);
    assert_eq!(game_state.lives, 3);
    assert_eq!(game_state.health, 100);
    assert_eq!(game_state.ammo, 29);
    assert_eq!(game_state.keys, 1);
    assert_eq!(game_state.best_weapon, WeaponType::MachineGun);
    assert_eq!(game_state.weapon, Some(WeaponType::MachineGun));
    assert_eq!(game_state.chosen_weapon, WeaponType::MachineGun);
    assert_eq!(game_state.face_frame, 0);
    assert_eq!(game_state.attack_frame, 0);
    assert_eq!(game_state.attack_count, 65535);
    assert_eq!(game_state.weapon_frame, 0);
    assert_eq!(game_state.episode, 0);
    assert_eq!(game_state.secret_count, 0);
    assert_eq!(game_state.treasure_count, 0);
    assert_eq!(game_state.kill_count, 15);
    assert_eq!(game_state.secret_total, 8);
    assert_eq!(game_state.treasure_total, 124);
    assert_eq!(game_state.kill_total, 51);
    assert_eq!(game_state.time_count, 14073);
    assert_eq!(game_state.kill_x, 0);
    assert_eq!(game_state.kill_y, 0);
    assert_eq!(game_state.victory_flag, false);

    //check level ratios
    assert_eq!(game_state.level_ratios.len(), 8);
    for i in 0..game_state.level_ratios.len() {
        let ratio = &game_state.level_ratios[i];
        if i == 0 {
            assert_eq!(ratio.kill, 90);
            assert_eq!(ratio.secret, 40);
            assert_eq!(ratio.treasure, 21);
            assert_eq!(ratio.time, 253); 
        } else {
            assert_eq!(ratio.kill, 0);
            assert_eq!(ratio.secret, 0);
            assert_eq!(ratio.treasure, 0);
            assert_eq!(ratio.time, 0);
        }
    }

    // check LevelState
    for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
            assert_eq!(level_state.level.tile_map[x][y], level_state_init.level.tile_map[x][y], "diff tile_map[{}][{}]", x, y);
        }
	} 
}

// helper

fn start_test_iw(loader: &dyn Loader) -> (ProjectionConfig, VGARenderer, Assets) {
    let config = config::load_wolf_config(loader);
    let vga = vga::new(0x13);

    //enable Mode Y
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
    
    let (graphics, fonts, tiles) = assets::load_all_graphics(loader, &None).expect("load all graphics");
    let assets = assets::load_assets(loader).expect("load assets");

    let prj = play::calc_projection(config.viewsize as usize);

    let vga_screen = Arc::new(vga);
    let rdr = vga_render::init(vga_screen, graphics, fonts, tiles, loader.variant());

    (prj, rdr, assets)
}
