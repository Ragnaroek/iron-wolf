#[cfg(test)]
#[path = "./scale_test.rs"]
mod scale_test;

use crate::gamedata::{SpriteData, SpritePost};
use crate::play::ProjectionConfig;
use crate::vga_render::{SCREENBWIDE, VGARenderer};

pub static MAP_MASKS_1: [u8; 4 * 8] = [
    1, 3, 7, 15, 15, 15, 15, 15, 2, 6, 14, 14, 14, 14, 14, 14, 4, 12, 12, 12, 12, 12, 12, 12, 8, 8,
    8, 8, 8, 8, 8, 8,
];

pub static MAP_MASKS_2: [u8; 4 * 8] = [
    0, 0, 0, 0, 1, 3, 7, 15, 0, 0, 0, 1, 3, 7, 15, 15, 0, 0, 1, 3, 7, 15, 15, 15, 0, 1, 3, 7, 15,
    15, 15, 15,
];

pub static MAP_MASKS_3: [u8; 4 * 8] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 0, 0, 1, 3, 7,
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
    pub pixel_scalers: Vec<Option<PixelScale>>,
}

pub struct CompiledScaler {
    pub scalers: Vec<Scaler>,
    pub scale_call: Vec<usize>, //jump table into the pixel_scalers
    pub max_scale: usize,
}

pub fn setup_scaling(scaler_height: usize, view_height: usize) -> CompiledScaler {
    let max_scale_height = scaler_height / 2;
    let step_by_two = view_height / 2;
    let mut scalers = Vec::new();

    let mut i = 1;
    while i <= max_scale_height {
        let scaler = build_comp_scale(i * 2, view_height);
        scalers.push(scaler);
        if i >= step_by_two {
            i += 2;
        }
        i += 1;
    }

    let mut scale_call = vec![0; max_scale_height + 3];
    let mut i = 1;
    let mut ptr = 0;
    while i <= max_scale_height {
        scale_call[i] = ptr;
        if i >= step_by_two {
            scale_call[i + 1] = ptr;
            scale_call[i + 2] = ptr;
            i += 2;
        }
        i += 1;
        ptr += 1;
    }
    scale_call[0] = 1;

    CompiledScaler {
        scalers,
        scale_call,
        max_scale: max_scale_height - 1,
    }
}

fn build_comp_scale(scaler_height: usize, view_height: usize) -> Scaler {
    let step = ((scaler_height as i32) << 16) / 64;

    let top_pix = (view_height as i32 - scaler_height as i32) / 2;
    let mut fix: i32 = 0;
    let mut pix_scaler_init = Vec::new();
    pix_scaler_init.resize_with(64, Default::default);
    let mut scaler = Scaler {
        for_height: scaler_height,
        width: Vec::with_capacity(64),
        pixel_scalers: pix_scaler_init,
    };

    for src in 0..=(64 as usize) {
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
            continue;
        }

        let mut scale = PixelScale {
            texture_src: src,
            mem_dests: Vec::new(),
        };
        for pix in start_pix..end_pix {
            if pix >= view_height as i32 {
                break; // off the bottom of the view area
            }
            if pix < 0 {
                continue; // not into the view area
            }

            scale.mem_dests.push(pix * SCREENBWIDE as i32);
        }
        scaler.pixel_scalers[src] = Some(scale);
    }
    scaler
}

pub fn scale_shape(
    rdr: &VGARenderer,
    wall_height: &Vec<i32>,
    prj: &ProjectionConfig,
    x_center: usize,
    sprite: &SpriteData,
    height: usize,
) {
    let scale = height >> 3;
    if scale == 0 || scale > prj.scaler.max_scale {
        return;
    }
    let scaler = &prj.scaler.scalers[prj.scaler.scale_call[scale]];

    // scale to the left (from pixel 31 to sprite.left_pix)
    let mut line_x = x_center;
    let mut src_x = 31;
    let mut stop_x = sprite.left_pix;
    let mut cmd_ptr = 31 - stop_x as i64;
    while src_x >= stop_x && line_x > 0 {
        let posts = &sprite.posts[cmd_ptr as usize];
        cmd_ptr -= 1;
        let mut slinewidth = scaler.width[src_x];
        src_x -= 1;
        if slinewidth == 0 {
            continue;
        }

        // handle single pixel line
        if slinewidth == 1 {
            line_x -= 1;
            if line_x < prj.view_width {
                if wall_height[line_x] >= height as i32 {
                    continue; // obscured by closer wall
                }
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            }
            continue;
        }

        if line_x > prj.view_width {
            line_x -= slinewidth;
            slinewidth = prj.view_width.saturating_sub(line_x);
            if slinewidth == 0 {
                continue;
            }
        } else {
            if slinewidth > line_x {
                slinewidth = line_x;
            }
            line_x -= slinewidth;
        }

        let left_vis = wall_height[line_x] < height as i32;
        let right_vis = wall_height[line_x + slinewidth - 1] < height as i32;

        if left_vis {
            if right_vis {
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            } else {
                // find first visible line from the right
                while wall_height[line_x + slinewidth - 1] >= height as i32 {
                    slinewidth -= 1;
                }
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            }
        } else {
            if !right_vis {
                continue; // totally obscured
            }
            // find first visible line from the left
            while wall_height[line_x] >= height as i32 {
                line_x += 1;
                slinewidth -= 1;
            }
            scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            break; // the rest of the left part of the shape is gone
        }
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

    let mut slinewidth = 0;
    while src_x <= stop_x {
        line_x += slinewidth;
        if line_x >= prj.view_width {
            break; // right part of sprite is out of view
        }

        let posts = &sprite.posts[cmd_ptr as usize];
        cmd_ptr += 1;
        slinewidth = scaler.width[src_x];
        src_x += 1;

        // handle single pixe lines
        if slinewidth == 1 {
            if wall_height[line_x] < height as i32 {
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            }
            continue;
        }

        // handle multi pixel lines
        if (line_x + slinewidth) > prj.view_width {
            slinewidth = prj.view_width - line_x;
        }
        if slinewidth == 0 {
            continue;
        }

        let left_vis = wall_height[line_x] < height as i32;
        let right_vis = wall_height[line_x + slinewidth - 1] < height as i32;
        if left_vis {
            if right_vis {
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            } else {
                while wall_height[line_x + slinewidth - 1] >= height as i32 {
                    slinewidth -= 1;
                }
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
                break; // the rest of the shape is gone
            }
        } else {
            if right_vis {
                while wall_height[line_x] >= height as i32 {
                    line_x += 1;
                    slinewidth -= 1;
                }
                scale_line(rdr, scaler, sprite, posts, line_x, slinewidth);
            } else {
                continue; // totally obscurred
            }
        }
    }
}

// simple = no clipping
pub fn simple_scale_shape(
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    x_center: usize,
    sprite: &SpriteData,
    height: usize,
) {
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

fn scale_line(
    rdr: &VGARenderer,
    scaler: &Scaler,
    sprite: &SpriteData,
    posts: &Vec<SpritePost>,
    line_x: usize,
    slinewidth: usize,
) {
    let mut mem_offset = (line_x >> 2) + rdr.buffer_offset();
    let mask_ix = (((line_x & 3) << 3) + slinewidth) - 1;

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
    } else {
        //mask1
        scale(rdr, scaler, sprite, posts, mem_offset, mask1);
    }
}

fn scale(
    rdr: &VGARenderer,
    scaler: &Scaler,
    sprite: &SpriteData,
    posts: &Vec<SpritePost>,
    mem_offset: usize,
    mask: u8,
) {
    rdr.set_mask(mask);
    for post in posts {
        let mut of = post.pixel_offset;
        for p in post.start..post.end {
            if let Some(pix_scaler) = &scaler.pixel_scalers[p] {
                let pix = sprite.pixel_pool[of];
                for mem_dest in &pix_scaler.mem_dests {
                    rdr.write_mem(mem_offset + *mem_dest as usize, pix)
                }
            }
            of += 1;
        }
    }
}
