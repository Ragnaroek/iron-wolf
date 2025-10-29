#![feature(test)]

extern crate iw;
extern crate test;

#[cfg(feature = "test")]
use {
    iw::assets,
    iw::config::default_iw_config,
    iw::def::new_game_state,
    iw::draw::{init_ray_cast, wall_refresh},
    iw::game::setup_game_level,
    iw::loader::DiskLoader,
    iw::rc::{Input, RenderContext},
    iw::sd,
    iw::start::new_view_size,
    iw::time::new_test_ticker,
    std::path::PathBuf,
    test::Bencher,
    vga::VGABuilder,
};

#[cfg(feature = "test")]
#[bench]
fn bench_ray_cast_loop(b: &mut Bencher) -> Result<(), String> {
    let mut data_path = PathBuf::new();
    data_path.push("./testdata/shareware_data");

    let mut iw_config = default_iw_config()?;
    iw_config.data.wolf3d_data = data_path;

    let loader = DiskLoader {
        variant: &assets::W3D1,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data,
    };

    let assets = assets::load_graphic_assets(&loader)?;

    let prj = new_view_size(19);
    let (graphics, fonts, tiles, texts) = assets::load_all_graphics(&loader, &None)?;

    let sound = sd::test_sound();

    let mut game_state = new_game_state();

    let vga = VGABuilder::new().video_mode(0x13).build()?;
    let input = Input::init_demo_playback(Vec::with_capacity(0));
    let ticker = new_test_ticker();

    let mut level_state = setup_game_level(&mut game_state, &assets, true).unwrap();
    let player = level_state.player();
    let mut cast = init_ray_cast(prj.view_width);
    cast.init_ray_cast_consts(&prj, player, 0);
    let mut rc = RenderContext::init(
        vga,
        ticker,
        graphics,
        fonts,
        tiles,
        texts,
        assets,
        &assets::W3D1,
        input,
        prj,
        cast,
        sound,
    );

    b.iter(|| {
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
        {
            let player = level_state.mut_player();
            player.x = 2283678;
            player.y = 3446039;
            player.angle = 98;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2428470;
            player.angle = 90;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2061034;
            player.angle = 334;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
        {
            let player = level_state.mut_player();
            player.x = 2246274;
            player.y = 833690;
            player.angle = 159;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
        {
            let player = level_state.mut_player();
            player.x = 2859077;
            player.y = 678021;
            player.angle = 290;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state);
        }
    });

    Ok(())
}
