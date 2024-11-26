use std::ascii::Char;

use crate::agent::{draw_level, draw_score, give_points};
use crate::assets::{num_pic, GraphicNum, Music};
use crate::def::{Assets, Difficulty, GameState, IWConfig, WindowState, STATUS_LINES};
use crate::input::Input;
use crate::loader::Loader;
use crate::menu::{clear_ms_screen, draw_stripes, READ_HCOLOR};
use crate::play::{draw_all_play_border, ProjectionConfig};
use crate::sd::Sound;
use crate::text::end_text;
use crate::time;
use crate::us1::print;
use crate::vga_render::VGARenderer;
use crate::vh::BLACK;

static ALPHA: [GraphicNum; 43] = [
    GraphicNum::NUM0PIC,
    GraphicNum::NUM1PIC,
    GraphicNum::NUM2PIC,
    GraphicNum::NUM3PIC,
    GraphicNum::NUM4PIC,
    GraphicNum::NUM5PIC,
    GraphicNum::NUM6PIC,
    GraphicNum::NUM7PIC,
    GraphicNum::NUM8PIC,
    GraphicNum::NUM9PIC,
    GraphicNum::COLONPIC,
    GraphicNum::NONE,
    GraphicNum::NONE,
    GraphicNum::NONE,
    GraphicNum::NONE,
    GraphicNum::NONE,
    GraphicNum::NONE,
    GraphicNum::APIC,
    GraphicNum::BPIC,
    GraphicNum::CPIC,
    GraphicNum::DPIC,
    GraphicNum::EPIC,
    GraphicNum::FPIC,
    GraphicNum::GPIC,
    GraphicNum::HPIC,
    GraphicNum::IPIC,
    GraphicNum::JPIC,
    GraphicNum::KPIC,
    GraphicNum::LPIC,
    GraphicNum::MPIC,
    GraphicNum::NPIC,
    GraphicNum::OPIC,
    GraphicNum::PPIC,
    GraphicNum::QPIC,
    GraphicNum::RPIC,
    GraphicNum::SPIC,
    GraphicNum::TPIC,
    GraphicNum::UPIC,
    GraphicNum::VPIC,
    GraphicNum::WPIC,
    GraphicNum::XPIC,
    GraphicNum::YPIC,
    GraphicNum::ZPIC,
];

const ASCII_ALPHA_RANGE: u8 = Char::SmallA as u8 - Char::CapitalA as u8; // 'a' - 'A'

struct ParTime {
    time: f32,
    time_str: &'static str,
}

const PAR_AMOUNT: i32 = 500;
const RATIO_XX: usize = 37;
const PERCENT_100_AMT: i32 = 10000;

const RATIO_X: usize = 6;
const RATIO_Y: usize = 14;
const TIME_X: usize = 14;
const TIME_Y: usize = 8;

static PAR_TIMES: [ParTime; 60] = [
    // Episode One Par Times
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 3.5,
        time_str: "03:30",
    },
    ParTime {
        time: 3.0,
        time_str: "03:00",
    },
    ParTime {
        time: 3.0,
        time_str: "03:00",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    }, // Boss level
    ParTime {
        time: 0.0,
        time_str: "??:??",
    }, // Secret level
    // Episode Two Par Times
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 3.5,
        time_str: "03:30",
    },
    ParTime {
        time: 3.0,
        time_str: "03:00",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 4.0,
        time_str: "04:00",
    },
    ParTime {
        time: 6.0,
        time_str: "06:00",
    },
    ParTime {
        time: 1.0,
        time_str: "01:00",
    },
    ParTime {
        time: 3.0,
        time_str: "03:00",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    // Episode Three Par Times
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 3.5,
        time_str: "03:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 6.0,
        time_str: "06:00",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    // Episode Four Par Times
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 1.0,
        time_str: "01:00",
    },
    ParTime {
        time: 4.5,
        time_str: "04:30",
    },
    ParTime {
        time: 3.5,
        time_str: "03:30",
    },
    ParTime {
        time: 2.0,
        time_str: "02:00",
    },
    ParTime {
        time: 4.5,
        time_str: "04:30",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    // Episode Five Par Times
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 1.5,
        time_str: "01:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 2.5,
        time_str: "02:30",
    },
    ParTime {
        time: 4.0,
        time_str: "04:00",
    },
    ParTime {
        time: 3.0,
        time_str: "03:00",
    },
    ParTime {
        time: 4.5,
        time_str: "04:30",
    },
    ParTime {
        time: 3.5,
        time_str: "03:30",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    // Episode Six Par Times
    ParTime {
        time: 6.5,
        time_str: "06:30",
    },
    ParTime {
        time: 4.0,
        time_str: "04:00",
    },
    ParTime {
        time: 4.5,
        time_str: "04:30",
    },
    ParTime {
        time: 6.0,
        time_str: "06:00",
    },
    ParTime {
        time: 5.0,
        time_str: "05:00",
    },
    ParTime {
        time: 5.5,
        time_str: "05:30",
    },
    ParTime {
        time: 5.5,
        time_str: "05:30",
    },
    ParTime {
        time: 8.5,
        time_str: "08:30",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
    ParTime {
        time: 0.0,
        time_str: "??:??",
    },
];

pub async fn victory(
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    input: &Input,
    assets: &Assets,
    win_state: &mut WindowState,
    loader: &dyn Loader,
) {
    sound.play_music(Music::URAHERO, assets, loader);
    clear_split_vwb(win_state);

    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);
    write(rdr, 18, 2, "you win!");
    write(rdr, TIME_X, TIME_Y - 2, "total time");
    write(rdr, 12, RATIO_Y - 2, "averages");
    write(rdr, RATIO_X + 8, RATIO_Y, "kill    %");
    write(rdr, RATIO_X + 4, RATIO_Y + 2, "secret    %");
    write(rdr, RATIO_X, RATIO_Y + 4, "treasure    %");

    rdr.pic(8, 4, GraphicNum::BJWINSPIC);

    let mut sec = 0;
    let mut kr = 0;
    let mut sr = 0;
    let mut tr = 0;
    for i in 0..8 {
        sec += game_state.level_ratios[i].time;
        kr += game_state.level_ratios[i].kill;
        sr += game_state.level_ratios[i].secret;
        tr += game_state.level_ratios[i].treasure;
    }
    kr /= 8;
    sr /= 8;
    tr /= 8;

    let min = sec as usize / 60;
    let sec = sec as usize % 60;

    let mut i = TIME_X * 8 + 1;
    rdr.pic(i, TIME_Y * 8, num_pic(min / 10));
    i += 2 * 8;
    rdr.pic(i, TIME_Y * 8, num_pic(min % 10));
    i += 2 * 8;
    write(rdr, i / 8, TIME_Y, ":");
    i += 1 * 8;
    rdr.pic(i, TIME_Y * 8, num_pic(sec / 10));
    i += 2 * 8;
    rdr.pic(i, TIME_Y * 8, num_pic(sec % 10));

    let str = kr.to_string();
    let x = RATIO_X + 24 - str.len() * 2;
    write(rdr, x, RATIO_Y, &str);

    let str = sr.to_string();
    let x = RATIO_X + 24 - str.len() * 2;
    write(rdr, x, RATIO_Y + 2, &str);

    let str = tr.to_string();
    let x = RATIO_X + 24 - str.len() * 2;
    write(rdr, x, RATIO_Y + 4, &str);

    if game_state.difficulty >= Difficulty::Medium {
        rdr.pic(30 * 8, TIME_Y * 8, GraphicNum::CTIMECODEPIC);
        win_state.font_number = 0;
        win_state.font_color = READ_HCOLOR;
        win_state.print_x = 30 * 8 - 3;
        win_state.print_y = TIME_Y * 8 + 8;
        win_state.print_x += 4;

        let v1 = (((min / 10) ^ (min % 10)) ^ 0xa) as u32 + 65;
        let v2 = (((sec / 10) ^ (sec % 10)) ^ 0xa) as u32 + 65;
        let v3 = (v1 ^ v2) + 65;
        print(
            rdr,
            win_state,
            &format!(
                "{}{}{}",
                char::from_u32(v1).expect("char"),
                char::from_u32(v2).expect("char"),
                char::from_u32(v3).expect("char")
            ),
        );
    }

    win_state.font_number = 1;

    rdr.activate_buffer(rdr.buffer_offset()).await;
    rdr.fade_in().await;
    input.ack().await;

    rdr.fade_out().await;

    end_text(rdr, input, game_state.episode).await;
}

pub fn clear_split_vwb(win_state: &mut WindowState) {
    // TODO clear 'update' global variable?
    win_state.window_x = 0;
    win_state.window_y = 0;
    win_state.window_w = 320;
    win_state.window_h = 160;
}

pub async fn check_highscore(rdr: &VGARenderer, input: &Input, score: i32, map: usize) {
    // TODO load high_score and check whether user achieved high_score

    draw_high_scores(rdr);
    rdr.activate_buffer(rdr.buffer_offset()).await;
    rdr.fade_in().await;

    input.clear_keys_down();
    input.wait_user_input(500).await;
}

pub fn draw_high_scores(rdr: &VGARenderer) {
    clear_ms_screen(rdr);
    draw_stripes(rdr, 10);

    rdr.pic(48, 0, GraphicNum::HIGHSCOREPIC);
    rdr.pic(4 * 8, 68, GraphicNum::CNAMEPIC);
    rdr.pic(20 * 8, 68, GraphicNum::CLEVELPIC);
    rdr.pic(28 * 8, 68, GraphicNum::CSCOREPIC);
}
/// LevelCompleted
///
/// Entered with the screen faded out
/// Still in split screen mode with the status bar
///
/// Exit with the screen faded out
pub async fn level_completed(
    ticker: &time::Ticker,
    rdr: &VGARenderer,
    input: &Input,
    game_state: &mut GameState,
    prj: &ProjectionConfig,
    sound: &mut Sound,
    assets: &Assets,
    win_state: &mut WindowState,
) {
    rdr.set_buffer_offset(rdr.active_buffer());

    clear_split_vwb(win_state);
    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);
    // TODO StartCPMusic(ENDLEVEL_MUS)

    input.clear_keys_down();
    input.start_ack();

    // do the intermission
    rdr.set_buffer_offset(rdr.active_buffer());
    rdr.pic(0, 16, GraphicNum::GUYPIC);

    let mut bj_breather = new_bj_breather();
    if game_state.map_on < 8 {
        // CURR: Imple write function and write "floor\ncompleted"!!!
        write(rdr, 14, 2, "floor\ncompleted");
        write(rdr, 14, 7, "bonus     0");
        write(rdr, 16, 10, "time");
        write(rdr, 16, 12, " par");
        write(rdr, 9, 14, "kill ratio    %");
        write(rdr, 5, 16, "secret ratio    %");
        write(rdr, 1, 18, "treasure ratio    %");

        write(rdr, 26, 2, (game_state.map_on + 1).to_string().as_str());
        let par_time = &PAR_TIMES[game_state.episode * 10 + game_state.map_on];
        write(rdr, 26, 12, par_time.time_str);

        let mut sec = game_state.time_count / 70;
        if sec > 99 * 60 {
            sec = 99 * 60;
        }
        let time_left = if game_state.time_count < (par_time.time * 4200.0) as u64 {
            ((par_time.time * 4200.0 / 70.0) as u64 - sec) as i32
        } else {
            0
        };

        let min = sec / 60;
        sec %= 60;

        let mut i = 26 * 8;
        rdr.pic(i, 10 * 8, num_pic((min / 10) as usize));
        i += 2 * 8;
        rdr.pic(i, 10 * 8, num_pic((min % 10) as usize));
        i += 2 * 8;
        write(rdr, i / 8, 10, ":");
        i += 2 * 8;
        rdr.pic(i, 10 * 8, num_pic((sec / 10) as usize));
        i += 2 * 8;
        rdr.pic(i, 10 * 8, num_pic((sec % 10) as usize));

        rdr.fade_in().await;

        // FIGURE RATIOS OUT BEFOREHAND
        let kill_ratio = if game_state.kill_total > 0 {
            (game_state.kill_count * 100) / game_state.kill_total
        } else {
            0
        };
        let secret_ratio = if game_state.secret_total > 0 {
            (game_state.secret_count * 100) / game_state.secret_total
        } else {
            0
        };
        let treasure_ratio = if game_state.treasure_total > 0 {
            (game_state.treasure_total * 100) / game_state.treasure_total
        } else {
            0
        };

        // PRINT TIME BONUS
        let mut bonus = time_left * PAR_AMOUNT;
        if bonus > 0 {
            for i in 0..=time_left {
                let str = (i * PAR_AMOUNT).to_string();
                let x = 36 - str.len() * 2;
                write(rdr, x, 7, &str);
                if i % (PAR_AMOUNT / 10) == 0 {
                    // TODO PlaySound(ENDBONUS1SND)
                    // TODO Breath while sound is playing, code is a dummy implementation
                    fake_sound_breathe(ticker, rdr, &mut bj_breather);
                }

                if input.check_ack() {
                    return done_normal_level_complete(
                        ticker,
                        rdr,
                        input,
                        game_state,
                        sound,
                        prj,
                        assets,
                        time_left,
                        kill_ratio,
                        secret_ratio,
                        treasure_ratio,
                        &mut bj_breather,
                    )
                    .await;
                }
            }

            // TODO PlaySound(ENDBONUS2SND)
            fake_sound_breathe(ticker, rdr, &mut bj_breather);
        }

        // KILL RATIO
        for i in 0..=kill_ratio {
            let str = i.to_string();
            let x = RATIO_XX - str.len() * 2;
            write(rdr, x, 14, &str);
            if i % 10 == 0 {
                fake_sound_breathe(ticker, rdr, &mut bj_breather);
            }

            if input.check_ack() {
                return done_normal_level_complete(
                    ticker,
                    rdr,
                    input,
                    game_state,
                    sound,
                    prj,
                    assets,
                    time_left,
                    kill_ratio,
                    secret_ratio,
                    treasure_ratio,
                    &mut bj_breather,
                )
                .await;
            }
        }
        if kill_ratio == 100 {
            bonus += PERCENT_100_AMT;
            let str = bonus.to_string();
            let x = (RATIO_XX - 1) - str.len() * 2;
            write(rdr, x, 7, &str);
            // TODO SD_PlaySound(PERCENT100SND)
        } else if kill_ratio == 0 {
            // TODO SD_StopSound()
            // TODO SD_PlaySound(NOBONUSSND)
        } else {
            // TODO SD_PlaySound(ENDBONUS2SND)
        }
        fake_sound_breathe(ticker, rdr, &mut bj_breather);

        // SECRET RATIO
        for i in 0..=secret_ratio {
            let str = i.to_string();
            let x = RATIO_XX - str.len() * 2;
            write(rdr, x, 16, &str);
            if i % 10 == 0 {
                fake_sound_breathe(ticker, rdr, &mut bj_breather);
            }
            if input.check_ack() {
                return done_normal_level_complete(
                    ticker,
                    rdr,
                    input,
                    game_state,
                    sound,
                    prj,
                    assets,
                    time_left,
                    kill_ratio,
                    secret_ratio,
                    treasure_ratio,
                    &mut bj_breather,
                )
                .await;
            }
        }
        if secret_ratio == 100 {
            bonus += PERCENT_100_AMT;
            let str = bonus.to_string();
            let x = (RATIO_XX - 1) - str.len() * 2;
            write(rdr, x, 7, &str);
            // TODO SD_PlaySound(PERCENT100SND)
        } else if secret_ratio == 0 {
            // TODO SD_StopSound()
            // TODO SD_PlaySound(NOBONUSSND)
        } else {
            // TODO SD_PlaySound(ENDBONUS2SND)
        }

        // TREASURE RATIO
        for i in 0..=treasure_ratio {
            let str = i.to_string();
            let x = RATIO_XX - str.len() * 2;
            write(rdr, x, 18, &str);
            if i % 10 == 0 {
                fake_sound_breathe(ticker, rdr, &mut bj_breather);
            }
            if input.check_ack() {
                return done_normal_level_complete(
                    ticker,
                    rdr,
                    input,
                    game_state,
                    sound,
                    prj,
                    assets,
                    time_left,
                    kill_ratio,
                    secret_ratio,
                    treasure_ratio,
                    &mut bj_breather,
                )
                .await;
            }
        }
        if treasure_ratio == 100 {
            bonus += PERCENT_100_AMT;
            let str = bonus.to_string();
            let x = (RATIO_XX - 1) - str.len() * 2;
            write(rdr, x, 7, &str);
            // TODO SD_PlaySound(PERCENT100SND)
        } else if treasure_ratio == 0 {
            // TODO SD_StopSound()
            // TODO SD_PlaySound(NOBONUSSND)
        }
        return done_normal_level_complete(
            ticker,
            rdr,
            input,
            game_state,
            sound,
            prj,
            assets,
            time_left,
            kill_ratio,
            secret_ratio,
            treasure_ratio,
            &mut bj_breather,
        )
        .await;
    }

    // secret floor completed
    write(rdr, 14, 4, "secret floor\n completed!");
    write(rdr, 10, 16, "15000 bonus!");
    rdr.fade_in().await;

    give_points(game_state, rdr, sound, assets, 15000);

    return finish_level_complete(ticker, rdr, input, game_state, prj, &mut bj_breather).await;
}

// placeholder for starting a sound, waiting for it to complete
// and letting BJ breathe.
fn fake_sound_breathe(ticker: &time::Ticker, rdr: &VGARenderer, bj_breather: &mut BjBreather) {
    let start = ticker.get_count();
    while (ticker.get_count() - start) < 50 {
        bj_breather.poll_breathe(ticker, rdr);
    }
}

async fn done_normal_level_complete(
    ticker: &time::Ticker,
    rdr: &VGARenderer,
    input: &Input,
    game_state: &mut GameState,
    sound: &mut Sound,
    prj: &ProjectionConfig,
    assets: &Assets,
    time_left: i32,
    kill_ratio: i32,
    secret_ratio: i32,
    treasure_ratio: i32,
    bj_breather: &mut BjBreather,
) {
    let str = kill_ratio.to_string();
    let x = RATIO_XX - str.len() * 2;
    write(rdr, x, 14, &str);

    let str = secret_ratio.to_string();
    let x = RATIO_XX - str.len() * 2;
    write(rdr, x, 16, &str);

    let str = treasure_ratio.to_string();
    let x = RATIO_XX - str.len() * 2;
    write(rdr, x, 18, &str);

    let mut bonus = time_left * PAR_AMOUNT;
    if kill_ratio == 100 {
        bonus += PERCENT_100_AMT;
    }
    if secret_ratio == 100 {
        bonus += PERCENT_100_AMT;
    }
    if treasure_ratio == 100 {
        bonus += PERCENT_100_AMT;
    }

    let str = bonus.to_string();
    let x = 36 - str.len() * 2;
    write(rdr, x, 7, &str);

    give_points(game_state, rdr, sound, assets, bonus);

    // SAVE RATIO INFORMATION FOR ENDGAME
    game_state.level_ratios[game_state.map_on].kill = kill_ratio;
    game_state.level_ratios[game_state.map_on].secret = secret_ratio;
    game_state.level_ratios[game_state.map_on].treasure = treasure_ratio;
    game_state.level_ratios[game_state.map_on].time = (game_state.time_count / 70) as i32;

    finish_level_complete(ticker, rdr, input, game_state, prj, bj_breather).await;
}

async fn finish_level_complete(
    ticker: &time::Ticker,
    rdr: &VGARenderer,
    input: &Input,
    game_state: &mut GameState,
    prj: &ProjectionConfig,
    bj_breather: &mut BjBreather,
) {
    draw_score(game_state, rdr);

    input.start_ack();
    while !input.check_ack() {
        bj_breather.poll_breathe(ticker, rdr);
    }

    rdr.fade_out().await;

    draw_all_play_border(rdr, prj);
}

fn write(rdr: &VGARenderer, x: usize, y: usize, str: &str) {
    let mut nx = x * 8;
    let ox = nx;
    let mut ny = y * 8;
    for c in str.chars() {
        if let Some(ascii_char) = c.as_ascii() {
            if ascii_char == Char::LineFeed {
                nx = ox;
                ny += 16;
            } else {
                match ascii_char {
                    Char::ExclamationMark => {
                        rdr.pic(nx, ny, GraphicNum::EXPOINTPIC);
                        nx += 8;
                    }
                    Char::Apostrophe => {
                        rdr.pic(nx, ny, GraphicNum::APOSTROPHEPIC);
                        nx += 8;
                    }
                    Char::Space => {
                        nx += 16;
                    }
                    Char::Colon => {
                        rdr.pic(nx, ny, GraphicNum::COLONPIC);
                        nx += 8;
                    }
                    Char::PercentSign => {
                        rdr.pic(nx, ny, GraphicNum::PERCENTPIC);
                        nx += 16;
                    }
                    _ => {
                        let mut ch = ascii_char;
                        if ch >= Char::SmallA {
                            ch = Char::from_u8(ch as u8 - ASCII_ALPHA_RANGE).expect("valid ascii")
                        }
                        ch = Char::from_u8(ch as u8 - Char::Digit0 as u8).expect("valid ascii");

                        rdr.pic(nx, ny, ALPHA[ch as usize]);
                        nx += 16;
                    }
                }
            }
        }
    }
}

struct BjBreather {
    which: bool,
    max: u64,

    time_start: u64,
}

fn new_bj_breather() -> BjBreather {
    BjBreather {
        which: true,
        max: 10,
        time_start: 0,
    }
}

impl BjBreather {
    fn poll_breathe(&mut self, ticker: &time::Ticker, rdr: &VGARenderer) {
        if ticker.get_count() > (self.time_start + self.max) {
            self.which = !self.which;
            if self.which {
                rdr.pic(0, 16, GraphicNum::GUYPIC);
            } else {
                rdr.pic(0, 16, GraphicNum::GUY2PIC);
            }
            self.time_start = ticker.get_count();
            self.max = 35;
        }
    }
}

pub async fn preload_graphics(
    ticker: &time::Ticker,
    iw_config: &IWConfig,
    state: &GameState,
    prj: &ProjectionConfig,
    input: &Input,
    rdr: &VGARenderer,
) {
    draw_level(state, rdr);
    // TODO ClearSplitVWB() (is there split screen support?)

    rdr.bar(0, 0, 320, 200 - STATUS_LINES, 127);
    rdr.pic((20 - 14) * 8, 80 - 3 * 8, GraphicNum::GETPSYCHEDPIC);

    rdr.fade_in().await;

    preload(ticker, iw_config, rdr).await;

    input.wait_user_input(70).await;
    rdr.fade_out().await;

    draw_all_play_border(rdr, prj);
}

// Only fakes the pre-load since in iw all graphics are already loaded into
// memory. Simulates the thermometer update on the Get Psyched Screen only.
async fn preload(ticker: &time::Ticker, iw_config: &IWConfig, rdr: &VGARenderer) {
    let x = 160 - 14 * 8;
    let y = 80 - 3 * 8;
    let width = 28 * 8;
    let height = 48;
    let total = 100;
    for current in 0..total {
        let w = width - 10;
        rdr.bar(x + 5, y + height - 3, w, 2, BLACK);
        let w = (w * current) / total;
        if w > 0 {
            rdr.bar(x + 5, y + height - 3, w, 2, 0x37); //SECONDCOLOR
            rdr.bar(x + 5, y + height - 3, w - 1, 1, 0x32);
        }
        if !iw_config.options.fast_psyched {
            ticker.tics(1).await;
        }
    }
}
