use std::{fs::create_dir_all, path::PathBuf};

use crate::{
    assets::{self, WolfFile},
    config::default_iw_config,
    loader::Loader,
};

use super::{load_wolf_config, write_wolf_config};

#[test]
fn test_default_iw_config() {
    let conf = default_iw_config();
    assert!(conf.is_ok(), "{:?}", conf);
    assert!(conf.unwrap().vanilla);
}

#[test]
fn test_read_write_wolf_config() {
    let mut read_data_path = PathBuf::new();
    read_data_path.push("./testdata/shareware_data");

    let read_loader = Loader {
        variant: &assets::W3D1,
        data_path: read_data_path,
        patch_path: None,
    };

    let read_config = load_wolf_config(&read_loader);

    let mut write_data_path = PathBuf::new();
    write_data_path.push("./testdata/tmp_write");
    create_dir_all(&write_data_path).expect("create tmp dir");

    let write_loader = Loader {
        variant: &assets::W3D1,
        data_path: write_data_path,
        patch_path: None,
    };

    write_wolf_config(&write_loader, &read_config).expect("write config");

    let original_data = read_loader.load_wolf_file(WolfFile::ConfigData);
    let reloaded_data = write_loader.load_wolf_file(WolfFile::ConfigData);

    assert_eq!(reloaded_data.len(), original_data.len());
    for i in 0..original_data.len() {
        assert_eq!(
            reloaded_data[i], original_data[i],
            "bytes at position {} do not match",
            i
        );
    }
}
