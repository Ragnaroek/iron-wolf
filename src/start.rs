#[cfg(test)]
#[path = "./start_test.rs"]
mod start_test;

use std::cmp::min;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::usize;

use vga::input::NumCode;
use vga::util::spawn_task;
use vga::SCReg;

use opl::OPL;

use crate::assets::{self, GraphicNum, GAMEPAL, SIGNON};
use crate::def::{
    difficulty, new_game_state, weapon_type, ActiveType, Assets, At, ClassType, Dir, DirType,
    DoorAction, DoorLock, DoorType, GameState, IWConfig, LevelRatio, LevelState, ObjKey, ObjType,
    Sprite, StaticKind, StaticType, WeaponType, WindowState, MAP_SIZE, MAX_DOORS, MAX_STATS,
    NUM_AREAS,
};
use crate::fixed::{new_fixed_u16, new_fixed_u32};
use crate::game::{game_loop, setup_game_level};
use crate::input::{self, Input};
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::menu::{
    check_for_episodes, control_panel, initial_menu_state, intro_song, message, start_cp_music,
    MenuState,
};
use crate::play::{self, ProjectionConfig};
use crate::time;
use crate::us1::c_print;
use crate::util::{new_data_reader_with_offset, new_data_writer, DataReader, DataWriter};
use crate::vga_render::{self, VGARenderer};
use crate::vl;
use crate::{config, game};

const OBJ_TYPE_LEN: usize = 60;
const STAT_TYPE_LEN: usize = 8;
const DOOR_TYPE_LEN: usize = 10;
const LEVEL_RATIO_TYPE_LEN: usize = 10;
const SAVEGAME_NAME_LEN: usize = 32;

static STR_SAVE_CHEAT : &'static str = "Your Save Game file is,\nshall we say, \"corrupted\".\nBut I'll let you go on and\nplay anyway....";

// state for the disk animation in the load/save screen
struct DiskAnim {
    x: usize,
    y: usize,
    which: bool,
}

fn new_disk_anim(x: usize, y: usize) -> DiskAnim {
    DiskAnim { x, y, which: false }
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

    let mut opl = opl::new()?;

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
        demo_loop(
            &iw_config,
            ticker,
            &vga_loop,
            &mut opl,
            &rdr,
            &input,
            &prj,
            &assets,
            &mut win_state,
            &mut menu_state,
            &loader,
        )
        .await;
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

pub fn initial_window_state() -> WindowState {
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
        in_game: false,
    }
}

async fn init_game(vga: &vga::VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    vl::set_palette(vga, GAMEPAL);
    signon_screen(vga);

    // TODO InitRedShifts
    finish_signon(vga, rdr, input, win_state).await;
}

async fn finish_signon(
    vga: &vga::VGA,
    rdr: &VGARenderer,
    input: &Input,
    win_state: &mut WindowState,
) {
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
    opl: &mut OPL,
    rdr: &VGARenderer,
    input: &input::Input,
    prj: &play::ProjectionConfig,
    assets: &Assets,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) {
    start_cp_music(opl, intro_song(loader.variant()), assets, loader);

    if !iw_config.options.no_wait {
        pg_13(rdr, input).await;
    }

    loop {
        while !iw_config.options.no_wait {
            // title screen & demo loop
            rdr.pic(0, 0, GraphicNum::TITLEPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE * 15).await {
                break;
            }
            rdr.fade_out().await;

            // credits page
            rdr.pic(0, 0, GraphicNum::CREDITSPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE * 10).await {
                break;
            }
            rdr.fade_out().await;

            // high scores
            draw_high_scores(rdr);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE * 10).await {
                break;
            }

            //TODO PlayDemo() here
        }

        rdr.fade_out().await;

        let mut game_state = new_game_state();

        // TODO RecordDemo()
        let save_load = control_panel(
            &ticker,
            &mut game_state,
            opl,
            rdr,
            input,
            assets,
            win_state,
            menu_state,
            loader,
            NumCode::None,
        )
        .await;

        game_loop(
            &ticker,
            iw_config,
            &mut game_state,
            vga,
            opl,
            rdr,
            input,
            prj,
            assets,
            win_state,
            menu_state,
            loader,
            save_load,
        )
        .await;
        rdr.fade_out().await;
    }
}

fn signon_screen(vga: &vga::VGA) {
    let mut buf_offset = 0;
    let mut vga_offset = 0;
    while buf_offset < SIGNON.len() - 4 {
        vga.set_sc_data(SCReg::MapMask, 1);
        vga.write_mem(vga_offset, SIGNON[buf_offset]);

        vga.set_sc_data(SCReg::MapMask, 2);
        vga.write_mem(vga_offset, SIGNON[buf_offset + 1]);

        vga.set_sc_data(SCReg::MapMask, 4);
        vga.write_mem(vga_offset, SIGNON[buf_offset + 2]);

        vga.set_sc_data(SCReg::MapMask, 8);
        vga.write_mem(vga_offset, SIGNON[buf_offset + 3]);

        vga_offset += 1;
        buf_offset += 4;
    }
}

async fn pg_13(rdr: &VGARenderer, input: &input::Input) {
    rdr.fade_out().await;
    rdr.bar(0, 0, 320, 200, 0x82);
    rdr.pic(216, 110, GraphicNum::PG13PIC);

    rdr.fade_in().await;
    input.wait_user_input(time::TICK_BASE * 7).await;
    rdr.fade_out().await;
}

pub fn quit(err: Option<&str>) {
    // TODO print error screen, wait for button press and the exit(0)
    println!("TODO draw exit screen, err = {:?}", err);
    exit(0)
}

pub fn save_the_game(
    level_state: &LevelState,
    game_state: &GameState,
    rdr: &VGARenderer,
    loader: &dyn Loader,
    which: usize,
    name: &str,
    x: usize,
    y: usize,
) {
    let mut disk_anim = new_disk_anim(x, y);

    // Save bytes to a writer (need this for checksumming)
    let writer = &mut new_data_writer(game_file_size(level_state, game_state));

    let mut header = [0; SAVEGAME_NAME_LEN];
    let name_bytes = name.as_bytes();
    for i in 0..(min(name.len(), SAVEGAME_NAME_LEN - 1)) {
        header[i] = name_bytes[i];
    }
    writer.write_bytes(&header);

    disk_anim.disk_flop_anim(rdr);
    write_game_state(writer, game_state);
    let (offset, checksum) = do_write_checksum(writer, SAVEGAME_NAME_LEN, 0);

    write_level_ratios(writer, game_state);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            writer.write_u8(level_state.level.tile_map[x][y] as u8);
        }
    }
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            match level_state.actor_at[x][y] {
                At::Nothing => writer.write_u16(0),
                At::Wall(at_val) => writer.write_u16(at_val),
                At::Obj(ObjKey(at_val)) => {
                    // invalid for W3D, as it assumes a real pointer to the
                    // obj here. This makes the save game invalid for load
                    // in W3D.
                    writer.write_u16((at_val + 255) as u16); // + 255 so it will not be recognized as a wall
                }
            }
        }
    }
    let (_, checksum) = do_write_checksum(writer, offset, checksum);
    // returned offset will not be used, since the obj section is skipped in the checksum check

    for x in 0..NUM_AREAS {
        for y in 0..NUM_AREAS {
            writer.write_u8(level_state.area_connect[x][y]);
        }
    }

    for i in 0..NUM_AREAS {
        let v = if level_state.area_by_player[i] { 1 } else { 0 };
        writer.write_u16(v);
    }

    for obj in &level_state.actors {
        write_obj_type(writer, obj);
    }
    write_obj_type(writer, &null_obj_type());

    let offset = writer.offset(); // no checksum over obj_type, reset the offset
    writer.skip(2); //lastobjlist always nulled, not needed for iw
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    for i in 0..level_state.statics.len() {
        write_static(writer, &level_state.statics[i]);
    }
    writer.skip((MAX_STATS - level_state.statics.len()) * STAT_TYPE_LEN);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    for i in 0..level_state.doors.len() {
        writer.write_u16(level_state.doors[i].position);
    }
    writer.skip((MAX_DOORS - level_state.doors.len()) * 2);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    for door in &level_state.doors {
        write_door(writer, door);
    }
    writer.skip((MAX_DOORS - level_state.doors.len()) * DOOR_TYPE_LEN);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    writer.write_u16(game_state.push_wall_state as u16);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);
    writer.write_u16(game_state.push_wall_x as u16);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);
    writer.write_u16(game_state.push_wall_y as u16);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);
    writer.write_u16(game_state.push_wall_dir as u16);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);
    writer.write_i16(game_state.push_wall_pos as i16);
    let (_, checksum) = do_write_checksum(writer, offset, checksum);

    writer.write_i32(checksum);

    loader
        .save_save_game(which, &writer.data)
        .expect("save game saved")
}

// in bytes
fn game_file_size(level_state: &LevelState, game_state: &GameState) -> usize {
    SAVEGAME_NAME_LEN +
    66 + // GameState
    game_state.level_ratios.len() * LEVEL_RATIO_TYPE_LEN + // LevelRatios
    4096 + // tile_map
    8192 + // actor_at
    1369 + // area_connect
    74 + // area_by_player
    (level_state.actors.len() + 1) * OBJ_TYPE_LEN + // obj, +1 for the nullobj
    2 + // lastobjlist ptr
    3200 + // statics
    128 + // door positions
    640 + // doors
    14 // push wall states + checksum
}

fn write_game_state(writer: &mut DataWriter, game_state: &GameState) {
    writer.write_u16(game_state.difficulty as u16);
    writer.write_u16(game_state.map_on as u16);
    writer.write_i32(game_state.old_score);
    writer.write_i32(game_state.score);
    writer.write_i32(game_state.next_extra);
    writer.write_i16(game_state.lives as i16);
    writer.write_i16(game_state.health as i16);
    writer.write_i16(game_state.ammo as i16);
    writer.write_i16(game_state.keys as i16);

    writer.write_u16(game_state.best_weapon as u16);
    if let Some(weapon) = game_state.weapon {
        writer.write_u16(weapon as u16);
    } else {
        writer.write_u16(0);
    }
    writer.write_u16(game_state.chosen_weapon as u16);

    writer.write_u16(game_state.face_frame as u16);
    writer.write_u16(game_state.attack_frame as u16);
    writer.write_u16(game_state.attack_count as u16);
    writer.write_u16(game_state.weapon_frame as u16);

    writer.write_u16(game_state.episode as u16);
    writer.write_u16(game_state.secret_count as u16);
    writer.write_u16(game_state.treasure_count as u16);
    writer.write_u16(game_state.kill_count as u16);
    writer.write_u16(game_state.secret_total as u16);
    writer.write_u16(game_state.treasure_total as u16);
    writer.write_u16(game_state.kill_total as u16);

    writer.write_u32(game_state.time_count as u32);
    writer.write_u32(game_state.kill_x as u32);
    writer.write_u32(game_state.kill_y as u32);
    if game_state.victory_flag {
        writer.write_u16(1);
    } else {
        writer.write_u16(0);
    }
}

fn write_level_ratios(writer: &mut DataWriter, game_state: &GameState) {
    for ratio in &game_state.level_ratios {
        writer.write_u16(ratio.kill as u16);
        writer.write_u16(ratio.secret as u16);
        writer.write_u16(ratio.treasure as u16);
        writer.write_i32(ratio.time);
    }
}

fn write_obj_type(writer: &mut DataWriter, obj: &ObjType) {
    writer.write_i16(obj.active as i16);
    writer.write_u16(obj.tic_count as u16);
    writer.write_u16(obj.class as u16);
    writer.skip(2); // state ptr not restored, pointer values not available
    writer.write_u8(obj.flags);
    writer.skip(1); //padding flags

    writer.write_i32(obj.distance);
    writer.write_u16(obj.dir as u16);

    writer.write_i32(obj.x);
    writer.write_i32(obj.y);

    writer.write_u16(obj.tilex as u16);
    writer.write_u16(obj.tiley as u16);
    writer.write_u8(obj.area_number as u8);
    writer.skip(1);

    writer.write_i16(obj.view_x as i16);
    writer.write_u16(obj.view_height as u16);

    writer.write_i32(obj.trans_x.to_i32());
    writer.write_i32(obj.trans_y.to_i32());

    writer.write_u16(obj.angle as u16);

    writer.write_i16(obj.hitpoints as i16);
    writer.write_i32(obj.speed);
    writer.write_i16(obj.temp1 as i16);
    writer.write_i16(obj.temp2 as i16);
    writer.write_i16(obj.temp3 as i16);
    writer.skip(4); // next, prev pointer always nulled
}

fn write_static(writer: &mut DataWriter, stat: &StaticType) {
    writer.write_u8(stat.tile_x as u8);
    writer.write_u8(stat.tile_y as u8);
    writer.skip(2); //visspot not used in iw
    writer.write_u16(stat.sprite as u16);
    writer.write_u8(stat.flags);
    writer.write_u8(stat.item_number as u8);
}

fn write_door(writer: &mut DataWriter, door: &DoorType) {
    writer.write_u8(door.tile_x as u8);
    writer.write_u8(door.tile_y as u8);
    let vertical = if door.vertical { 1 } else { 0 };
    writer.write_u16(vertical);
    writer.write_u8(door.lock as u8);
    writer.skip(1); // padding
    writer.write_u16(door.action as u16);
    writer.write_u16(door.tic_count as u16);
}

// Returns true if the savegame file passed the checksum test, otherwise returns false.
pub async fn load_the_game(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    rdr: &VGARenderer,
    input: &Input,
    prj: &ProjectionConfig,
    assets: &Assets,
    loader: &dyn Loader,
    which: usize,
    x: usize,
    y: usize,
) {
    let checksums_matched = do_load(
        level_state,
        game_state,
        rdr,
        prj,
        assets,
        loader,
        which,
        x,
        y,
    );
    if !checksums_matched {
        message(rdr, win_state, &STR_SAVE_CHEAT);

        input.clear_keys_down();
        input.ack().await;
    }
}

pub fn do_load(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    assets: &Assets,
    loader: &dyn Loader,
    which: usize,
    x: usize,
    y: usize,
) -> bool {
    let mut disk_anim = new_disk_anim(x, y);
    let data = loader.load_save_game(which).expect("save game loaded");
    let reader = &mut new_data_reader_with_offset(&data, SAVEGAME_NAME_LEN); //first 32 bytes are savegame name

    // reconstruct GameState
    disk_anim.disk_flop_anim(rdr);
    load_game_state(reader, game_state);
    let (offset, checksum) = do_read_checksum(reader, SAVEGAME_NAME_LEN, 0);

    // reconstruct LevelRatio
    disk_anim.disk_flop_anim(rdr);
    load_level_ratios(reader, game_state);
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    disk_anim.disk_flop_anim(rdr);
    *level_state = setup_game_level(prj, game_state, assets).expect("set up game level"); // TODO replace expect with Quit()

    // load tilemap
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            let tile = reader.read_u8();
            level_state.level.tile_map[x][y] = tile as u16;
        }
    }
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    let mut at_vals = Vec::with_capacity(level_state.actors.len());
    // load actorat
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            let at_val = reader.read_u16();
            if at_val == 0 {
                level_state.actor_at[x][y] = At::Nothing;
            } else if at_val < 256 {
                level_state.actor_at[x][y] = At::Wall(at_val);
            } else {
                level_state.actor_at[x][y] = At::Obj(ObjKey(at_val as usize));
                at_vals.push(at_val);
            }
        }
    }
    // sorted at_vals should give a mapping from save game pointers to actors keys
    // used later in fixing up the "wrong" ObjKey values used above.
    at_vals.sort();
    let (_, checksum) = do_read_checksum(reader, offset, checksum);
    // returned offset will not be used, since the obj section is skipped in the checksum check

    for x in 0..NUM_AREAS {
        for y in 0..NUM_AREAS {
            level_state.area_connect[x][y] = reader.read_u8();
        }
    }

    for i in 0..NUM_AREAS {
        // boolean is an enum (= int = 16bit) in the original code
        let val = reader.read_u16();
        level_state.area_by_player[i] = val != 0;
    }

    let player = read_partial_obj_type(reader);
    copy_partial_obj_type(level_state.mut_player(), &player);

    let mut actors_loaded = Vec::with_capacity(level_state.actors.len());
    loop {
        disk_anim.disk_flop_anim(rdr);
        let actor = read_partial_obj_type(reader);
        if actor.active == ActiveType::BadObject {
            break;
        }
        actors_loaded.push(actor)
    }

    for i in 1..level_state.actors.len() {
        copy_partial_obj_type(&mut level_state.actors[i], &actors_loaded[i - 1]);
    }

    // fix up actor_at array pointers
    // if a original save game is loaded it contains C pointers that need
    // to be replaced by proper ObjKeys for iw
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            if let At::Obj(ObjKey(at_val)) = level_state.actor_at[x][y] {
                let ix = at_vals.iter().position(|val| (*val) as usize == at_val);
                level_state.actor_at[x][y] = At::Obj(ObjKey(
                    1 + ix.expect("at val not found for fix up of actor_at"),
                ));
            }
        }
    }

    let offset = reader.offset(); // no checksum over obj_type, reset the offset
    reader.skip(2); // laststatobj pointer, don't need that in iw
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    // only read the statics available in the level. The remaining
    // statics contain garbage and will be skipped
    for i in 0..level_state.statics.len() {
        level_state.statics[i] = read_static(reader);
    }
    reader.skip((MAX_STATS - level_state.statics.len()) * STAT_TYPE_LEN); // skip garbage static data
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    let mut door_positions = vec![0; level_state.doors.len()];
    for i in 0..level_state.doors.len() {
        let door_pos = reader.read_u16();
        door_positions[i] = door_pos;
    }
    reader.skip((MAX_DOORS - level_state.doors.len()) * 2); // skip garbage door position data
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    for i in 0..level_state.doors.len() {
        let mut door = read_door_type(reader, level_state.doors[i].num);
        door.position = door_positions[i];
        level_state.doors[i] = door;
    }
    reader.skip((MAX_DOORS - level_state.doors.len()) * DOOR_TYPE_LEN); // skip garbage door data
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    game_state.push_wall_state = reader.read_u16() as u64;
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);
    game_state.push_wall_x = reader.read_u16() as usize;
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);
    game_state.push_wall_y = reader.read_u16() as usize;
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);
    game_state.push_wall_dir =
        Dir::try_from(reader.read_u16() as usize).expect("valid door direction");
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);
    game_state.push_wall_pos = reader.read_i16() as i32;
    let (_, checksum) = do_read_checksum(reader, offset, checksum);

    let old_checksum = reader.read_i32();
    let checksums_matched = old_checksum == checksum;
    if !checksums_matched {
        game_state.score = 0;
        game_state.lives = 1;
        game_state.weapon = Some(WeaponType::Pistol);
        game_state.chosen_weapon = WeaponType::Pistol;
        game_state.best_weapon = WeaponType::Pistol;
        game_state.ammo = 8;
    }
    checksums_matched
}

fn read_door_type(reader: &mut DataReader, num: usize) -> DoorType {
    let tile_x = reader.read_u8() as usize;
    let tile_y = reader.read_u8() as usize;
    let vertical = reader.read_u16() != 0;
    let lock = reader.read_u8() as usize;
    reader.skip(1); // padding
    let action = reader.read_u16() as usize;
    let tic_count = reader.read_u16() as u32;
    DoorType {
        num,
        tile_x,
        tile_y,
        vertical,
        lock: DoorLock::try_from(lock).expect("DoorLock"),
        action: DoorAction::try_from(action).expect("DoorAction"),
        tic_count,
        position: 0, // will be filled in at a later point
    }
}

fn read_static(reader: &mut DataReader) -> StaticType {
    let tile_x = reader.read_u8() as usize;
    let tile_y = reader.read_u8() as usize;
    reader.skip(2); // visspot ptr is not used in iw
    let shape_num = reader.read_u16() as usize;
    let flags = reader.read_u8();
    let item_number = reader.read_u8() as usize;

    StaticType {
        tile_x,
        tile_y,
        sprite: Sprite::try_from(shape_num).expect("valid Sprite"),
        flags,
        item_number: StaticKind::try_from(item_number).expect("valid StaticType"),
    }
}

// Copies every field except the state.
fn copy_partial_obj_type(dst: &mut ObjType, src: &ObjType) {
    dst.active = src.active;
    dst.tic_count = src.tic_count;
    dst.class = src.class;
    dst.flags = src.flags;
    dst.distance = src.distance;
    dst.dir = src.dir;
    dst.x = src.x;
    dst.y = src.y;
    dst.tilex = src.tilex;
    dst.tiley = src.tiley;
    dst.area_number = src.area_number;
    dst.view_x = src.view_x;
    dst.view_height = src.view_height;
    dst.trans_x = src.trans_x;
    dst.trans_y = src.trans_y;
    dst.angle = src.angle;
    dst.hitpoints = src.hitpoints;
    dst.speed = src.speed;
    dst.temp1 = src.temp1;
    dst.temp2 = src.temp3;
    dst.temp3 = src.temp3;
}

pub fn null_obj_type() -> ObjType {
    ObjType {
        active: ActiveType::BadObject,
        tic_count: 0,
        class: ClassType::Nothing,
        state: None,
        flags: 0,
        distance: 0,
        dir: DirType::East, // East is 0, not NoDir
        x: 0,
        y: 0,
        tilex: 0,
        tiley: 0,
        area_number: 0,
        view_x: 0,
        view_height: 0,
        trans_x: new_fixed_u32(0),
        trans_y: new_fixed_u32(0),
        angle: 0,
        hitpoints: 0,
        speed: 0,
        temp1: 0,
        temp2: 0,
        temp3: 0,
        pitch: 0,
    }
}

// Read ObjType from file. 'state' is always excluded and set to 'None', since it
// is a pointer value that makes no nense in the Rust world.
fn read_partial_obj_type(reader: &mut DataReader) -> ObjType {
    let active = ActiveType::try_from(reader.read_i16()).expect("ActiveType");
    if active == ActiveType::BadObject {
        reader.skip(OBJ_TYPE_LEN - 2);
        return null_obj_type();
    }
    let tic_count = reader.read_u16() as u32;
    let class = ClassType::try_from(reader.read_u16() as usize).expect("ClassType");
    reader.skip(2); // state_ptr not restored
    let flags = reader.read_u8();
    reader.skip(1); // padding for flags

    let distance = reader.read_i32();
    let dir = DirType::try_from(reader.read_u16() as usize).expect("DirType");

    let x = reader.read_i32();
    let y = reader.read_i32();

    let tilex = reader.read_u16() as usize;
    let tiley = reader.read_u16() as usize;
    let area_number = reader.read_u8() as usize;
    reader.skip(1); // padding for area_number

    let view_x = reader.read_i16() as i32;
    let view_height = reader.read_u16() as i32;

    let tx_1 = reader.read_u16();
    let tx_2 = reader.read_u16();
    let trans_x = new_fixed_u16(tx_2, tx_1);

    let ty_1 = reader.read_u16();
    let ty_2 = reader.read_u16();
    let trans_y = new_fixed_u16(ty_2, ty_1);

    let angle = reader.read_u16() as i32;

    let hitpoints = reader.read_i16() as i32;
    let speed = reader.read_i32() as i32;
    let temp1 = reader.read_i16() as i32;
    let temp2 = reader.read_i16() as i32;
    let temp3 = reader.read_i16() as i32;

    reader.skip(4); // next and prev pointer we don't need in iw (each 2 bytes/1 word)

    ObjType {
        active,
        tic_count,
        class,
        state: None,
        flags,
        distance,
        dir,
        x,
        y,
        tilex,
        tiley,
        area_number,
        view_x,
        view_height,
        trans_x,
        trans_y,
        angle,
        hitpoints,
        speed,
        temp1,
        temp2,
        temp3,
        pitch: 0,
    }
}

fn load_game_state(reader: &mut DataReader, game_state: &mut GameState) {
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
        level_ratios.push(LevelRatio {
            kill,
            secret,
            treasure,
            time,
        })
    }

    game_state.level_ratios = level_ratios;
}

fn do_read_checksum(reader: &DataReader, prev_offset: usize, checksum_init: i32) -> (usize, i32) {
    let offset = reader.offset();
    let block = reader.slice(prev_offset, offset);
    checksum(offset, block, prev_offset, checksum_init)
}

fn do_write_checksum(writer: &DataWriter, prev_offset: usize, checksum_init: i32) -> (usize, i32) {
    let offset = writer.offset();
    let block: &[u8] = writer.slice(prev_offset, offset);
    checksum(offset, block, prev_offset, checksum_init)
}

fn checksum(offset: usize, block: &[u8], prev_offset: usize, checksum_init: i32) -> (usize, i32) {
    let mut checksum = checksum_init;
    for i in 0..((offset - prev_offset) - 1) {
        checksum += (block[i] ^ block[i + 1]) as i32;
    }
    (offset, checksum)
}
