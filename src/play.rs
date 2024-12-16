#[cfg(test)]
#[path = "./play_test.rs"]
mod play_test;

use vga::input::NumCode;
use vga::util;
use vga::VGA;

use crate::act1::{move_doors, move_push_walls};
use crate::agent::{
    draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_score, draw_weapon,
};
use crate::assets::Music;
use crate::assets::{GraphicNum, GAMEPAL};
use crate::config::WolfConfig;
use crate::debug::debug_keys;
use crate::def::ActiveType;
use crate::def::ObjType;
use crate::def::WindowState;
use crate::def::{
    Assets, At, Button, Control, ControlState, GameState, LevelState, ObjKey, PlayState, ANGLES,
    ANGLE_QUAD, FINE_ANGLES, FL_NEVERMARK, FL_NONMARK, FOCAL_LENGTH, GLOBAL1, NUM_BUTTONS,
    SCREENLOC, STATUS_LINES, TILEGLOBAL,
};
use crate::draw::{three_d_refresh, RayCast};
use crate::fixed::{new_fixed, new_fixed_u32, Fixed};
use crate::input;
use crate::input::Input;
use crate::inter::clear_split_vwb;
use crate::loader::Loader;
use crate::menu::control_panel;
use crate::menu::message;
use crate::menu::Menu;
use crate::menu::MenuState;
use crate::menu::SaveLoadGame;
use crate::menu::LSA_X;
use crate::menu::LSA_Y;
use crate::scale::{setup_scaling, CompiledScaler};
use crate::sd::Sound;
use crate::start::load_the_game;
use crate::start::save_the_game;
use crate::time;
use crate::us1::draw_window;
use crate::util::check_param;
use crate::vga_render::VGARenderer;
use crate::vl::set_palette;

//TODO separate draw.c stuff from play.c stuff in here

const MIN_DIST: i32 = 0x5800;

const HEIGHT_RATIO: f32 = 0.5;
const SCREEN_WIDTH: usize = 80;

const PI: f32 = 3.141592657;

const ANG90: usize = FINE_ANGLES / 4;
const ANG180: usize = ANG90 * 2;

const NUM_FINE_TANGENTS: usize = FINE_ANGLES / 2 + ANG180;
const VIEW_GLOBAL: usize = 0x10000;
const RAD_TO_INT: f64 = FINE_ANGLES as f64 / 2.0 / std::f64::consts::PI;

const RUN_MOVE: u64 = 70;
const BASE_MOVE: u64 = 35;

const NUM_RED_SHIFTS: usize = 6;
const RED_STEPS: i32 = 8;

const NUM_WHITE_SHIFTS: usize = 3;
const WHITE_STEPS: i32 = 20;
const WHITE_TICS: i32 = 6;

static BUTTON_SCAN: [NumCode; NUM_BUTTONS] = [
    NumCode::Control,
    NumCode::Alt,
    NumCode::RShift,
    NumCode::Space,
    NumCode::Num1,
    NumCode::Num2,
    NumCode::Num3,
    NumCode::Num4,
];

// LIST OF SONGS FOR EACH VERSION
pub static SONGS: [Music; 6 * 10] = [
    //
    // Episode One
    //
    Music::GETTHEM,
    Music::SEARCHN,
    Music::POW,
    Music::SUSPENSE,
    Music::GETTHEM,
    Music::SEARCHN,
    Music::POW,
    Music::SUSPENSE,
    Music::WARMARCH, // Boss level
    Music::CORNER,   // Secret level
    //
    // Episode Two
    //
    Music::NAZIOMI,
    Music::PREGNANT,
    Music::GOINGAFT,
    Music::HEADACHE,
    Music::NAZIOMI,
    Music::PREGNANT,
    Music::HEADACHE,
    Music::GOINGAFT,
    Music::WARMARCH, // Boss level
    Music::DUNGEON,  // Secret level
    //
    // Episode Three
    //
    Music::INTROCW3,
    Music::NAZIRAP,
    Music::TWELFTH,
    Music::ZEROHOUR,
    Music::INTROCW3,
    Music::NAZIRAP,
    Music::TWELFTH,
    Music::ZEROHOUR,
    Music::ULTIMATE, // Boss level
    Music::PACMAN,   // Secret level
    //
    // Episode Four
    //
    Music::GETTHEM,
    Music::SEARCHN,
    Music::POW,
    Music::SUSPENSE,
    Music::GETTHEM,
    Music::SEARCHN,
    Music::POW,
    Music::SUSPENSE,
    Music::WARMARCH, // Boss level
    Music::CORNER,   // Secret level
    //
    // Episode Five
    //
    Music::NAZIOMI,
    Music::PREGNANT,
    Music::GOINGAFT,
    Music::HEADACHE,
    Music::NAZIOMI,
    Music::PREGNANT,
    Music::HEADACHE,
    Music::GOINGAFT,
    Music::WARMARCH, // Boss level
    Music::DUNGEON,  // Secret level
    //
    // Episode Six
    //
    Music::INTROCW3,
    Music::NAZIRAP,
    Music::TWELFTH,
    Music::ZEROHOUR,
    Music::INTROCW3,
    Music::NAZIRAP,
    Music::TWELFTH,
    Music::ZEROHOUR,
    Music::ULTIMATE, // Boss level
    Music::FUNKYOU,  // Secret level
];

// TODO red/and whiteshifts as static array that is initialised in main?
// Or allocate in main func and supply to Update func!

struct ColourShifts {
    pub red_shifts: [[u8; 768]; NUM_RED_SHIFTS],
    pub white_shifts: [[u8; 768]; NUM_WHITE_SHIFTS],
}

pub struct ProjectionConfig {
    pub view_width: usize,
    pub view_height: usize,
    pub center_x: usize,
    pub shoot_delta: usize,
    pub screenofs: usize,
    pub height_numerator: i32,
    pub pixelangle: Vec<i32>,
    pub sines: Vec<Fixed>,
    pub fine_tangents: [i32; NUM_FINE_TANGENTS],
    pub scale: i32,
    pub scaler: CompiledScaler,
}

impl ProjectionConfig {
    pub fn sin(&self, ix: usize) -> Fixed {
        self.sines[ix]
    }

    pub fn cos(&self, ix: usize) -> Fixed {
        self.sines[ix + ANGLE_QUAD as usize]
    }
}

pub fn new_control_state() -> ControlState {
    ControlState {
        control: Control { x: 0, y: 0 },
        angle_frac: 0,
        button_held: [false; NUM_BUTTONS],
        button_state: [false; NUM_BUTTONS],
    }
}

pub fn calc_projection(view_size: usize) -> ProjectionConfig {
    let view_width = (view_size * 16) & !15;
    let view_height = ((((view_size * 16) as f32 * HEIGHT_RATIO) as u16) & !1) as usize;
    let center_x: usize = view_width / 2 - 1;
    let shoot_delta = view_width / 10;
    let screenofs = (200 - STATUS_LINES - view_height) / 2 * SCREEN_WIDTH + (320 - view_width) / 8;
    let half_view = view_width / 2;

    let face_dist = FOCAL_LENGTH + MIN_DIST;

    let pixelangle = calc_pixelangle(view_width, face_dist as f64);
    let sines = calc_sines();
    let fine_tangents = calc_fine_tangents();

    let scale = half_view as i32 * face_dist / (VIEW_GLOBAL as i32 / 2);
    let height_numerator = (TILEGLOBAL * scale) >> 6;

    let scaler = setup_scaling((view_width as f32 * 1.5) as usize, view_height);

    ProjectionConfig {
        view_width,
        view_height,
        center_x,
        shoot_delta,
        screenofs,
        height_numerator,
        pixelangle,
        sines,
        fine_tangents,
        scale,
        scaler,
    }
}

fn calc_fine_tangents() -> [i32; NUM_FINE_TANGENTS] {
    let mut tangents = [0; FINE_ANGLES];
    for i in 0..FINE_ANGLES / 8 {
        let tang = ((i as f64 + 0.5) / RAD_TO_INT).tan();
        tangents[i] = (tang * TILEGLOBAL as f64) as i32;
        tangents[FINE_ANGLES / 4 - 1 - i] = (1.0 / tang * TILEGLOBAL as f64) as i32;
    }
    tangents
}

fn calc_pixelangle(view_width: usize, face_dist: f64) -> Vec<i32> {
    let half_view = view_width / 2;

    let mut pixelangles = vec![0; view_width as usize];
    for i in 0..half_view {
        let tang = ((i * VIEW_GLOBAL) as f64 / view_width as f64) / face_dist;
        let angle = (tang.atan() * RAD_TO_INT) as i32;
        pixelangles[half_view - 1 - i] = angle;
        pixelangles[half_view + i] = -angle;
    }

    pixelangles
}

fn calc_sines() -> Vec<Fixed> {
    //TODO_VANILLA +1?? Bug in the original? does it write outside the array there in the original?
    let mut sines: Vec<Fixed> = vec![new_fixed(0, 0); ANGLES + ANGLE_QUAD + 1];

    let mut angle: f32 = 0.0;
    let angle_step = PI / 2.0 / ANGLE_QUAD as f32;
    for i in 0..=ANGLE_QUAD {
        let value: u32 = (GLOBAL1 as f32 * angle.sin()) as u32;
        //TODO ugly fixes in here, make this exact to the old c-code
        let v_fixed = new_fixed_u32(value.min(65535));
        let mut value_neg = value | 0x80000000u32;
        if i == 90 {
            //otherwise a ??rounding error?? occurs and walking
            //backward does not work anymore (TODO Fix this proper,
            //latest in the generalisation)
            value_neg -= 1;
        }
        let v_fixed_neg = new_fixed_u32(value_neg);
        sines[i] = v_fixed;
        sines[i + ANGLES] = v_fixed;
        sines[ANGLES / 2 - i] = v_fixed;
        sines[ANGLES - i] = v_fixed_neg;
        sines[ANGLES / 2 + i] = v_fixed_neg;
        angle += angle_step;
    }
    sines
}

pub async fn play_loop(
    wolf_config: &mut WolfConfig,
    ticker: &time::Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    control_state: &mut ControlState,
    vga: &VGA,
    sound: &mut Sound,
    rc: &mut RayCast,
    rdr: &VGARenderer,
    input: &input::Input,
    prj: &ProjectionConfig,
    assets: &Assets,
    loader: &dyn Loader,
    save_load_param: Option<SaveLoadGame>,
) {
    let mut save_load = save_load_param;
    let shifts = init_colour_shifts();

    game_state.play_state = PlayState::StillPlaying;
    // TODO frameon = 0??
    // TODO running = false?
    // TODO anglefrac = 0?
    game_state.face_count = 0;
    // TODO funnyticcount = 0?
    // TODO lasttimeout = 0??
    ticker.clear_count();
    input.clear_keys_down();
    clear_palette_shifts(game_state);

    handle_save_load(
        level_state,
        game_state,
        win_state,
        rdr,
        input,
        prj,
        assets,
        loader,
        save_load,
    )
    .await;

    {
        // TODO Debug!
        game_state.god_mode = true;
        /*
        if game_state.episode == 0 && game_state.map_on == 0 {
            let player = level_state.mut_player();
            player.x = 1465555;
            player.y = 3112211;
            player.angle = 0;
        }*/
        /*
        if game_state.episode == 0 && game_state.map_on == 0 {
            let player = level_state.mut_player();
            player.x = 2013924;
            player.y = 2163760;
            player.angle = 50;
        }*/

        /*
        if game_state.episode == 0 && game_state.map_on == 1 {
            let player = level_state.mut_player();
            player.x = 3019722;
            player.y = 224653;
            player.angle = 0;
        }*/
    }

    //TODO A lot to do here (clear palette, poll controls, prepare world)
    while game_state.play_state == PlayState::StillPlaying {
        // TODO replace this very inefficient calc_tic function. It waits
        // for a tic to happen which burns a lot of cycles on fast CPU.
        // Completely get rid of the tick thread and compute ticks through
        // timings from the rendering loop.
        // Also make rendering independent from everything that is based on tics!!
        // (call poll_controls and do_actors only with 70Hz!)
        let tics = ticker.calc_tics();

        poll_controls(control_state, tics, input);

        move_doors(level_state, game_state, sound, assets, tics);
        move_push_walls(level_state, game_state, tics);

        for i in 0..level_state.actors.len() {
            do_actor(
                ObjKey(i),
                tics,
                level_state,
                game_state,
                sound,
                rdr,
                control_state,
                prj,
                assets,
            );
        }

        update_palette_shifts(game_state, vga, &shifts, tics).await;

        three_d_refresh(ticker, game_state, level_state, rc, rdr, sound, prj, assets).await;

        save_load = check_keys(
            wolf_config,
            ticker,
            sound,
            rdr,
            assets,
            win_state,
            menu_state,
            game_state,
            level_state.player(),
            input,
            prj,
            loader,
        )
        .await;
        handle_save_load(
            level_state,
            game_state,
            win_state,
            rdr,
            input,
            prj,
            assets,
            loader,
            save_load,
        )
        .await;

        game_state.time_count += tics;

        // TODO SD_Poll() ?
        // TODO UpdateSoundLoc

        let offset_prev = rdr.buffer_offset();
        for i in 0..3 {
            rdr.set_buffer_offset(SCREENLOC[i]);
        }
        rdr.set_buffer_offset(offset_prev);
    }
}

async fn handle_save_load(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    rdr: &VGARenderer,
    input: &Input,
    prj: &ProjectionConfig,
    assets: &Assets,
    loader: &dyn Loader,
    save_load: Option<SaveLoadGame>,
) {
    if let Some(what) = save_load {
        match what {
            SaveLoadGame::Load(which) => {
                game_state.loaded_game = true;
                load_the_game(
                    level_state,
                    game_state,
                    win_state,
                    rdr,
                    input,
                    prj,
                    assets,
                    loader,
                    which,
                    LSA_X + 8,
                    LSA_Y + 5,
                )
                .await;
            }
            SaveLoadGame::Save(which, name) => {
                save_the_game(
                    level_state,
                    game_state,
                    rdr,
                    loader,
                    which,
                    &name,
                    LSA_X + 8,
                    LSA_Y + 5,
                );
            }
        }
    }
}

fn do_actor(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    if level_state.obj(k).active == ActiveType::No
        && !level_state.area_by_player[level_state.obj(k).area_number]
    {
        return;
    }

    if level_state.obj(k).flags & (FL_NONMARK | FL_NEVERMARK) == 0 {
        let (tilex, tiley) = {
            let obj = level_state.obj(k);
            (obj.tilex, obj.tiley)
        };
        level_state.actor_at[tilex][tiley] = At::Nothing;
    }

    // non transitional object

    if level_state.obj(k).tic_count == 0 {
        if let Some(think) = level_state.obj(k).state.expect("state").think {
            think(
                k,
                tics,
                level_state,
                game_state,
                sound,
                rdr,
                control_state,
                prj,
                assets,
            );
            if level_state.obj(k).state.is_none() {
                return;
            }
        }

        let (tilex, tiley, flags) = {
            let obj = level_state.obj(k);
            (obj.tilex, obj.tiley, obj.flags)
        };
        if flags & FL_NEVERMARK != 0 {
            return;
        }
        if flags & FL_NONMARK != 0 && level_state.actor_at[tilex][tiley] != At::Nothing {
            return;
        }
        level_state.actor_at[tilex][tiley] = At::Obj(k);
        return;
    }

    // transitional object
    level_state.update_obj(k, |obj| {
        obj.tic_count = obj.tic_count.saturating_sub(tics as u32)
    });
    while level_state.obj(k).tic_count <= 0 {
        if let Some(action) = level_state.obj(k).state.expect("state").action {
            action(
                k,
                tics,
                level_state,
                game_state,
                sound,
                rdr,
                control_state,
                prj,
                assets,
            );
            if level_state.obj(k).state.is_none() {
                return;
            }
        }

        level_state.update_obj(k, |obj| obj.state = obj.state.expect("state").next);
        if level_state.obj(k).state.is_none() {
            return;
        }

        if level_state.obj(k).state.expect("state").tic_time == 0 {
            level_state.update_obj(k, |obj| obj.tic_count = 0);
            break; // think a last time below
        }
        level_state.update_obj(k, |obj| obj.tic_count += obj.state.expect("state").tic_time);
    }

    if let Some(think) = level_state.obj(k).state.expect("state").think {
        think(
            k,
            tics,
            level_state,
            game_state,
            sound,
            rdr,
            control_state,
            prj,
            assets,
        );
        if level_state.obj(k).state.is_none() {
            return;
        }
    }

    let (tilex, tiley, flags) = {
        let obj = level_state.obj(k);
        (obj.tilex, obj.tiley, obj.flags)
    };
    if flags & FL_NEVERMARK != 0 {
        return;
    }
    if flags & FL_NONMARK != 0 && level_state.actor_at[tilex][tiley] != At::Nothing {
        return;
    }
    level_state.actor_at[tilex][tiley] = At::Obj(k);
}

pub async fn draw_play_screen(state: &GameState, rdr: &VGARenderer, prj: &ProjectionConfig) {
    rdr.fade_out().await;

    let offset_prev = rdr.buffer_offset();
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        draw_play_border(rdr, prj);
        rdr.pic(0, 200 - STATUS_LINES, GraphicNum::STATUSBARPIC);
    }
    rdr.set_buffer_offset(offset_prev);

    draw_face(state, rdr);
    draw_health(state, rdr);
    draw_lives(state, rdr);
    draw_level(state, rdr);
    draw_ammo(state, rdr);
    draw_keys(state, rdr);
    draw_weapon(state, rdr);
    draw_score(state, rdr);
}

fn draw_all_play_border_sides(rdr: &VGARenderer, prj: &ProjectionConfig) {
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        draw_play_border_side(rdr, prj);
    }
}

/// To fix window overwrites
fn draw_play_border_side(rdr: &VGARenderer, prj: &ProjectionConfig) {
    let xl = 160 - prj.view_width / 2;
    let yl = (200 - STATUS_LINES - prj.view_height) / 2;

    rdr.bar(0, 0, xl - 1, 200 - STATUS_LINES, 127);
    rdr.bar(xl + prj.view_width + 1, 0, xl - 2, 200 - STATUS_LINES, 127);

    vw_vlin(rdr, yl - 1, yl + prj.view_height, xl - 1, 0);
    vw_vlin(rdr, yl - 1, yl + prj.view_height, xl + prj.view_width, 125);
}

pub fn draw_all_play_border(rdr: &VGARenderer, prj: &ProjectionConfig) {
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        draw_play_border(rdr, prj);
    }
}

pub fn draw_play_border(rdr: &VGARenderer, prj: &ProjectionConfig) {
    //clear the background:
    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);

    let xl = 160 - prj.view_width / 2;
    let yl = (200 - STATUS_LINES - prj.view_height) / 2;

    //view area
    rdr.bar(xl, yl, prj.view_width, prj.view_height, 127);

    //border around the view area
    vw_hlin(rdr, xl - 1, xl + prj.view_width, yl - 1, 0);
    vw_hlin(rdr, xl - 1, xl + prj.view_width, yl + prj.view_height, 125);
    vw_vlin(rdr, yl - 1, yl + prj.view_height, xl - 1, 0);
    vw_vlin(rdr, yl - 1, yl + prj.view_height, xl + prj.view_width, 125);

    rdr.plot(xl - 1, yl + prj.view_height, 124);
}

fn vw_hlin(rdr: &VGARenderer, x: usize, z: usize, y: usize, c: u8) {
    rdr.hlin(x, y, (z - x) + 1, c)
}

fn vw_vlin(rdr: &VGARenderer, y: usize, z: usize, x: usize, c: u8) {
    rdr.vlin(x, y, (z - y) + 1, c)
}

///	Generates a window of a given width & height in the
/// middle of the screen
pub fn center_window(rdr: &VGARenderer, win_state: &mut WindowState, width: usize, height: usize) {
    draw_window(
        rdr,
        win_state,
        ((320 / 8) - width) / 2,
        ((160 / 8) - height) / 2,
        width,
        height,
    );
}

async fn check_keys(
    wolf_config: &mut WolfConfig,
    ticker: &time::Ticker,
    sound: &mut Sound,
    rdr: &VGARenderer,
    assets: &Assets,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    game_state: &mut GameState,
    player: &ObjType,
    input: &input::Input,
    prj: &ProjectionConfig,
    loader: &dyn Loader,
) -> Option<SaveLoadGame> {
    if input.key_pressed(NumCode::BackSpace)
        && input.key_pressed(NumCode::LShift)
        && input.key_pressed(NumCode::Alt)
        && check_param("goobers")
    {
        clear_split_vwb(win_state);

        message(rdr, win_state, "Debugging keys are\nnow available!");
        input.clear_keys_down();
        input.ack().await;
        win_state.debug_ok = true;
        return None;
    }

    let scan = input.last_scan();

    // TODO Check for FX keys pressed (let scan = input.one_of_key_pressed())
    if scan == NumCode::Escape {
        rdr.fade_out().await;
        menu_state.select_menu(Menu::Top);
        let prev_buffer = rdr.buffer_offset();
        rdr.set_buffer_offset(rdr.active_buffer());
        let save_load = control_panel(
            wolf_config,
            ticker,
            game_state,
            sound,
            rdr,
            input,
            assets,
            win_state,
            menu_state,
            loader,
            scan,
        )
        .await;
        rdr.set_buffer_offset(prev_buffer);

        win_state.set_font_color(0, 15);
        input.clear_keys_down();
        draw_play_screen(game_state, rdr, prj).await;
        //TODO stargame and loadedgame handling
        rdr.fade_in().await;
        return save_load;
    }

    if input.key_pressed(NumCode::Tab) && win_state.debug_ok {
        let prev_buffer = rdr.buffer_offset();
        rdr.set_buffer_offset(rdr.active_buffer());
        debug_keys(rdr, win_state, game_state, player, input).await;
        rdr.set_buffer_offset(prev_buffer);
        return None;
    }
    return None;
}

// reads input delta since last tic and manipulates the player state
fn poll_controls(state: &mut ControlState, tics: u64, input: &input::Input) {
    state.control.x = 0;
    state.control.y = 0;
    state.button_held.copy_from_slice(&state.button_state);

    poll_keyboard_buttons(state, input);

    poll_keyboard_move(state, input, tics);
    //TODO Mouse Move
    //TODO Joystick Move?

    //bound movement to a maximum
    let max = 100 * tics as i32;
    let min = -max;

    if state.control.x > max {
        state.control.x = max;
    } else if state.control.x < min {
        state.control.x = min;
    }

    if state.control.y > max {
        state.control.y = max;
    } else if state.control.y < min {
        state.control.y = min;
    }
}

fn poll_keyboard_buttons(state: &mut ControlState, input: &input::Input) {
    for i in 0..NUM_BUTTONS {
        state.button_state[i] = input.key_pressed(BUTTON_SCAN[i])
    }
}

fn poll_keyboard_move(state: &mut ControlState, input: &input::Input, tics: u64) {
    //TODO impl button mapping, uses hardcoded buttons as for now
    let move_factor = if state.button_state[Button::Run as usize] {
        RUN_MOVE * tics
    } else {
        BASE_MOVE * tics
    } as i32;

    if input.key_pressed(NumCode::UpArrow) {
        state.control.y -= move_factor;
    }
    if input.key_pressed(NumCode::DownArrow) {
        state.control.y += move_factor;
    }
    if input.key_pressed(NumCode::LeftArrow) {
        state.control.x -= move_factor;
    }
    if input.key_pressed(NumCode::RightArrow) {
        state.control.x += move_factor;
    }
}

/*
=============================================================================

                    PALETTE SHIFTING STUFF

=============================================================================
*/

pub fn start_bonus_flash(game_state: &mut GameState) {
    game_state.bonus_count = NUM_WHITE_SHIFTS as i32 * WHITE_TICS;
}

pub fn start_damage_flash(game_state: &mut GameState, damage: i32) {
    game_state.damage_count += damage;
}

fn init_colour_shifts() -> ColourShifts {
    let mut red_shifts = [[0; 768]; NUM_RED_SHIFTS];
    let mut white_shifts = [[0; 768]; NUM_WHITE_SHIFTS];

    for i in 0..NUM_RED_SHIFTS {
        let mut ix = 0;
        for _ in 0..256 {
            let delta = 64 - GAMEPAL[ix] as i32;
            red_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / RED_STEPS) as u8;
            ix += 1;

            let delta = -(GAMEPAL[ix] as i32);
            red_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / RED_STEPS) as u8;
            ix += 1;

            let delta = -(GAMEPAL[ix] as i32);
            red_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / RED_STEPS) as u8;
            ix += 1;
        }
    }

    for i in 0..NUM_WHITE_SHIFTS {
        let mut ix = 0;
        for _ in 0..256 {
            let delta = 64 - GAMEPAL[ix] as i32;
            white_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / WHITE_STEPS) as u8;
            ix += 1;

            let delta = 62 - GAMEPAL[ix] as i32;
            white_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / WHITE_STEPS) as u8;
            ix += 1;

            let delta = -(GAMEPAL[ix] as i32);
            white_shifts[i][ix] = (GAMEPAL[ix] as i32 + delta * i as i32 / WHITE_STEPS) as u8;
            ix += 1;
        }
    }

    ColourShifts {
        red_shifts,
        white_shifts,
    }
}

fn clear_palette_shifts(game_state: &mut GameState) {
    game_state.bonus_count = 0;
    game_state.damage_count = 0;
}

async fn update_palette_shifts(
    game_state: &mut GameState,
    vga: &VGA,
    shifts: &ColourShifts,
    tics: u64,
) {
    let mut white;
    if game_state.bonus_count != 0 {
        white = game_state.bonus_count / WHITE_TICS + 1;
        if white > NUM_WHITE_SHIFTS as i32 {
            white = NUM_WHITE_SHIFTS as i32;
        }
        game_state.bonus_count -= tics as i32;
        if game_state.bonus_count < 0 {
            game_state.bonus_count = 0;
        }
    } else {
        white = 0;
    }

    let mut red;
    if game_state.damage_count != 0 {
        red = game_state.damage_count / 10 + 1;
        if red > NUM_RED_SHIFTS as i32 {
            red = NUM_RED_SHIFTS as i32;
        }

        game_state.damage_count -= tics as i32;
        if game_state.damage_count < 0 {
            game_state.damage_count = 0;
        }
    } else {
        red = 0;
    }

    if red != 0 {
        util::vsync(vga).await;
        set_palette(vga, &shifts.red_shifts[red as usize - 1]);
        game_state.pal_shifted = true;
    } else if white != 0 {
        util::vsync(vga).await;
        set_palette(vga, &shifts.white_shifts[white as usize - 1]);
        game_state.pal_shifted = true;
    } else if game_state.pal_shifted {
        util::vsync(vga).await;
        set_palette(vga, &GAMEPAL); // back to normal
        game_state.pal_shifted = false;
    }
}

/// Resets palette to normal if needed
pub async fn finish_palette_shifts(game_state: &mut GameState, vga: &VGA) {
    if game_state.pal_shifted {
        game_state.pal_shifted = false;
        util::vsync(vga).await;
        set_palette(vga, &GAMEPAL);
    }
}
