#[cfg(test)]
#[path = "./play_test.rs"]
mod play_test;

use std::time::Instant;

use tokio::time::sleep;

#[cfg(feature = "tracing")]
use tracing::info_span;
#[cfg(feature = "tracing")]
use tracing::instrument;

use vga::VGA;
use vga::input::NumCode;

use crate::act1::{move_doors, move_push_walls};
use crate::agent::{
    draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_score, draw_weapon,
};
use crate::assets::Music;
use crate::assets::{GAMEPAL, GraphicNum};
use crate::config::WolfConfig;
use crate::debug::debug_keys;
use crate::def::{
    ANGLE_QUAD, ANGLES, ActiveType, Assets, At, Button, Control, ControlState, FINE_ANGLES,
    FL_NEVERMARK, FL_NONMARK, FOCAL_LENGTH, GLOBAL1, GameState, IWConfig, LevelState, NUM_BUTTONS,
    ObjKey, PlayState, SCREENLOC, STATUS_LINES, TILEGLOBAL, WindowState,
};
use crate::draw::RayCastConsts;
use crate::draw::init_ray_cast_consts;
use crate::draw::{RayCast, three_d_refresh};
use crate::fixed::{Fixed, new_fixed, new_fixed_u32};
use crate::input::DIR_SCAN_EAST;
use crate::input::DIR_SCAN_NORTH;
use crate::input::DIR_SCAN_SOUTH;
use crate::input::DIR_SCAN_WEST;
use crate::input::Input;
use crate::input::InputMode;
use crate::input::{self};
use crate::inter::clear_split_vwb;
use crate::loader::Loader;
use crate::menu::GameStateUpdate;
use crate::menu::LSA_X;
use crate::menu::LSA_Y;
use crate::menu::Menu;
use crate::menu::MenuState;
use crate::menu::control_panel;
use crate::menu::message;
use crate::scale::{CompiledScaler, setup_scaling};
use crate::sd::Sound;
use crate::start::load_the_game;
use crate::time;
use crate::time::TARGET_FRAME_DURATION;
use crate::time::Ticker;
use crate::us1::draw_window;
use crate::util::check_param;
use crate::vga_render::VGARenderer;
use crate::vl::set_palette;

//TODO separate draw.c stuff from play.c stuff in here

const MIN_DIST: i32 = 0x5800;

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

const DEMO_TICS: u64 = 4;

pub static BUTTON_JOY: [Button; 4] = [Button::Attack, Button::Strafe, Button::Use, Button::Run];

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

pub fn calc_projection(width: usize, height: usize) -> ProjectionConfig {
    let view_width = width & !15;
    let view_height = height & !1;
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

    let scaler = setup_scaling((view_width as f64 * 1.5) as usize, view_height);

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
    iw_config: &IWConfig,
    ticker: &time::Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    control_state: &mut ControlState,
    vga: &VGA,
    sound: &mut Sound,
    rc_param: RayCast,
    rdr: &VGARenderer,
    input: &mut Input,
    prj_param: ProjectionConfig,
    assets: &Assets,
    loader: &dyn Loader,
) -> (ProjectionConfig, RayCast) {
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

    let mut prj = prj_param;
    let mut rc = rc_param;
    let mut _frame_id: u64 = 0;
    while game_state.play_state == PlayState::StillPlaying {
        #[cfg(feature = "tracing")]
        let span = info_span!("frame", id = _frame_id);
        _frame_id += 1;

        let next_tic = if input.mode == InputMode::Player {
            1
        } else {
            DEMO_TICS
        };

        let (next_frame_start, curr_tics) = ticker.next_tics_time(next_tic);
        let want_frame_start = next_frame_start + (TARGET_FRAME_DURATION / 2); // target mid frame time
        let wait_time = want_frame_start.saturating_duration_since(Instant::now());
        sleep(wait_time).await;

        let mut tics = ticker.get_count().saturating_sub(curr_tics); // in the best case next_tics many tics, saturating in case the count is reset/non-monotonic
        if tics == 0 {
            tics = 1;
        }
        if input.mode == InputMode::DemoPlayback {
            tics = DEMO_TICS;
        }

        let player = level_state.player();
        let rc_consts = init_ray_cast_consts(&prj, player, game_state.push_wall_pos);

        #[cfg(feature = "tracing")]
        span.in_scope(|| {
            update_game_state(
                tics,
                ticker,
                level_state,
                game_state,
                control_state,
                sound,
                rdr,
                input,
                &prj,
                &rc_consts,
                assets,
            )
        });
        #[cfg(not(feature = "tracing"))]
        update_game_state(
            tics,
            ticker,
            level_state,
            game_state,
            control_state,
            sound,
            rdr,
            input,
            &prj,
            &rc_consts,
            assets,
        );

        update_palette_shifts(game_state, vga, &shifts, tics).await;

        three_d_refresh(
            ticker,
            game_state,
            level_state,
            &mut rc,
            rdr,
            sound,
            &prj,
            &rc_consts,
            assets,
            input.mode == InputMode::DemoPlayback,
        )
        .await;

        let update = check_keys(
            wolf_config,
            iw_config,
            ticker,
            sound,
            rc,
            rdr,
            assets,
            win_state,
            menu_state,
            level_state,
            game_state,
            input,
            prj,
            loader,
        )
        .await;
        prj = update.projection_config;
        rc = update.ray_cast;

        if let Some(which) = update.load {
            load_the_game(
                iw_config,
                level_state,
                game_state,
                win_state,
                rdr,
                input,
                assets,
                loader,
                which,
                LSA_X + 8,
                LSA_Y + 5,
            )
            .await;
            update_status_bar(game_state, rdr);
        }

        game_state.time_count += tics;

        // TODO SD_Poll() ?
        // TODO UpdateSoundLoc

        let offset_prev = rdr.buffer_offset();
        for i in 0..3 {
            rdr.set_buffer_offset(SCREENLOC[i]);
        }
        rdr.set_buffer_offset(offset_prev);
    }
    (prj, rc)
}

fn update_game_state(
    tics: u64,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    control_state: &mut ControlState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    input: &mut Input,
    prj: &ProjectionConfig,
    rc_consts: &RayCastConsts,
    assets: &Assets,
) {
    poll_controls(control_state, tics, input);
    if input.mode == InputMode::DemoPlayback {
        if input.demo_ptr == input.demo_buffer.as_ref().expect("demo_data").len() {
            game_state.play_state = PlayState::Completed;
        }
    }

    // actor thinking

    game_state.made_noise = false;

    move_doors(level_state, game_state, sound, assets, rc_consts, tics);
    move_push_walls(level_state, game_state, tics);

    for i in 0..level_state.actors.len() {
        let k = ObjKey(i);
        if level_state.actors.exists(k) {
            do_actor(
                k,
                tics,
                ticker,
                level_state,
                game_state,
                sound,
                rdr,
                input,
                control_state,
                prj,
                rc_consts,
                assets,
            );
        }
    }
}

fn do_actor(
    k: ObjKey,
    tics: u64,
    ticker: &Ticker,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    input: &Input,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    rc_consts: &RayCastConsts,
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
        if let Some(think) = level_state
            .obj(k)
            .state
            .expect(&format!(
                "state,k={:?}, class={:?}, is={}",
                k,
                level_state.obj(k).class,
                level_state.obj(k).state.is_some(),
            ))
            .think
        {
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
                rc_consts,
            );
            if level_state.obj(k).state.is_none() {
                level_state.actors.drop_obj(k);
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
    level_state.update_obj(k, |obj| obj.tic_count -= tics as i32);
    while level_state.obj(k).tic_count <= 0 {
        if let Some(action) = level_state.obj(k).state.expect("state").action {
            action(
                k,
                tics,
                ticker,
                level_state,
                game_state,
                sound,
                rdr,
                input,
                control_state,
                prj,
                assets,
                rc_consts,
            );
            if level_state.obj(k).state.is_none() {
                level_state.actors.drop_obj(k);
                return;
            }
        }

        level_state.update_obj(k, |obj| obj.state = obj.state.expect("state").next);
        if level_state.obj(k).state.is_none() {
            level_state.actors.drop_obj(k);
            return;
        }

        if level_state.obj(k).state.expect("state").tic_time == 0 {
            level_state.update_obj(k, |obj| obj.tic_count = 0);
            break; // think a last time below
        }
        level_state.update_obj(k, |obj| {
            obj.tic_count += obj.state.expect("state").tic_time as i32
        });
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
            rc_consts,
        );
        if level_state.obj(k).state.is_none() {
            level_state.actors.drop_obj(k);
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
        draw_play_border(rdr, prj.view_width, prj.view_height);
        rdr.pic(0, 200 - STATUS_LINES, GraphicNum::STATUSBARPIC);
    }
    rdr.set_buffer_offset(offset_prev);

    update_status_bar(state, rdr);
}

pub fn update_status_bar(state: &GameState, rdr: &VGARenderer) {
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
        draw_play_border(rdr, prj.view_width, prj.view_height);
    }
}

pub fn draw_play_border(rdr: &VGARenderer, width: usize, height: usize) {
    //clear the background:
    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);

    let xl = 160 - width / 2;
    let yl = (200 - STATUS_LINES - height) / 2;

    //view area
    rdr.bar(xl, yl, width, height, 0);

    //border around the view area
    vw_hlin(rdr, xl - 1, xl + width, yl - 1, 0);
    vw_hlin(rdr, xl - 1, xl + width, yl + height, 125);
    vw_vlin(rdr, yl - 1, yl + height, xl - 1, 0);
    vw_vlin(rdr, yl - 1, yl + height, xl + width, 125);

    rdr.plot(xl - 1, yl + height, 124);
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

#[cfg_attr(feature = "tracing", instrument(skip_all))]
async fn check_keys(
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    ticker: &time::Ticker,
    sound: &mut Sound,
    rc: RayCast,
    rdr: &VGARenderer,
    assets: &Assets,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    input: &mut Input,
    prj: ProjectionConfig,
    loader: &dyn Loader,
) -> GameStateUpdate {
    if input.mode == InputMode::DemoPlayback {
        return GameStateUpdate::with_render_update(prj, rc);
    }

    if input.key_pressed(NumCode::BackSpace)
        && input.key_pressed(NumCode::LShift)
        && input.key_pressed(NumCode::Alt)
        && check_param("goobers")
    {
        clear_split_vwb(win_state);

        message(rdr, win_state, "Debugging keys are\nnow available!");
        input.clear_keys_down();
        input.ack();
        win_state.debug_ok = true;
        draw_all_play_border_sides(rdr, &prj);
        return GameStateUpdate::with_render_update(prj, rc);
    }

    let scan = input.last_scan();
    if scan == NumCode::F1
        || scan == NumCode::F2
        || scan == NumCode::F3
        || scan == NumCode::F4
        || scan == NumCode::F5
        || scan == NumCode::F6
        || scan == NumCode::F7
        || scan == NumCode::F8
        || scan == NumCode::F9
        || scan == NumCode::Escape
    {
        rdr.fade_out().await;
        menu_state.select_menu(Menu::Top);
        let prev_buffer = rdr.buffer_offset();
        rdr.set_buffer_offset(rdr.active_buffer());
        let update = control_panel(
            wolf_config,
            iw_config,
            ticker,
            level_state,
            game_state,
            sound,
            rc,
            rdr,
            input,
            prj,
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
        draw_play_screen(game_state, rdr, &update.projection_config).await;
        if !game_state.start_game && !game_state.loaded_game {
            rdr.fade_in().await;
            start_music(game_state, sound, assets, loader);
        }

        if game_state.loaded_game {
            game_state.play_state = PlayState::Abort;
        }

        //TODO reset lasttimecount and doe some mouse present check

        return update;
    };

    if input.key_pressed(NumCode::Tab) && (win_state.debug_ok || iw_config.options.enable_debug) {
        let prev_buffer = rdr.buffer_offset();
        rdr.set_buffer_offset(rdr.active_buffer());

        win_state.font_number = 0;
        win_state.set_font_color(0, 15);
        debug_keys(
            ticker,
            rdr,
            win_state,
            game_state,
            level_state.player(),
            input,
        )
        .await;

        rdr.set_buffer_offset(prev_buffer);
        return GameStateUpdate::with_render_update(prj, rc);
    }

    return GameStateUpdate::with_render_update(prj, rc);
}

// reads input delta since last tic and manipulates the player state
fn poll_controls(state: &mut ControlState, tics: u64, input: &mut Input) {
    state.control.x = 0;
    state.control.y = 0;
    state.button_held.copy_from_slice(&state.button_state);

    if input.mode == InputMode::DemoPlayback {
        let demo_data = input.demo_buffer.as_ref().expect("demo data");
        let mut button_bits = demo_data[input.demo_ptr];
        input.demo_ptr += 1;
        for i in 0..NUM_BUTTONS {
            state.button_state[i] = (button_bits & 1) != 0;
            button_bits >>= 1;
        }

        state.control.x = (demo_data[input.demo_ptr] as i8) as i32;
        input.demo_ptr += 1;

        state.control.y = (demo_data[input.demo_ptr] as i8) as i32;
        input.demo_ptr += 1;

        state.control.x *= tics as i32;
        state.control.y *= tics as i32;
        return;
    }

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
        state.button_state[i] = input.key_pressed(input.button_scan[i])
    }
}

fn poll_keyboard_move(state: &mut ControlState, input: &input::Input, tics: u64) {
    let move_factor = if state.button_state[Button::Run as usize] {
        RUN_MOVE * tics
    } else {
        BASE_MOVE * tics
    } as i32;

    if input.key_pressed(input.dir_scan[DIR_SCAN_NORTH]) {
        state.control.y -= move_factor;
    }
    if input.key_pressed(input.dir_scan[DIR_SCAN_SOUTH]) {
        state.control.y += move_factor;
    }
    if input.key_pressed(input.dir_scan[DIR_SCAN_WEST]) {
        state.control.x -= move_factor;
    }
    if input.key_pressed(input.dir_scan[DIR_SCAN_EAST]) {
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

#[cfg_attr(feature = "tracing", instrument(skip_all))]
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
        set_palette(vga, &shifts.red_shifts[red as usize - 1]);
        game_state.pal_shifted = true;
    } else if white != 0 {
        set_palette(vga, &shifts.white_shifts[white as usize - 1]);
        game_state.pal_shifted = true;
    } else if game_state.pal_shifted {
        set_palette(vga, &GAMEPAL); // back to normal
        game_state.pal_shifted = false;
    }
}

/// Resets palette to normal if needed
pub fn finish_palette_shifts(game_state: &mut GameState, vga: &VGA) {
    if game_state.pal_shifted {
        game_state.pal_shifted = false;
        set_palette(vga, &GAMEPAL);
    }
}

pub fn start_music(
    game_state: &mut GameState,
    sound: &mut Sound,
    assets: &Assets,
    loader: &dyn Loader,
) {
    let track = SONGS[game_state.map_on + game_state.episode * 10];
    sound.play_music(track, assets, loader);
}
