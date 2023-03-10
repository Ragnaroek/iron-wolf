use super::vga_render::SCREENBWIDE;

#[derive(Debug)]
pub struct PixelScale {
    pub texture_src: usize,
    pub mem_dests: Vec<usize>,
}

#[derive(Debug)]
pub struct Scaler {
    pub for_height: usize,
    pub pixel_scalers: Vec<PixelScale>,
}

pub struct CompiledScaler {
    pub scalers : Vec<Scaler>
}

pub fn setup_scaling(scaler_height: usize, view_height: usize) -> CompiledScaler {
    let max_scale_height = scaler_height / 2;
    let step_by_two = view_height / 2;
    let mut scalers = Vec::new();

    let mut i = 1;
    while i <= max_scale_height {
        if i>max_scale_height {
            break;
        }

        let scaler_height = i*2;
        let scaler = build_comp_scale(scaler_height, view_height);
        println!("## scaler = {:?}", scaler);
        scalers.push(scaler);
        if i>=step_by_two {
            i +=2;
        }

        i += 1;
    } 

    CompiledScaler { scalers }
}

fn build_comp_scale(scaler_height: usize, view_height: usize) -> Scaler {
    println!("### height = {}", scaler_height);
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

        //println!("## start = {}, end = {}", start_pix, end_pix);

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

            scale.mem_dests.push(start_pix as usize * start_pix as usize * SCREENBWIDE);
        }
        scaler.pixel_scalers.push(scale);
    }
    scaler
}

/* 
fn full_scale(height: i32, view_height: usize, texture: &Texture, rdr: &dyn Renderer) {


    println!("height={}, view_height={}, step={}", height, view_height, step);
    let top_pix = (view_height as i32 - height)/2; 
    let mut fix = 0;
    for src in 0..=64 {
        let mut start_pix = fix >> 16;
        fix += step;
        let mut end_pix = fix >> 16;

        start_pix += top_pix;
        end_pix += top_pix;

        //println!("## start = {}, end = {}", start_pix, end_pix);

        if start_pix == end_pix || end_pix < 0 || start_pix >= view_height as i32 || src == 64 {
            continue
        }        



        for pix in start_pix..end_pix {
            if pix >= view_height as i32 {
                break;  // off the bottom of the view area
            }
            if pix < 0 {
                continue; // not into the view area
            }

            let pixel = texture.bytes[src as usize];
            rdr.write_mem(start_pix as usize * SCREENBWIDE, pixel);
        }
    }
}*/