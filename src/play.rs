#[cfg(test)]
#[path = "./play_test.rs"]
mod play_test;

use web_time::{Duration, Instant};

use vga::VGA;
use vga::input::NumCode;
use vga::util::sleep;

use crate::act1::{move_doors, move_push_walls};
use crate::agent::draw_fps;
use crate::agent::{
    draw_ammo, draw_face, draw_health, draw_keys, draw_level, draw_lives, draw_score, draw_weapon,
};
use crate::assets::Music;
use crate::assets::{GAMEPAL, GraphicNum};
use crate::config::WolfConfig;
use crate::debug::debug_keys;
use crate::def::BenchmarkResult;
use crate::def::{
    ANGLE_QUAD, ANGLES, ActiveType, At, Button, Control, ControlState, FINE_ANGLES, FL_NEVERMARK,
    FL_NONMARK, FOCAL_LENGTH, GLOBAL1, GameState, IWConfig, LevelState, NUM_BUTTONS, ObjKey,
    PlayState, SCREENLOC, STATUS_LINES, TILEGLOBAL, WindowState,
};
use crate::draw::three_d_refresh;
use crate::fixed::Fixed;
use crate::inter::clear_split_vwb;
use crate::loader::Loader;
use crate::menu::{GameStateUpdate, LSA_X, LSA_Y, Menu, MenuState, control_panel, message};
use crate::rc::{
    DIR_SCAN_EAST, DIR_SCAN_NORTH, DIR_SCAN_SOUTH, DIR_SCAN_WEST, InputMode, RenderContext,
};
use crate::scale::{CompiledScaler, setup_scaling};
use crate::start::load_the_game;
use crate::time::TARGET_FRAME_DURATION;
use crate::us1::draw_window;
use crate::util::check_param;
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

pub const DEMO_TICS: u64 = 4;

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

#[derive(Clone)]
struct Fps {
    real: f32,
    unbounded: f32,
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
    let mut sines: Vec<Fixed> = vec![Fixed::new(0, 0); ANGLES + ANGLE_QUAD + 1];

    let mut angle: f32 = 0.0;
    let angle_step = PI / 2.0 / ANGLE_QUAD as f32;
    for i in 0..=ANGLE_QUAD {
        let value: u32 = (GLOBAL1 as f32 * angle.sin()) as u32;
        //TODO ugly fixes in here, make this exact to the old c-code
        let v_fixed = Fixed::new_from_u32(value.min(65535));
        let mut value_neg = value | 0x80000000u32;
        if i == 90 {
            //otherwise a ??rounding error?? occurs and walking
            //backward does not work anymore (TODO Fix this proper,
            //latest in the generalisation)
            value_neg -= 1;
        }
        let v_fixed_neg = Fixed::new_from_u32(value_neg);
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
    rc: &mut RenderContext,
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    control_state: &mut ControlState,
    loader: &dyn Loader,
    benchmark: bool,
) -> Option<BenchmarkResult> {
    let shifts = init_colour_shifts();

    game_state.play_state = PlayState::StillPlaying;
    // TODO frameon = 0??
    // TODO running = false?
    // TODO anglefrac = 0?
    game_state.face_count = 0;
    // TODO funnyticcount = 0?
    // TODO lasttimeout = 0??
    rc.ticker.clear_count();
    rc.clear_keys_down();
    clear_palette_shifts(game_state);

    let mut fps_buffer_ptr = 0;
    let mut fps_buffer: Vec<Option<Fps>> = if iw_config.options.show_frame_rate {
        vec![None; 70]
    } else {
        vec![None; 0]
    };

    let mut benchmark_result = if benchmark {
        Some(BenchmarkResult {
            total: Duration::ZERO,
            real: Duration::ZERO,
            unbounded: Duration::ZERO,
        })
    } else {
        None
    };

    let play_loop_start = Instant::now();

    let mut _frame_id: u64 = 0;
    let mut demo_tic = 0;
    while game_state.play_state == PlayState::StillPlaying {
        let r_start_frame = if iw_config.options.show_frame_rate || benchmark {
            draw_play_border(rc, rc.projection.view_width, rc.projection.view_height); //clear border, as the fps count is written on the border
            Some(Instant::now())
        } else {
            None
        };

        let (next_frame_start, curr_tics) = rc.ticker.next_tics_time(1);
        let want_frame_start = next_frame_start + (TARGET_FRAME_DURATION / 2); // target mid frame time
        let wait_time = want_frame_start.saturating_duration_since(Instant::now());
        sleep(wait_time.as_millis_f64() as u32).await;
        rc.display();

        if rc.input.mode == InputMode::DemoPlayback {
            demo_tic += 1;
            if demo_tic < DEMO_TICS {
                /* Don't sleep to long (not DEMO_TICS long) to give the other task some room
                 * to work on stuff.
                 */
                continue;
            } else {
                demo_tic = 0;
            }
        }

        let u_start_frame = if iw_config.options.show_frame_rate || benchmark {
            Some(Instant::now())
        } else {
            None
        };

        let mut tics = rc.ticker.get_count().saturating_sub(curr_tics); // in the best case next_tics many tics, saturating in case the count is reset/non-monotonic
        if tics == 0 {
            tics = 1;
        }
        if rc.input.mode == InputMode::DemoPlayback {
            tics = DEMO_TICS;
        }

        update_game_state(rc, tics, level_state, game_state, control_state).await;

        update_palette_shifts(game_state, &mut rc.vga, &shifts, tics).await;

        three_d_refresh(
            rc,
            game_state,
            level_state,
            rc.input.mode == InputMode::DemoPlayback,
        )
        .await;

        let update = check_keys(
            rc,
            wolf_config,
            iw_config,
            win_state,
            menu_state,
            level_state,
            game_state,
            loader,
        )
        .await;

        if let Some(which) = update.load {
            load_the_game(
                rc,
                iw_config,
                level_state,
                game_state,
                win_state,
                loader,
                which,
                LSA_X + 8,
                LSA_Y + 5,
            )
            .await;
            update_status_bar(rc, game_state);
        }

        if rc.input.mode == InputMode::DemoPlayback {
            if rc.check_ack() {
                rc.clear_keys_down();
                game_state.play_state = PlayState::Abort;
            }
        }

        if iw_config.options.show_frame_rate {
            fps_buffer_ptr = update_fps(
                rc,
                r_start_frame.expect("r_start_frame"),
                u_start_frame.expect("u_start_frame"),
                &mut fps_buffer,
                fps_buffer_ptr,
            );
        }
        if benchmark {
            let b = benchmark_result.as_mut().unwrap();
            b.real += r_start_frame.expect("r_start_frame").elapsed();
            b.unbounded += u_start_frame.expect("u_start_frame").elapsed();
        }

        game_state.time_count += tics;

        // TODO SD_Poll() ?
        // TODO UpdateSoundLoc

        let offset_prev = rc.buffer_offset();
        for i in 0..3 {
            rc.set_buffer_offset(SCREENLOC[i]);
        }
        rc.set_buffer_offset(offset_prev);
    }

    if benchmark {
        benchmark_result.as_mut().unwrap().total = play_loop_start.elapsed();
    }

    benchmark_result
}

async fn update_game_state(
    rc: &mut RenderContext,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    control_state: &mut ControlState,
) {
    poll_controls(rc, control_state, tics);
    if rc.input.mode == InputMode::DemoPlayback {
        if rc.input.demo_ptr == rc.input.demo_buffer.as_ref().expect("demo_data").len() {
            game_state.play_state = PlayState::Completed;
        }
    }

    // actor thinking

    game_state.made_noise = false;

    move_doors(rc, level_state, game_state, tics);
    move_push_walls(level_state, game_state, tics);

    for i in 0..level_state.actors.len() {
        let k = ObjKey(i);
        if level_state.actors.exists(k) {
            do_actor(rc, k, tics, level_state, game_state, control_state).await;
        }
    }
}

async fn do_actor(
    rc: &mut RenderContext,
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    control_state: &mut ControlState,
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
            think(rc, k, tics, level_state, game_state, control_state);
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
            action(rc, k, tics, level_state, game_state, control_state);
            if level_state.obj(k).state.is_none() {
                level_state.actors.drop_obj(k);
                return;
            }
        }

        if let Some(async_action) = level_state.obj(k).state.expect("state").async_action {
            async_action(rc, k, tics, level_state, game_state, control_state).await;
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
        think(rc, k, tics, level_state, game_state, control_state);
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

fn update_fps(
    rc: &mut RenderContext,
    r_start_frame: Instant,
    u_start_frame: Instant,
    fps_buffer: &mut Vec<Option<Fps>>,
    fps_buffer_ptr: usize,
) -> usize {
    let r_elapsed = r_start_frame.elapsed().as_secs_f32();
    let u_elapsed = u_start_frame.elapsed().as_secs_f32();

    fps_buffer[fps_buffer_ptr] = Some(Fps {
        real: r_elapsed,
        unbounded: u_elapsed,
    });
    let mut next_ptr = fps_buffer_ptr + 1;
    if next_ptr >= fps_buffer.len() {
        next_ptr = 0;
    }

    let (r_avg, u_avg) = avg_fps(fps_buffer);
    let fps_str = format!(
        "{:.0}/{:.0}",
        (1.0 / r_avg) * DEMO_TICS as f32,
        (1.0 / u_avg)
    );
    draw_fps(rc, &fps_str);
    next_ptr
}

fn avg_fps(fps_buffer: &Vec<Option<Fps>>) -> (f32, f32) {
    let mut sum_r = 0.0;
    let mut sum_u = 0.0;
    let mut l = 0.0;
    for opt_fps in fps_buffer {
        if let Some(fps) = opt_fps {
            sum_r += fps.real;
            sum_u += fps.unbounded;
            l += 1.0;
        }
    }
    (sum_r / l, sum_u / l)
}

pub async fn draw_play_screen(rc: &mut RenderContext, state: &GameState) {
    rc.fade_out().await;

    let offset_prev = rc.buffer_offset();
    for i in 0..3 {
        rc.set_buffer_offset(SCREENLOC[i]);
        draw_play_border(rc, rc.projection.view_width, rc.projection.view_height);
        rc.pic(0, 200 - STATUS_LINES, GraphicNum::STATUSBARPIC);
    }
    rc.set_buffer_offset(offset_prev);

    update_status_bar(rc, state);
}

pub fn update_status_bar(rc: &mut RenderContext, state: &GameState) {
    draw_face(rc, state);
    draw_health(rc, state);
    draw_lives(rc, state);
    draw_level(rc, state);
    draw_ammo(rc, state);
    draw_keys(rc, state);
    draw_weapon(rc, state);
    draw_score(rc, state);
}

fn draw_all_play_border_sides(rc: &mut RenderContext) {
    for i in 0..3 {
        rc.set_buffer_offset(SCREENLOC[i]);
        draw_play_border_side(rc);
    }
}

/// To fix window overwrites
fn draw_play_border_side(rc: &mut RenderContext) {
    let xl = 160 - rc.projection.view_width / 2;
    let yl = (200 - STATUS_LINES - rc.projection.view_height) / 2;

    rc.bar(0, 0, xl - 1, 200 - STATUS_LINES, 127);
    rc.bar(
        xl + rc.projection.view_width + 1,
        0,
        xl - 2,
        200 - STATUS_LINES,
        127,
    );

    vw_vlin(rc, yl - 1, yl + rc.projection.view_height, xl - 1, 0);
    vw_vlin(
        rc,
        yl - 1,
        yl + rc.projection.view_height,
        xl + rc.projection.view_width,
        125,
    );
}

pub fn draw_all_play_border(rc: &mut RenderContext) {
    for i in 0..3 {
        rc.set_buffer_offset(SCREENLOC[i]);
        draw_play_border(rc, rc.projection.view_width, rc.projection.view_height);
    }
}

pub fn draw_play_border(rc: &mut RenderContext, width: usize, height: usize) {
    //clear the background:
    rc.bar(0, 0, 320, 200 - STATUS_LINES, 127);

    let xl = 160 - width / 2;
    let yl = (200 - STATUS_LINES - height) / 2;

    //view area
    rc.bar(xl, yl, width, height, 0);

    //border around the view area
    vw_hlin(rc, xl - 1, xl + width, yl - 1, 0);
    vw_hlin(rc, xl - 1, xl + width, yl + height, 125);
    vw_vlin(rc, yl - 1, yl + height, xl - 1, 0);
    vw_vlin(rc, yl - 1, yl + height, xl + width, 125);

    rc.plot(xl - 1, yl + height, 124);
}

fn vw_hlin(rc: &mut RenderContext, x: usize, z: usize, y: usize, c: u8) {
    rc.hlin(x, y, (z - x) + 1, c)
}

fn vw_vlin(rc: &mut RenderContext, y: usize, z: usize, x: usize, c: u8) {
    rc.vlin(x, y, (z - y) + 1, c)
}

///	Generates a window of a given width & height in the
/// middle of the screen
pub fn center_window(
    rc: &mut RenderContext,
    win_state: &mut WindowState,
    width: usize,
    height: usize,
) {
    draw_window(
        rc,
        win_state,
        ((320 / 8) - width) / 2,
        ((160 / 8) - height) / 2,
        width,
        height,
    );
}

async fn check_keys(
    rc: &mut RenderContext,
    wolf_config: &mut WolfConfig,
    iw_config: &IWConfig,
    win_state: &mut WindowState,
    menu_state: &mut MenuState,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    loader: &dyn Loader,
) -> GameStateUpdate {
    if rc.input.mode == InputMode::DemoPlayback {
        return GameStateUpdate::without_update();
    }

    if rc.key_pressed(NumCode::BackSpace)
        && rc.key_pressed(NumCode::LShift)
        && rc.key_pressed(NumCode::Alt)
        && check_param("goobers")
    {
        clear_split_vwb(win_state);

        message(rc, win_state, "Debugging keys are\nnow available!");
        rc.clear_keys_down();
        rc.ack().await;
        win_state.debug_ok = true;
        draw_all_play_border_sides(rc);
        return GameStateUpdate::without_update();
    }

    let scan = rc.last_scan();
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
        rc.fade_out().await;
        menu_state.select_menu(Menu::Top);
        let prev_buffer = rc.buffer_offset();
        rc.set_buffer_offset(rc.active_buffer());
        let update = control_panel(
            rc,
            wolf_config,
            iw_config,
            level_state,
            game_state,
            win_state,
            menu_state,
            loader,
            scan,
        )
        .await;
        rc.set_buffer_offset(prev_buffer);

        win_state.set_font_color(0, 15);
        rc.clear_keys_down();
        draw_play_screen(rc, game_state).await;
        if !game_state.start_game && !game_state.loaded_game {
            rc.fade_in().await;
            start_music(rc, game_state, loader);
        }

        if game_state.loaded_game {
            game_state.play_state = PlayState::Abort;
        }

        //TODO reset lasttimecount and doe some mouse present check

        return update;
    };

    if rc.key_pressed(NumCode::Tab) && (win_state.debug_ok || iw_config.options.enable_debug) {
        let prev_buffer = rc.buffer_offset();
        rc.set_buffer_offset(rc.active_buffer());

        win_state.font_number = 0;
        win_state.set_font_color(0, 15);
        debug_keys(rc, win_state, game_state, level_state.player()).await;

        rc.set_buffer_offset(prev_buffer);
        return GameStateUpdate::without_update();
    }

    return GameStateUpdate::without_update();
}

// reads input delta since last tic and manipulates the player state
fn poll_controls(rc: &mut RenderContext, state: &mut ControlState, tics: u64) {
    state.control.x = 0;
    state.control.y = 0;
    state.button_held.copy_from_slice(&state.button_state);

    if rc.input.mode == InputMode::DemoPlayback {
        let demo_data = rc.input.demo_buffer.as_ref().expect("demo data");
        let mut button_bits = demo_data[rc.input.demo_ptr];
        rc.input.demo_ptr += 1;
        for i in 0..NUM_BUTTONS {
            state.button_state[i] = (button_bits & 1) != 0;
            button_bits >>= 1;
        }

        state.control.x = (demo_data[rc.input.demo_ptr] as i8) as i32;
        rc.input.demo_ptr += 1;

        state.control.y = (demo_data[rc.input.demo_ptr] as i8) as i32;
        rc.input.demo_ptr += 1;

        state.control.x *= tics as i32;
        state.control.y *= tics as i32;
        return;
    }

    poll_keyboard_buttons(rc, state);

    poll_keyboard_move(rc, state, tics);
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

fn poll_keyboard_buttons(rc: &mut RenderContext, state: &mut ControlState) {
    for i in 0..NUM_BUTTONS {
        state.button_state[i] = rc.key_pressed(rc.input.button_scan[i])
    }
}

fn poll_keyboard_move(rc: &mut RenderContext, state: &mut ControlState, tics: u64) {
    let move_factor = if state.button_state[Button::Run as usize] {
        RUN_MOVE * tics
    } else {
        BASE_MOVE * tics
    } as i32;

    if rc.key_pressed(rc.input.dir_scan[DIR_SCAN_NORTH]) {
        state.control.y -= move_factor;
    }
    if rc.key_pressed(rc.input.dir_scan[DIR_SCAN_SOUTH]) {
        state.control.y += move_factor;
    }
    if rc.key_pressed(rc.input.dir_scan[DIR_SCAN_WEST]) {
        state.control.x -= move_factor;
    }
    if rc.key_pressed(rc.input.dir_scan[DIR_SCAN_EAST]) {
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
    vga: &mut VGA,
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
pub fn finish_palette_shifts(game_state: &mut GameState, vga: &mut VGA) {
    if game_state.pal_shifted {
        game_state.pal_shifted = false;
        set_palette(vga, &GAMEPAL);
    }
}

pub fn start_music(rc: &mut RenderContext, game_state: &mut GameState, loader: &dyn Loader) {
    let track = SONGS[game_state.map_on + game_state.episode * 10];
    rc.play_music(track, loader);
}
