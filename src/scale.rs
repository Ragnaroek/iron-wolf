#[cfg(test)]
#[path = "./scale_test.rs"]
mod scale_test;

use libiw::gamedata::{SpriteData, SpritePost};

use crate::play::ProjectionConfig;
use crate::vga_render::{Renderer, SCREENBWIDE};

pub static MAP_MASKS_1 : [u8; 4*8] = [
    1 ,3 ,7 ,15,15,15,15,15,
    2 ,6 ,14,14,14,14,14,14,
    4 ,12,12,12,12,12,12,12,
    8 ,8 ,8 ,8 ,8 ,8 ,8 ,8
];

pub static MAP_MASKS_2 : [u8; 4*8] = [
    0 ,0 ,0 ,0 ,1 ,3 ,7 ,15,
    0 ,0 ,0 ,1 ,3 ,7 ,15,15,
    0 ,0 ,1 ,3 ,7 ,15,15,15,
    0 ,1 ,3 ,7 ,15,15,15,15 
];
    
pub static MAP_MASKS_3 : [u8; 4*8] = [
    0 ,0 ,0 ,0 ,0 ,0 ,0 ,0,
    0 ,0 ,0 ,0 ,0 ,0 ,0 ,1,
    0 ,0 ,0 ,0 ,0 ,0 ,1 ,3,
    0 ,0 ,0 ,0 ,0 ,1 ,3 ,7 
];

#[derive(Debug)]
pub struct PixelScale {
    pub texture_src: usize,
    pub mem_dests: Vec<i32>, //TODO change to an interval with start + length (mem_dests are always continous!)
}

#[derive(Debug)]
pub struct Scaler {
    pub for_height: usize,
    pub width: Vec<usize>,
    pub pixel_scalers: Vec<PixelScale>,
}

pub struct CompiledScaler {
    pub scalers : Vec<Scaler>,
    pub scale_call: Vec<usize>, //jump table into the pixel_scalers
}

pub fn setup_scaling(scaler_height: usize, view_height: usize) -> CompiledScaler {
    let max_scale_height = scaler_height / 2;
    let step_by_two = view_height / 2;
    let mut scalers = Vec::new();

    let mut i = 1;
    while i <= max_scale_height {
        let scaler = build_comp_scale(i*2, view_height);
        scalers.push(scaler);
        if i >= step_by_two {
            i += 2;
        }
        i += 1;
    } 

    let mut scale_call = vec![0; max_scale_height+1];
    let mut i = 1;
    let mut ptr = 0;
    while i <= max_scale_height {
        scale_call[i]=ptr;
        if i >= step_by_two {
            scale_call[i+1]=ptr;
            scale_call[i+2]=ptr;
            i += 2;
        }
        i += 1;
        ptr += 1;
    } 
    scale_call[0] = 1;

    CompiledScaler { scalers, scale_call }
}

fn build_comp_scale(scaler_height: usize, view_height: usize) -> Scaler {
    let step = ((scaler_height as i32) << 16) / 64;

    let top_pix = (view_height as i32 - scaler_height as i32)/2; 
    let mut fix : i32 = 0;
    let mut scaler = Scaler{
        for_height: scaler_height,
        width: Vec::with_capacity(64),
        pixel_scalers: Vec::new(),
    };

    for src  in 0..=(64 as usize) {
        let mut start_pix = fix >> 16;
        fix += step;
        let mut end_pix = fix >> 16;

        start_pix += top_pix;
        end_pix += top_pix;


        if end_pix > start_pix {
            scaler.width.push((end_pix - start_pix) as usize);
        } else {
            scaler.width.push(0);
        }

        if start_pix == end_pix || end_pix < 0 || start_pix >= view_height as i32 || src == 64 {
            continue
        }

        let mut scale = PixelScale{texture_src: src, mem_dests: Vec::new()};
        for pix in start_pix..end_pix {
            if pix >= view_height as i32 {
                break;  // off the bottom of the view area
            }
            if pix < 0 {
                continue; // not into the view area
            }

            scale.mem_dests.push(pix * SCREENBWIDE as i32);
        }
        scaler.pixel_scalers.push(scale);
    }
    scaler
}

// simple = no clipping
pub fn simple_scale_shape(rdr: &dyn Renderer, prj: &ProjectionConfig, x_center: usize, sprite: &SpriteData, height: usize) {
    let scaler = &prj.scaler.scalers[prj.scaler.scale_call[height >> 1]];
    
    // scale to the left (from pixel 31 to sprite.left_pix)
    let mut line_x = x_center;
    let mut src_x = 31;
    let mut stop_x = sprite.left_pix;
    let mut cmd_ptr = 31 - stop_x as i64;    
    while src_x >= stop_x {
        let posts = &sprite.posts[cmd_ptr as usize];
        cmd_ptr -= 1;
        let slinewidth = scaler.width[src_x];
        src_x -= 1;
        if slinewidth == 0 {
            continue;
        }

        line_x -= slinewidth;
        scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
    }

    // scale to the right
    line_x = x_center;
    stop_x = sprite.right_pix;
    if sprite.left_pix < 31 {
        src_x = 31;
        cmd_ptr = 32 - sprite.left_pix as i64;
    } else {
        src_x = sprite.left_pix - 1;
        cmd_ptr = 0;
    }
    src_x += 1;
    while src_x <= stop_x {
        let posts = &sprite.posts[cmd_ptr as usize];
        cmd_ptr += 1;
        let slinewidth = scaler.width[src_x];
        src_x += 1;
        if slinewidth == 0 {
            continue;
        }

        scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
        line_x += slinewidth;
    }
}

fn scale_line(rdr: &dyn Renderer, scaler: &Scaler, sprite: &SpriteData, posts: &Vec<SpritePost>, line_x: usize, slinewidth: usize) {
    let mut mem_offset = (line_x >> 2) + rdr.buffer_offset();
    let mask_ix = (((line_x & 3) << 3) + slinewidth)-1;

    let mask3 = MAP_MASKS_3[mask_ix];
    let mask2 = MAP_MASKS_2[mask_ix];
    let mask1 = MAP_MASKS_1[mask_ix];
    if mask3 != 0 {
        scale(rdr, scaler, sprite, posts, mem_offset, mask1);
        mem_offset += 1;
        scale(rdr, scaler, sprite, posts, mem_offset, mask2);
        mem_offset += 1;
        scale(rdr, scaler, sprite, posts, mem_offset, mask3); 
    } else if mask2 != 0 {
        scale(rdr, scaler, sprite, posts, mem_offset, mask1);
        mem_offset += 1;
        scale(rdr, scaler, sprite, posts, mem_offset, mask2);
    } else { //mask1
        scale(rdr, scaler, sprite, posts, mem_offset, mask1);
    }
}

fn scale(rdr: &dyn Renderer, scaler: &Scaler, sprite: &SpriteData, posts: &Vec<SpritePost>, mem_offset: usize, mask: u8) {
    rdr.set_mask(mask);
    for post in posts {
        let mut of = post.pixel_offset;
        for p in post.start..post.end {
            let pix_scaler = &scaler.pixel_scalers[p as usize];
            let pix = sprite.pixel_pool[of];
            for mem_dest in &pix_scaler.mem_dests {
                rdr.write_mem(mem_offset + *mem_dest as usize, pix)
            }
            of += 1;
        }
    }
}