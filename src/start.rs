#[cfg(test)]
#[path = "./start_test.rs"]
mod start_test;

use std::process::exit;
use std::sync::{Arc, Mutex};

use vga::SCReg;
use vga::input::NumCode;
use vga::util::spawn_task;

use crate::def::{difficulty, new_game_state, weapon_type, Assets, GameState, LevelRatio, LevelState, WindowState, MAP_SIZE};
use crate::assets::{self, GraphicNum, SIGNON, GAMEPAL};
use crate::def::IWConfig;
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::config;
use crate::menu::{check_for_episodes, control_panel, initial_menu_state, MenuState };
use crate::play::{self, ProjectionConfig};
use crate::us1::c_print;
use crate::util::{new_data_reader_with_offset, DataReader};
use crate::vl;
use crate::vga_render::{self, VGARenderer};
use crate::time;
use crate::input::{self, Input};
use crate::game::{game_loop, setup_game_level};

// state for the disk animation in the load/save screen
struct DiskAnim {
    x: usize,
    y: usize,
    which: bool
}

fn new_disk_anim(x: usize, y: usize) -> DiskAnim {
    DiskAnim{x, y, which: false}
}

impl DiskAnim {
    fn disk_flop_anim(&mut self, rdr: &VGARenderer) {
        if self.which {
            rdr.pic(self.x, self.y, GraphicNum::CDISKLOADING2PIC)
        } else {
            rdr.pic(self.x, self.y, GraphicNum::CDISKLOADING1PIC)
        }   
    }
}

pub fn iw_start(loader: impl Loader + 'static, iw_config: IWConfig) -> Result<(), String> {
    let config = config::load_wolf_config(&loader);

    let vga = vga::new(0x13);
	//enable Mode Y
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let patch_config = &loader.load_patch_config_file();

    let (graphics, fonts, tiles) = assets::load_all_graphics(&loader, patch_config)?;
    let assets = assets::load_assets(&loader)?;

    // TODO calc_projection and setup_scaling have to be re-done if view size changes in config
    let prj = play::calc_projection(config.viewsize as usize);

    let ticker = time::new_ticker();
    let input_monitoring = Arc::new(Mutex::new(vga::input::new_input_monitoring()));
    let input = input::init(ticker.time_count.clone(), input_monitoring.clone());

    let mut win_state = initial_window_state();
    let mut menu_state = initial_menu_state();

    check_for_episodes(&mut menu_state);

    let vga_screen = Arc::new(vga);
    let vga_loop = vga_screen.clone();
    let rdr = vga_render::init(vga_screen.clone(), graphics, fonts, tiles, loader.variant());

	spawn_task(async move {
        init_game(&vga_loop, &rdr, &input, &mut win_state).await;
        demo_loop(&iw_config, ticker, &vga_loop, &rdr, &input, &prj, &assets, &mut win_state, &mut menu_state, &loader).await;
    });

	let options: vga::Options = vga::Options {
		show_frame_rate: false,
        input_monitoring: Some(input_monitoring),
		..Default::default()
	};
    vga_screen.start(options).unwrap();
    /*
    vga_screen.start_debug_planar_mode(
        1300,
        700,
        options, 
    ).unwrap();
    */

    Ok(())
}

fn initial_window_state() -> WindowState {
    WindowState {
        window_x: 0,
        window_y: 0,
        window_w: 320,
        window_h: 160,
        print_x: 0,
        print_y: 0,
        font_color: 0,
        font_number: 0,
        back_color: 0,
        debug_ok: false,
    }
}

async fn init_game(vga: &vga::VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    vl::set_palette(vga, GAMEPAL);
    signon_screen(vga);

    // TODO InitRedShifts
    finish_signon(vga, rdr, input, win_state).await;
}

async fn finish_signon(vga: &vga::VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    let peek = vga.read_mem(0);
    rdr.bar(0, 189, 300, 11, peek);

    win_state.window_x = 0;
    win_state.window_w = 320;
    win_state.print_y = 190;
    win_state.set_font_color(14, 4);
    c_print(rdr, win_state, "Press a key");

    input.ack().await;

    rdr.bar(0, 189, 300, 11, peek);

    win_state.print_y = 190;
    win_state.set_font_color(10, 4);
    c_print(rdr, win_state, "Working...");

    win_state.set_font_color(0, 15);
}

async fn demo_loop(
    iw_config: &IWConfig,
    ticker: time::Ticker,
    vga: &vga::VGA,
    rdr: &VGARenderer,
    input: &input::Input,
    prj: &play::ProjectionConfig,
    assets: &Assets,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader) {
    if !iw_config.options.no_wait {
        pg_13(rdr, input).await;
    }

    loop {
        while !iw_config.options.no_wait { // title screen & demo loop
            rdr.pic(0, 0, GraphicNum::TITLEPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*15).await {
                break;
            }
            rdr.fade_out().await;

            // credits page
            rdr.pic(0,0, GraphicNum::CREDITSPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*10).await {
                break;
            }
            rdr.fade_out().await;
        
            // high scores
            draw_high_scores(rdr);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*10).await {
                break;
            }

            //TODO PlayDemo() here
        }

        rdr.fade_out().await;

        let mut game_state = new_game_state();

        // TODO RecordDemo()
        let save_load = control_panel(&ticker, &mut game_state, rdr, input, win_state, menu_state, loader, NumCode::None).await;

        game_loop(&ticker, iw_config, &mut game_state, vga, rdr, input, prj, assets, win_state, menu_state, loader, save_load).await;
        rdr.fade_out().await;
    }
}

fn signon_screen(vga: &vga::VGA) {
    let mut buf_offset = 0;
    let mut vga_offset = 0;
    while buf_offset < SIGNON.len()-4 {
        vga.set_sc_data(SCReg::MapMask, 1);
        vga.write_mem(vga_offset, SIGNON[buf_offset]);

		vga.set_sc_data(SCReg::MapMask, 2);
        vga.write_mem(vga_offset, SIGNON[buf_offset+1]);
		
		vga.set_sc_data(SCReg::MapMask, 4);
        vga.write_mem(vga_offset, SIGNON[buf_offset+2]);

		vga.set_sc_data(SCReg::MapMask, 8);
        vga.write_mem(vga_offset, SIGNON[buf_offset+3]);

        vga_offset += 1;
        buf_offset += 4;
    }
}

async fn pg_13(rdr: &VGARenderer, input: &input::Input) {
    rdr.fade_out().await; 
    rdr.bar(0, 0, 320, 200, 0x82);
    rdr.pic(216, 110, GraphicNum::PG13PIC);
    
    rdr.fade_in().await;
    input.wait_user_input(time::TICK_BASE*7).await;
    rdr.fade_out().await;
}

pub fn quit(err: Option<&str>) {
    // TODO print error screen, wait for button press and the exit(0)
    println!("TODO draw exit screen, err = {:?}", err);
    exit(0)
}

pub fn load_the_game(level_state: &mut LevelState, game_state: &mut GameState, rdr: &VGARenderer, prj: &ProjectionConfig, assets: &Assets, loader: &dyn Loader, which: usize, x: usize, y: usize) {
    let mut disk_anim = new_disk_anim(x, y);
    let data = loader.load_save_game(which).expect("save game loaded");
    let reader = &mut new_data_reader_with_offset(&data, 32); //first 32 bytes are savegame name
    
    // reconstruct GameState
    disk_anim.disk_flop_anim(rdr);
    load_game_state(reader, game_state);
    // TODO Checksum

    // reconstruct LevelRation
    disk_anim.disk_flop_anim(rdr);
    load_level_ratios(reader, game_state);
    // TODO Checksum

    disk_anim.disk_flop_anim(rdr);
    println!("offset = {}", reader.offset());
    *level_state = setup_game_level(prj, game_state, assets).expect("set up game level"); // TODO replace expect with Quit()

    // load tilemap 
	for y in 0..MAP_SIZE {
		for x in 0..MAP_SIZE {
			let tile = reader.read_u8();
            level_state.level.tile_map[x][y] = tile as u16;
        }
	}

    // TODO load actorat
    // TODO load areaconnect

    // TODO load player

    // TODO Load laststatobj && staobjlist ?
    // TODO doorpositions
    // TODO doorobjlist
    // TODO pwallstate
    
    // TODO read checksum


    // TODO Checksum
}   

fn load_game_state(reader: &mut DataReader, game_state: &mut GameState)  {
    game_state.difficulty = difficulty(reader.read_u16() as usize);
    game_state.map_on = reader.read_u16() as usize;
    game_state.old_score = reader.read_i32();
    game_state.score = reader.read_i32();
    game_state.next_extra = reader.read_i32();
    game_state.lives = reader.read_i16() as i32;
    game_state.health = reader.read_i16() as i32;
    game_state.ammo = reader.read_i16() as i32;
    game_state.keys = reader.read_i16() as i32;

    game_state.best_weapon = weapon_type(reader.read_u16() as usize);
    game_state.weapon = Some(weapon_type(reader.read_u16() as usize));
    game_state.chosen_weapon = weapon_type(reader.read_u16() as usize);

    game_state.face_frame = reader.read_u16() as usize;
    game_state.attack_frame = reader.read_u16() as usize;
    game_state.attack_count = reader.read_u16() as i32;
    game_state.weapon_frame = reader.read_u16() as usize;

    game_state.episode = reader.read_u16() as usize;
    game_state.secret_count = reader.read_u16() as i32;
    game_state.treasure_count = reader.read_u16() as i32;
    game_state.kill_count = reader.read_u16() as i32;
    game_state.secret_total = reader.read_u16() as i32;
    game_state.treasure_total = reader.read_u16() as i32;
    game_state.kill_total = reader.read_u16() as i32;

    game_state.time_count = reader.read_u32() as u64;
    game_state.kill_x = reader.read_u32() as usize;
    game_state.kill_y = reader.read_u32() as usize;
    game_state.victory_flag = reader.read_u16() != 0;
}

fn load_level_ratios(reader: &mut DataReader, game_state: &mut GameState) {
    let mut level_ratios = Vec::with_capacity(8);
    for _ in 0..8 {
        let kill = reader.read_u16() as i32;
        let secret = reader.read_u16() as i32;
        let treasure = reader.read_u16() as i32;
        let time = reader.read_i32();
        level_ratios.push(LevelRatio{
            kill,
            secret,
            treasure,
            time,
        })
    }

    game_state.level_ratios = level_ratios;
}