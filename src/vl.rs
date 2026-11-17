use vga::{ColorReg, RenderBackend, VGA};

pub fn set_palette<R: RenderBackend>(vga: &mut VGA<R>, palette: &[u8]) {
    debug_assert_eq!(palette.len(), 768);
    vga.set_color_reg(ColorReg::AddressWriteMode, 0);
    for v in palette.iter().take(768) {
        vga.set_color_reg(ColorReg::Data, *v);
    }
}

pub fn get_palette<R: RenderBackend>(vga: &mut VGA<R>) -> Vec<u8> {
    let mut palette = Vec::with_capacity(768);
    vga.set_color_reg(ColorReg::AddressReadMode, 0);
    for _ in 0..768 {
        palette.push(vga.get_color_reg(ColorReg::Data));
    }
    palette
}

fn fill_palette<R: RenderBackend>(vga: &mut VGA<R>, red: u8, green: u8, blue: u8) {
    vga.set_color_reg(ColorReg::AddressWriteMode, 0);
    for _ in 0..256 {
        vga.set_color_reg(ColorReg::Data, red);
        vga.set_color_reg(ColorReg::Data, green);
        vga.set_color_reg(ColorReg::Data, blue);
    }
}

pub async fn fade_out<R: RenderBackend>(
    vga: &mut VGA<R>,
    start: usize,
    end: usize,
    red: u8,
    green: u8,
    blue: u8,
    steps: usize,
) {
    let palette_orig = get_palette(vga);
    let mut palette_new = palette_orig.clone();

    for i in 0..steps {
        let mut ix = 0;
        for _ in start..end {
            let orig = palette_orig[ix] as i32;
            let delta = red as i32 - orig;
            palette_new[ix] = (orig + (delta * i as i32 / steps as i32)) as u8;
            ix += 1;

            let orig = palette_orig[ix] as i32;
            let delta = green as i32 - orig;
            palette_new[ix] = (orig + (delta * i as i32 / steps as i32)) as u8;
            ix += 1;

            let orig = palette_orig[ix] as i32;
            let delta = blue as i32 - orig;
            palette_new[ix] = (orig + (delta * i as i32 / steps as i32)) as u8;
            ix += 1;
        }
        set_palette(vga, &palette_new);
    }

    fill_palette(vga, red, green, blue);
}

pub async fn fade_in<R: RenderBackend>(
    vga: &mut VGA<R>,
    start: usize,
    end: usize,
    palette: &[u8],
    steps: usize,
) {
    let palette1 = get_palette(vga);
    let mut palette2 = palette1.clone();

    let start = start * 3;
    let end = end * 3 + 2;

    for i in 0..steps {
        for j in start..end {
            let (sub, _) = palette[j].overflowing_sub(palette1[j]);
            let delta = sub as usize;
            let (add, _) = palette1[j].overflowing_add((delta * i / steps) as u8);
            palette2[j] = add;
        }
        set_palette(vga, &palette2);
    }
    set_palette(vga, palette);
}
