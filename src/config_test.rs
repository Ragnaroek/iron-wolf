use crate::config::default_iw_config;

#[test]
fn test_default_iw_config() {
    let conf = default_iw_config();
    assert!(conf.is_ok(), "{:?}", conf);
    assert!(conf.unwrap().vanilla);
}