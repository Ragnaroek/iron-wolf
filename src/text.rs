use vga::input::NumCode;

use crate::assets::{GAMEPAL, GraphicNum};
use crate::start::quit;
use crate::us1::{draw_string, measure_string};
use crate::vl;
use crate::{input::Input, vga_render::VGARenderer};

const BACK_COLOR: u8 = 0x11;

const WORD_LIMIT: usize = 80;
const TOP_MARGIN: usize = 16;
const BOTTOM_MARGIN: usize = 32;
const LEFT_MARGIN: usize = 16;
const RIGHT_MARGIN: usize = 16;
const FONT_HEIGHT: usize = 10;
const PIC_MARGIN: usize = 8;
const TEXT_ROWS: usize = (200 - TOP_MARGIN - BOTTOM_MARGIN) / FONT_HEIGHT;
const SPACE_WIDTH: usize = 7;
const SCREEN_PIX_WIDTH: usize = 320;
const SCREEN_MID: usize = SCREEN_PIX_WIDTH / 2;

struct LayoutContext {
    left_margin: [usize; TEXT_ROWS],
    right_margin: [usize; TEXT_ROWS],
    px: usize,
    py: usize,
    row_on: usize,
    page_num: usize,
    num_pages: usize,
    layout_done: bool,
    font_color: u8,
    font_number: usize,
}

impl LayoutContext {
    fn new() -> LayoutContext {
        LayoutContext {
            left_margin: [0; TEXT_ROWS as usize],
            right_margin: [0; TEXT_ROWS as usize],
            px: LEFT_MARGIN,
            py: TOP_MARGIN,
            row_on: 0,
            page_num: 1,
            num_pages: 0,
            layout_done: false,
            font_color: 0,
            font_number: 0,
        }
    }
}

struct Text {
    text: Vec<char>,
    ptr: usize, // index of the next char!
}

impl Text {
    fn new(str: &str) -> Text {
        Text {
            text: str.chars().collect(),
            ptr: 0,
        }
    }

    fn prev(&mut self) -> Option<char> {
        if self.ptr == 0 {
            return None;
        }
        self.ptr -= 1;
        Some(self.text[self.ptr])
    }

    fn next(&mut self) -> Option<char> {
        if self.ptr >= self.text.len() {
            return None;
        }
        let ch = Some(self.text[self.ptr]);
        self.ptr += 1;
        ch
    }

    fn peek(&self) -> Option<char> {
        if self.ptr >= self.text.len() {
            return None;
        }
        Some(self.text[self.ptr])
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

pub async fn end_text(rdr: &VGARenderer, input: &Input, which: usize) {
    show_article(rdr, input, which).await;

    rdr.fade_out().await;
}

async fn show_article(rdr: &VGARenderer, input: &Input, which: usize) {
    rdr.bar(0, 0, 320, 200, BACK_COLOR);

    let mut text = Text::new(&rdr.texts[which]);

    let mut layout_ctx = LayoutContext::new();
    layout_ctx.num_pages = 2;

    let mut new_page = true;
    let mut first_page = true;
    loop {
        if new_page {
            new_page = false;
            page_layout(&mut layout_ctx, rdr, &mut text, true);
        }

        if first_page {
            vl::fade_in(&rdr.vga, 0, 255, GAMEPAL, 10).await;
            first_page = false;
        }

        input.ack().await;
        match input.last_scan() {
            NumCode::Escape => break,
            NumCode::UpArrow | NumCode::PgUp | NumCode::LeftArrow => {
                if layout_ctx.page_num > 1 {
                    back_page(&mut text);
                    layout_ctx.page_num -= 1;
                    new_page = true;
                }
            }
            NumCode::Return | NumCode::DownArrow | NumCode::PgDn | NumCode::RightArrow => {
                if layout_ctx.page_num < layout_ctx.num_pages {
                    layout_ctx.page_num += 1;
                    new_page = true;
                }
            }
            _ => { /*ignore */ }
        }
    }

    input.clear_keys_down();
}

/// Clears the screen, draws the pics on the page, and word wraps the text.
fn page_layout(
    layout_ctx: &mut LayoutContext,
    rdr: &VGARenderer,
    text: &mut Text,
    show_number: bool,
) {
    rdr.bar(0, 0, 320, 200, BACK_COLOR);
    rdr.pic(0, 0, GraphicNum::HTOPWINDOWPIC);
    rdr.pic(0, 8, GraphicNum::HLEFTWINDOWPIC);
    rdr.pic(312, 8, GraphicNum::HRIGHTWINDOWPIC);
    rdr.pic(8, 176, GraphicNum::HBOTTOMINFOPIC);

    for i in 0..(TEXT_ROWS as usize) {
        layout_ctx.left_margin[i] = LEFT_MARGIN;
        layout_ctx.right_margin[i] = SCREEN_PIX_WIDTH - RIGHT_MARGIN;
    }
    layout_ctx.px = LEFT_MARGIN;
    layout_ctx.py = TOP_MARGIN;
    layout_ctx.row_on = 0;
    layout_ctx.layout_done = false;

    // make sure we are starting layout text (^P first command)
    text.skip_whitespace();
    let ch0 = text.next();
    if ch0 != Some('^') || text.next() != Some('P') {
        quit(Some("PageLayout: Text not headed with ^P"));
    }

    rip_to_eol(text);

    loop {
        let opt_ch = text.peek();
        if let Some(ch) = opt_ch {
            if ch == '^' {
                text.next();
                let result = handle_command(layout_ctx, rdr, text);
                if result.is_err() {
                    quit(Some(&format!(
                        "PageLayout: Illegal command {:?}",
                        result.err()
                    )));
                };
            } else if ch == '\t' {
                text.next();
                layout_ctx.px = (layout_ctx.px + 8) & 0xf8;
            } else if ch.is_whitespace() {
                handle_ctrls(layout_ctx, text);
            } else {
                let result = handle_word(layout_ctx, rdr, text);
                if result.is_err() {
                    quit(Some(&format!(
                        "PageLayout: cannot layout word {:?}",
                        result.err()
                    )));
                }
            }
        } else {
            break;
        }

        if layout_ctx.layout_done {
            break;
        }
    }

    if show_number {
        let font = &rdr.fonts[layout_ctx.font_number];
        draw_string(
            rdr,
            font,
            &format!("pg {} of {}", layout_ctx.page_num, layout_ctx.num_pages),
            213,
            183,
            0x4F,
        );
    }
}

fn back_page(text: &mut Text) {
    loop {
        let ch0 = text.prev();
        let ch1 = text.prev().map(|c| c.to_ascii_uppercase());
        if ch1 == Some('^') && ch0 == Some('P') {
            return;
        }
    }
}

fn handle_command(
    layout_ctx: &mut LayoutContext,
    rdr: &VGARenderer,
    text: &mut Text,
) -> Result<(), String> {
    let cmd_opt = text.next().map(|c| c.to_ascii_uppercase());
    match cmd_opt {
        None => {}
        Some('P') | Some('E') => {
            // ^P is start of next page, ^E is end of file
            layout_ctx.layout_done = true;
            text.prev();
            text.prev(); // back up to the '^'
        }
        Some('C') => {
            layout_ctx.font_color = parse_hex_u8(text)?;
        }
        Some('G') => {
            let g = parse_pic_command(text)?;
            rdr.pic(g.pic_x & !7, g.pic_y, g.pic_num);
            let pic_num = g.pic_num as usize - rdr.variant.start_pics;
            let graphic_data = &rdr.graphics[pic_num];
            // adjust margins
            let pic_mid = g.pic_x + graphic_data.width / 2;
            let margin = if pic_mid > SCREEN_MID {
                g.pic_x - PIC_MARGIN
            } else {
                g.pic_x + graphic_data.width + PIC_MARGIN
            };

            let top = g.pic_y.saturating_sub(TOP_MARGIN) / FONT_HEIGHT;
            let mut bottom = (g.pic_y + graphic_data.height - TOP_MARGIN) / FONT_HEIGHT;
            if bottom > TEXT_ROWS {
                bottom = TEXT_ROWS - 1;
            }

            for i in top..bottom {
                if pic_mid > SCREEN_MID {
                    layout_ctx.right_margin[i] = margin;
                } else {
                    layout_ctx.left_margin[i] = margin;
                }
            }

            // adjust this line if needed
            if layout_ctx.px < layout_ctx.left_margin[layout_ctx.row_on] {
                layout_ctx.px = layout_ctx.left_margin[layout_ctx.row_on];
            }
        }
        _ => {
            todo!("impl command {:?}", cmd_opt) /* ignore unknow command */
        }
    }
    Ok(())
}

fn handle_ctrls(layout_ctx: &mut LayoutContext, text: &mut Text) {
    if let Some('\n') = text.next() {
        new_line(layout_ctx, text);
    }
}

fn new_line(layout_ctx: &mut LayoutContext, text: &mut Text) {
    layout_ctx.row_on += 1;
    if layout_ctx.row_on == TEXT_ROWS {
        // overflowed the page, so skip until next page break
        layout_ctx.layout_done = true;
        loop {
            let opt_char = text.next();
            if let Some('^') = opt_char {
                let opt_c = text.peek().map(|c| c.to_ascii_uppercase());
                if opt_c == Some('E') || opt_c == Some('P') {
                    layout_ctx.layout_done = true;
                    text.prev(); //back up to ^
                    return;
                }
            }
        }
    }

    layout_ctx.px = layout_ctx.left_margin[layout_ctx.row_on];
    layout_ctx.py += FONT_HEIGHT;
}

fn handle_word(
    layout_ctx: &mut LayoutContext,
    rdr: &VGARenderer,
    text: &mut Text,
) -> Result<(), String> {
    let mut word = "".to_string();
    loop {
        if let Some(ch) = text.peek() {
            if !ch.is_whitespace() {
                word.push(ch);
                text.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    if word.chars().count() > WORD_LIMIT {
        return Err("Word limit exceeded".to_string());
    }

    let font = &rdr.fonts[layout_ctx.font_number];
    let (w, _) = measure_string(font, &word);

    while layout_ctx.px + w > layout_ctx.right_margin[layout_ctx.row_on] {
        new_line(layout_ctx, text);
        if layout_ctx.layout_done {
            return Ok(()); // overflowed page
        }
    }

    // print it
    let new_pos = layout_ctx.px + w;
    draw_string(
        rdr,
        font,
        &word,
        layout_ctx.px,
        layout_ctx.py,
        layout_ctx.font_color,
    );
    layout_ctx.px = new_pos;

    // suck up any extra spaces
    let mut num_spaces = 0;
    loop {
        let peek = text.peek();
        if let Some(' ') = peek {
            num_spaces += 1;
            text.next();
        } else {
            break;
        }
    }
    layout_ctx.px += num_spaces * SPACE_WIDTH;

    Ok(())
}

#[derive(Debug)]
struct G {
    pic_y: usize,
    pic_x: usize,
    pic_num: GraphicNum,
}

fn parse_pic_command(text: &mut Text) -> Result<G, String> {
    let pic_y = parse_number(text)?;
    if text.next() != Some(',') {
        return Err("expected , in pic command".to_string());
    }
    let pic_x = parse_number(text)?;
    if text.next() != Some(',') {
        return Err("expected , in pic command".to_string());
    }
    let g_val = parse_number(text)?;
    let pic_num = GraphicNum::try_from(g_val);
    if pic_num.is_err() {
        return Err(format!(
            "illegal graphic chunk id in G command: {:?}",
            g_val
        ));
    }
    rip_to_eol(text);
    Ok(G {
        pic_y,
        pic_x,
        pic_num: pic_num.unwrap(),
    })
}

fn parse_hex_u8(text: &mut Text) -> Result<u8, String> {
    let mut hex_str = "".to_string();
    let n1 = text.next();
    if n1.is_some() {
        hex_str.push(n1.unwrap());
    }
    let n2 = text.next();
    if n2.is_some() {
        hex_str.push(n2.unwrap());
    }

    u8::from_str_radix(&hex_str, 16).map_err(|e| e.to_string())
}

fn parse_number(text: &mut Text) -> Result<usize, String> {
    let mut num_str = "".to_string();
    loop {
        if let Some(ch) = text.peek() {
            if ch.is_numeric() {
                num_str.push(ch);
                text.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let num = num_str.parse::<usize>().map_err(|e| e.to_string())?;
    Ok(num)
}

fn rip_to_eol(text: &mut Text) {
    while text.next() != Some('\n') {}
}
