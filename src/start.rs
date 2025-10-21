#[cfg(test)]
#[path = "./start_test.rs"]
mod start_test;

use std::cmp::min;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use std::usize;

use tokio::runtime::{self, Runtime};

use vga::input::NumCode;
use vga::{SCReg, VGA, VGABuilder};

use crate::act2::get_state_by_id;
use crate::assets::{self, GAMEPAL, GraphicNum, SIGNON};
use crate::config::{WolfConfig, check_timedemo_env};
use crate::def::{
    ActiveType, At, ClassType, Difficulty, Dir, DirType, DoorAction, DoorLock, DoorType, GameState,
    HEIGHT_RATIO, IWConfig, LevelRatio, LevelState, MAP_SIZE, MAX_DOORS, MAX_STATS, NUM_AREAS,
    ObjKey, ObjType, PLAYER_KEY, Sprite, StaticKind, StaticType, WeaponType, WindowState,
    new_game_state,
};
use crate::draw::{RayCast, init_ray_cast};
use crate::fixed::Fixed;
use crate::game::{game_loop, play_demo, setup_game_level};
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::menu::{
    MenuState, check_for_episodes, control_panel, initial_menu_state, intro_screen, intro_song,
    message,
};
use crate::play::{self, DEMO_TICS, ProjectionConfig, draw_play_border};
use crate::rc::{Input, RenderContext};
use crate::time;
use crate::us1::c_print;
use crate::util::{DataReader, DataWriter};
use crate::vl;
use crate::{config, sd};

const OBJ_TYPE_LEN: usize = 60;
const STAT_TYPE_LEN: usize = 8;
const DOOR_TYPE_LEN: usize = 10;
const LEVEL_RATIO_TYPE_LEN: usize = 10;
const SAVEGAME_NAME_LEN: usize = 32;

static STR_SAVE_CHEAT: &'static str = "Your Save Game file is,\nshall we say, \"corrupted\".\nBut I'll let you go on and\nplay anyway....";

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
    fn disk_flop_anim(&mut self, rc: &mut RenderContext, iw_config: &IWConfig) {
        if self.which {
            rc.pic(self.x, self.y, GraphicNum::CDISKLOADING2PIC)
        } else {
            rc.pic(self.x, self.y, GraphicNum::CDISKLOADING1PIC)
        }
        self.which = !self.which;

        rc.display();

        if !iw_config.options.fast_loading {
            std::thread::sleep(Duration::from_millis(40));
        }
    }
}

pub fn iw_start(loader: impl Loader + 'static, iw_config: IWConfig) -> Result<(), String> {
    let mut wolf_config = config::load_wolf_config(&loader);
    let patch_config = &loader.load_patch_config_file()?;

    let rt = tokio_runtime()?;
    let rt_ref = Arc::new(rt);

    let sound = sd::startup(rt_ref.clone())?;
    let assets = assets::load_assets(&sound, &loader)?;
    let (graphics, fonts, tiles, texts) = assets::load_all_graphics(&loader, patch_config)?;

    let ticker = time::new_ticker(rt_ref.clone());

    let mut win_state = initial_window_state();
    let mut menu_state = initial_menu_state(loader.variant());

    check_for_episodes(&mut menu_state, loader.variant());

    vga::util::spawn_async(async move {
        let mut vga = VGABuilder::new()
            .video_mode(0x13)
            .title("Iron Wolf".to_string())
            .fullscreen(iw_config.options.fullscreen)
            .build()
            .expect("vga build");
        //enable Mode Y
        let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
        vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

        let input = Input::init_player(&wolf_config);

        if let Some(which_demo) = check_timedemo_env() {
            let projection = init_projection(&wolf_config, &vga);
            let cast = init_ray_cast(projection.view_width);

            let mut rc = RenderContext::init(
                vga,
                ticker,
                graphics,
                fonts,
                tiles,
                texts,
                assets,
                loader.variant(),
                input,
                projection,
                sound,
            );

            let (_, abort, benchmark_result) = play_demo(
                &mut rc,
                &mut wolf_config,
                &iw_config,
                &mut win_state,
                &mut menu_state,
                cast,
                &loader,
                which_demo,
                true,
            )
            .await;

            if abort {
                println!("timedemo aborted")
            } else {
                let b = benchmark_result.expect("benchmark result");
                let r_fps = (b.real.as_secs_f32() / b.total.as_secs_f32()) * 70.0;

                let num_frames = b.total.as_secs_f32() * 70.0;
                let avg_frame = (b.unbounded.as_secs_f32() / num_frames) * DEMO_TICS as f32;
                let u_fps = 1.0 / avg_frame;
                println!("timedemo, total time: {:.2}s", b.total.as_secs_f32());
                println!(
                    "\treal time: {:.2}s, unbounded time: {:.2}s",
                    b.real.as_secs_f32(),
                    b.unbounded.as_secs_f32()
                );
                println!("\t{:.2} real fps, {:.2} unbounded fps", r_fps, u_fps);
                exit(0);
            }
        } else {
            let projection = init_projection(&wolf_config, &vga);
            let cast = init_ray_cast(projection.view_width);
            let mut rc = RenderContext::init(
                vga,
                ticker,
                graphics,
                fonts,
                tiles,
                texts,
                assets,
                loader.variant(),
                input,
                projection,
                sound,
            );

            init_game(&mut rc, &mut win_state).await;

            demo_loop(
                &mut rc,
                &mut wolf_config,
                &iw_config,
                cast,
                &mut win_state,
                &mut menu_state,
                &loader,
            )
            .await;
        }
    });

    Ok(())
}

pub fn tokio_runtime() -> Result<Runtime, String> {
    #[cfg(feature = "web")]
    let rt = runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| e.to_string())?;

    #[cfg(any(feature = "sdl", feature = "test"))]
    let rt = runtime::Runtime::new().map_err(|e| e.to_string())?;

    Ok(rt)
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

fn init_projection(wolf_config: &WolfConfig, vga: &VGA) -> ProjectionConfig {
    vl::set_palette(vga, GAMEPAL);
    new_view_size(wolf_config.viewsize)
}

async fn init_game(rc: &mut RenderContext, win_state: &mut WindowState) {
    signon_screen(&mut rc.vga);
    intro_screen(rc);
    // TODO InitRedShifts
    finish_signon(rc, win_state).await;
}

/// Returns width, height dimensions
fn dim_from_viewsize(view_size: u16) -> (usize, usize) {
    (
        (view_size * 16) as usize,
        (view_size as f64 * 16.0 * HEIGHT_RATIO) as usize,
    )
}

pub fn new_view_size(view_size: u16) -> ProjectionConfig {
    let (w, h) = dim_from_viewsize(view_size);
    play::calc_projection(w, h)
}

pub fn show_view_size(rc: &mut RenderContext, view_size: u16) {
    let (w, h) = dim_from_viewsize(view_size);
    draw_play_border(rc, w, h);
}

async fn finish_signon(rc: &mut RenderContext, win_state: &mut WindowState) {
    let peek = rc.vga.read_mem(0);
    rc.bar(0, 189, 300, 11, peek);

    win_state.window_x = 0;
    win_state.window_w = 320;
    win_state.print_y = 190;
    win_state.set_font_color(14, 4);
    c_print(rc, win_state, "Press a key");

    rc.ack();

    rc.bar(0, 189, 300, 11, peek);
    win_state.print_y = 190;
    win_state.set_font_color(10, 4);
    c_print(rc, win_state, "Working...");

    win_state.set_font_color(0, 15);

    rc.display();
}

async fn demo_loop(
    rc: &mut RenderContext,
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    cast_param: RayCast,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    loader: &dyn Loader,
) {
    rc.sound
        .play_music(intro_song(loader.variant()), &rc.assets, loader);

    if !iw_config.options.no_wait {
        pg_13(rc).await;
    }

    let mut last_demo = 0;

    let mut cast = cast_param;
    loop {
        while !iw_config.options.no_wait {
            // title screen & demo loop
            rc.pic(0, 0, GraphicNum::TITLEPIC);
            rc.fade_in().await;
            if rc.wait_user_input(time::TICK_BASE * 15) {
                break;
            }
            rc.fade_out().await;

            // credits page
            rc.pic(0, 0, GraphicNum::CREDITSPIC);
            rc.fade_in().await;
            if rc.wait_user_input(time::TICK_BASE * 10) {
                break;
            }
            rc.fade_out().await;

            // high scores
            draw_high_scores(rc, win_state, &wolf_config.high_scores);
            rc.fade_in().await;
            if rc.wait_user_input(time::TICK_BASE * 10) {
                break;
            }

            // demo
            let (cast_demo, abort, _) = play_demo(
                rc,
                wolf_config,
                iw_config,
                win_state,
                menu_state,
                cast,
                loader,
                last_demo,
                false,
            )
            .await;
            cast = cast_demo;
            last_demo = (last_demo + 1) % 4;

            rc.set_buffer_offset(rc.active_buffer());

            if abort {
                break;
            }
        }

        rc.fade_out().await;

        let mut game_state = new_game_state();
        let mut level_state =
            setup_game_level(&mut game_state, &rc.assets, false).expect("setup game level");

        // TODO RecordDemo()
        let update = control_panel(
            rc,
            wolf_config,
            iw_config,
            &mut level_state,
            &mut game_state,
            cast,
            win_state,
            menu_state,
            loader,
            NumCode::None,
        )
        .await;
        cast = update.ray_cast;

        cast = game_loop(
            rc,
            wolf_config,
            iw_config,
            &mut level_state,
            &mut game_state,
            cast,
            win_state,
            menu_state,
            loader,
        )
        .await;
        rc.fade_out().await;
    }
}

fn signon_screen(vga: &mut VGA) {
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

async fn pg_13(rc: &mut RenderContext) {
    rc.fade_out().await;
    rc.bar(0, 0, 320, 200, 0x82);
    rc.pic(216, 110, GraphicNum::PG13PIC);

    rc.fade_in().await;
    rc.wait_user_input(time::TICK_BASE * 7);
    rc.fade_out().await;
}

pub fn quit(err: Option<&str>) -> ! {
    // TODO print error screen, wait for button press and the exit(0)
    println!("TODO draw exit screen, err = {:?}", err);
    exit(0)
}

pub fn save_the_game(
    rc: &mut RenderContext,
    iw_config: &IWConfig,
    level_state: &LevelState,
    game_state: &GameState,
    loader: &dyn Loader,
    which: usize,
    name: &str,
    x: usize,
    y: usize,
) {
    let mut disk_anim = new_disk_anim(x, y);

    // Save bytes to a writer (need this for checksuming)
    let writer = &mut DataWriter::new(game_file_size(level_state, game_state));

    let mut header = [0; SAVEGAME_NAME_LEN];
    let name_bytes = name.as_bytes();
    for i in 0..(min(name.len(), SAVEGAME_NAME_LEN - 1)) {
        header[i] = name_bytes[i];
    }
    writer.write_bytes(&header);

    disk_anim.disk_flop_anim(rc, iw_config);
    write_game_state(writer, game_state);
    let (offset, checksum) = do_write_checksum(writer, SAVEGAME_NAME_LEN, 0);

    disk_anim.disk_flop_anim(rc, iw_config);
    write_level_ratios(writer, game_state);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            writer.write_u8(level_state.level.tile_map[x][y] as u8);
        }
    }
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
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

    for i in 0..level_state.actors.len() {
        let k = ObjKey(i);
        disk_anim.disk_flop_anim(rc, iw_config);
        if level_state.actors.exists(k) {
            write_obj_type(writer, level_state.obj(k));
        }
    }
    disk_anim.disk_flop_anim(rc, iw_config);
    write_obj_type(writer, &null_obj_type());

    let offset = writer.offset(); // no checksum over obj_type, reset the offset
    writer.write_u16(level_state.statics.len() as u16);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
    for i in 0..level_state.statics.len() {
        write_static(writer, &level_state.statics[i]);
    }
    writer.skip((MAX_STATS - level_state.statics.len()) * STAT_TYPE_LEN);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
    for i in 0..level_state.doors.len() {
        writer.write_u16(level_state.doors[i].position);
    }
    writer.skip((MAX_DOORS - level_state.doors.len()) * 2);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
    for door in &level_state.doors {
        write_door(writer, door);
    }
    writer.skip((MAX_DOORS - level_state.doors.len()) * DOOR_TYPE_LEN);
    let (offset, checksum) = do_write_checksum(writer, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
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
    writer.write_u32(game_state.old_score);
    writer.write_u32(game_state.score);
    writer.write_u32(game_state.next_extra);
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
    if let Some(state) = obj.state {
        writer.write_u16(state.id);
    } else {
        writer.write_u16(0);
    }
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
    rc: &mut RenderContext,
    iw_config: &IWConfig,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    loader: &dyn Loader,
    which: usize,
    x: usize,
    y: usize,
) {
    rc.fade_in().await;

    let checksums_matched = do_load(rc, iw_config, level_state, game_state, loader, which, x, y);
    if !checksums_matched {
        message(rc, win_state, &STR_SAVE_CHEAT);

        rc.clear_keys_down();
        rc.ack();
    }
}

pub fn do_load(
    rc: &mut RenderContext,
    iw_config: &IWConfig,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    loader: &dyn Loader,
    which: usize,
    x: usize,
    y: usize,
) -> bool {
    let mut disk_anim = new_disk_anim(x, y);
    let data = loader.load_save_game(which).expect("save game loaded");
    let reader = &mut DataReader::new_with_offset(&data, SAVEGAME_NAME_LEN); //first 32 bytes are savegame name

    // reconstruct GameState
    disk_anim.disk_flop_anim(rc, iw_config);
    load_game_state(reader, game_state);
    let (offset, checksum) = do_read_checksum(reader, SAVEGAME_NAME_LEN, 0);

    // reconstruct LevelRatio
    disk_anim.disk_flop_anim(rc, iw_config);
    load_level_ratios(reader, game_state);
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    disk_anim.disk_flop_anim(rc, iw_config);
    *level_state = setup_game_level(game_state, &rc.assets, false).expect("set up game level"); // TODO replace expect with Quit()

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
                if !at_vals.contains(&at_val) {
                    level_state.actor_at[x][y] = At::Obj(ObjKey(at_val as usize));
                    at_vals.push(at_val);
                } else {
                    level_state.actor_at[x][y] = At::Nothing;
                }
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

    let player = read_obj_type(reader);
    level_state.actors.put_obj(PLAYER_KEY, player);

    let mut i = 1;
    loop {
        disk_anim.disk_flop_anim(rc, iw_config);
        let actor = read_obj_type(reader);
        if actor.active == ActiveType::BadObject {
            break;
        }
        level_state.actors.put_obj(ObjKey(i), actor);
        i += 1;
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
    let mut statics_len = reader.read_u16() as usize;
    if statics_len > MAX_STATS {
        statics_len = level_state.statics.len();
    }
    let (offset, checksum) = do_read_checksum(reader, offset, checksum);

    // only read the statics available in the save file. The remaining
    // statics contain garbage and will be skipped
    for i in 0..statics_len {
        if i < level_state.statics.len() {
            level_state.statics[i] = read_static(reader);
        } else {
            level_state.statics.push(read_static(reader));
        }
    }
    reader.skip((MAX_STATS - statics_len) * STAT_TYPE_LEN); // skip garbage static data
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
    let mut shape_num = reader.read_u16() as usize;
    let flags = reader.read_u8();
    let item_number = reader.read_u8() as usize;

    if shape_num == u16::MAX as usize {
        shape_num = usize::MAX;
    }

    StaticType {
        tile_x,
        tile_y,
        sprite: Sprite::try_from(shape_num).expect("valid Sprite"),
        flags,
        item_number: StaticKind::try_from(item_number).expect("valid StaticType"),
    }
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
        trans_x: Fixed::new_from_u32(0),
        trans_y: Fixed::new_from_u32(0),
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
fn read_obj_type(reader: &mut DataReader) -> ObjType {
    let active = ActiveType::try_from(reader.read_i16()).expect("ActiveType");
    if active == ActiveType::BadObject {
        reader.skip(OBJ_TYPE_LEN - 2);
        return null_obj_type();
    }
    let tic_count = reader.read_u16() as i32;
    let class = ClassType::try_from(reader.read_u16() as usize).expect("ClassType");

    let state_id = reader.read_u16();
    let state = get_state_by_id(state_id);

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
    let trans_x = Fixed::new_from_u16(tx_2, tx_1);

    let ty_1 = reader.read_u16();
    let ty_2 = reader.read_u16();
    let trans_y = Fixed::new_from_u16(ty_2, ty_1);

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
        state,
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
    game_state.difficulty = Difficulty::from_pos(reader.read_u16() as usize);
    game_state.map_on = reader.read_u16() as usize;
    game_state.old_score = reader.read_u32();
    game_state.score = reader.read_u32();
    game_state.next_extra = reader.read_u32();
    game_state.lives = reader.read_i16() as i32;
    game_state.health = reader.read_i16() as i32;
    game_state.ammo = reader.read_i16() as i32;
    game_state.keys = reader.read_i16() as i32;

    game_state.best_weapon = WeaponType::from_usize(reader.read_u16() as usize);
    game_state.weapon = Some(WeaponType::from_usize(reader.read_u16() as usize));
    game_state.chosen_weapon = WeaponType::from_usize(reader.read_u16() as usize);

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
