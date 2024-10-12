use core::{ascii, num};
use std::ascii::Char;

use vga::VGA;

use crate::{assets::{num_pic, GraphicNum}, def::{GameState, WindowState, STATUS_LINES}, input::Input, menu::{clear_ms_screen, draw_stripes}, vga_render::VGARenderer, vh::vw_fade_in};
use crate::time;

static ALPHA : [GraphicNum; 43] = [
    GraphicNum::NUM0PIC, GraphicNum::NUM1PIC, GraphicNum::NUM2PIC, GraphicNum::NUM3PIC, GraphicNum::NUM4PIC, GraphicNum::NUM5PIC,
    GraphicNum::NUM6PIC, GraphicNum::NUM7PIC, GraphicNum::NUM8PIC, GraphicNum::NUM9PIC, GraphicNum::COLONPIC, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::NONE, GraphicNum::APIC, GraphicNum::BPIC, 
    GraphicNum::CPIC, GraphicNum::DPIC, GraphicNum::EPIC, GraphicNum::FPIC, GraphicNum::GPIC, GraphicNum::HPIC, GraphicNum::IPIC, GraphicNum::JPIC, GraphicNum::KPIC, 
    GraphicNum::LPIC, GraphicNum::MPIC, GraphicNum::NPIC, GraphicNum::OPIC, GraphicNum::PPIC, GraphicNum::QPIC, GraphicNum::RPIC, GraphicNum::SPIC, GraphicNum::TPIC, 
    GraphicNum::UPIC, GraphicNum::VPIC, GraphicNum::WPIC, GraphicNum::XPIC, GraphicNum::YPIC, GraphicNum::ZPIC];

const ASCII_ALPHA_RANGE : u8 = Char::SmallA as u8 - Char::CapitalA as u8; // 'a' - 'A'

struct ParTime {
    time: f32,
    time_str: &'static str
}

const PAR_AMOUNT : i32 = 500;

static PAR_TIMES : [ParTime; 60] = [
	 // Episode One Par Times
     ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 3.5,	time_str: "03:30"},
	 ParTime{time: 3.0,	time_str: "03:00"},
	 ParTime{time: 3.0,	time_str: "03:00"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 0.0,	time_str: "??:??"},	// Boss level
	 ParTime{time: 0.0,	time_str: "??:??"},	// Secret level
     // Episode Two Par Times
	 ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 3.5,	time_str: "03:30"},
	 ParTime{time: 3.0,	time_str: "03:00"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 4.0,	time_str: "04:00"},
	 ParTime{time: 6.0,	time_str: "06:00"},
	 ParTime{time: 1.0,	time_str: "01:00"},
	 ParTime{time: 3.0,	time_str: "03:00"},
	 ParTime{time: 0.0,	time_str: "??:??"},
	 ParTime{time: 0.0,	time_str: "??:??"},
     // Episode Three Par Times
	 ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 3.5,	time_str: "03:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 6.0,	time_str: "06:00"},
	 ParTime{time: 0.0,	time_str: "??:??"},
	 ParTime{time: 0.0,	time_str: "??:??"},
     // Episode Four Par Times
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 1.0,	time_str: "01:00"},
	 ParTime{time: 4.5,	time_str: "04:30"},
	 ParTime{time: 3.5,	time_str: "03:30"},
	 ParTime{time: 2.0,	time_str: "02:00"},
	 ParTime{time: 4.5,	time_str: "04:30"},
	 ParTime{time: 0.0,	time_str: "??:??"},
	 ParTime{time: 0.0,	time_str: "??:??"},
     // Episode Five Par Times
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 1.5,	time_str: "01:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 2.5,	time_str: "02:30"},
	 ParTime{time: 4.0,	time_str: "04:00"},
	 ParTime{time: 3.0,	time_str: "03:00"},
	 ParTime{time: 4.5,	time_str: "04:30"},
	 ParTime{time: 3.5,	time_str: "03:30"},
	 ParTime{time: 0.0,	time_str: "??:??"},
	 ParTime{time: 0.0,	time_str: "??:??"},
     // Episode Six Par Times
	 ParTime{time: 6.5,	time_str: "06:30"},
	 ParTime{time: 4.0,	time_str: "04:00"},
	 ParTime{time: 4.5,	time_str: "04:30"},
	 ParTime{time: 6.0,	time_str: "06:00"},
	 ParTime{time: 5.0,	time_str: "05:00"},
	 ParTime{time: 5.5,	time_str: "05:30"},
	 ParTime{time: 5.5,	time_str: "05:30"},
	 ParTime{time: 8.5,	time_str: "08:30"},
	 ParTime{time: 0.0,	time_str: "??:??"},
	 ParTime{time: 0.0,	time_str: "??:??"}
];

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
    rdr.pic(4*8, 68, GraphicNum::CNAMEPIC);
    rdr.pic(20*8, 68, GraphicNum::CLEVELPIC);
    rdr.pic(28*8, 68, GraphicNum::CSCOREPIC);
}
/// LevelCompleted
///
/// Entered with the screen faded out
/// Still in split screen mode with the status bar
///
/// Exit with the screen faded out
pub async fn level_completed(ticker: &time::Ticker, vga: &VGA, rdr: &VGARenderer, input: &Input, game_state: &GameState, win_state: &mut WindowState) {
    rdr.set_buffer_offset(rdr.active_buffer());

    clear_split_vwb(win_state);
    rdr.bar(0, 0, 320, 200-STATUS_LINES, 127);
    // TODO StartCPMusic(ENDLEVEL_MUS)

    // do the intermission
    rdr.set_buffer_offset(rdr.active_buffer());
    rdr.pic(0, 16, GraphicNum::LGUYPIC);

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

        write(rdr, 26, 2, (game_state.map_on+1).to_string().as_str());
        let par_time = &PAR_TIMES[game_state.episode*10+game_state.map_on]; 
        write(rdr, 26, 12, par_time.time_str);

        let mut sec = game_state.time_count/70;
        if sec > 99*60 {
            sec = 99*60;
        }
        let time_left = if game_state.time_count < (par_time.time * 4200.0) as u64 {
            ((par_time.time * 4200.0 / 70.0) as u64 - sec) as i32
        } else {
            0
        };

        let min = sec / 60;
        sec %= 60;

        let mut i = 26*8;
        rdr.pic(i, 10*8, num_pic((min/10) as usize));
        i += 2*8;
        rdr.pic(i, 10*8, num_pic((min%10) as usize));
        i += 2*8;
        write(rdr, i/8, 10, ":");
        i += 2*8;
        rdr.pic(i, 10*8, num_pic((sec/10) as usize));
        i += 2*8;
        rdr.pic(i, 10*8, num_pic((sec%10) as usize));

        rdr.fade_in().await;

        // FIGURE RATIOS OUT BEFOREHAND
        let kr = if game_state.kill_total > 0 {
            (game_state.kill_count * 100) / game_state.kill_total
        } else {
            0
        };
        let sr = if game_state.secret_total > 0 {
            (game_state.secret_count * 100) / game_state.secret_total
        } else {
            0
        };
        let tr = if game_state.treasure_total > 0 {
            (game_state.treasure_total * 100) / game_state.treasure_total
        } else {
            0
        };

        // PRINT TIME BONUS
        let bonus = time_left * PAR_AMOUNT;
        if bonus > 0 {
            for i in 0..time_left {
              let str = (i*PAR_AMOUNT).to_string();
              let x = 36 - str.len() * 2;
              write(rdr, x, 7, &str);
              // TODO PlaySound(ENDBONUS1SND)
              // TODO Breath while sound is playing
              // TODO check for key press to skip animation
            }

            // TODO PlaySound(ENDBONUS2SND)
            // TODO Breath while sound is playing
        }

    } else {
        // TODO secret floot completed
    }

    rdr.pic(0, 110, GraphicNum::LGUY2PIC);

    // TODO write level complete data into screen

    vw_fade_in(vga).await;

    input.start_ack();
    while !input.check_ack() {
        bj_breather.breathe(ticker, rdr).await;
    }
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
                    },
                    Char::Apostrophe => {
                        rdr.pic(nx, ny, GraphicNum::APOSTROPHEPIC);
                        nx += 8;
                    },
                    Char::Space => {
                        nx += 16;
                    },
                    Char::Colon => {
                        rdr.pic(nx, ny, GraphicNum::COLONPIC);
                        nx += 8;
                    },
                    Char::PercentSign => {
                        rdr.pic(nx, ny, GraphicNum::PERCENTPIC);
                        nx += 16;
                    },
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
    which : bool,
    max: u64
}

fn new_bj_breather() -> BjBreather {
    BjBreather{which: true, max: 10}
}

impl BjBreather {
    // Breathe Mr. BJ!!!
    async fn breathe(&mut self, ticker: &time::Ticker, rdr: &VGARenderer) {
        println!("breath = {}", self.which);
        ticker.tics(self.max).await;

        self.which = !self.which;
        if self.which {
            println!("PIC1 {}", GraphicNum::LGUYPIC as usize);
            rdr.pic(0, 16, GraphicNum::LGUYPIC);
        } else {
            println!("PIC2 {}", GraphicNum::LGUY2PIC as usize);
            rdr.pic(0, 16, GraphicNum::LGUY2PIC);
        }
        self.max = 35;
    }
}
