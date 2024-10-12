#[cfg(test)]
#[path = "./scale_test.rs"]
mod scale_test;

use super::vga_render::SCREENBWIDE;

#[derive(Debug)]
pub struct PixelScale {
    pub texture_src: usize,
    pub mem_dests: Vec<i32>, //TODO change to an interval with start + length (mem_dests are always continous!)
}

#[derive(Debug)]
pub struct Scaler {
    pub for_height: usize,
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
        pixel_scalers: Vec::new(),
    };

    for src  in 0..=(64 as usize) {
        let mut start_pix = fix >> 16;
        fix += step;
        let mut end_pix = fix >> 16;

        start_pix += top_pix;
        end_pix += top_pix;
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