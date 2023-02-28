use super::new_projection_config;
use super::new_fixed_i32;

#[test]
fn test_calc_sines() {
    let prj = new_projection_config(19);
    
    assert_eq!(prj.sin(0), new_fixed_i32(0));
    assert_eq!(prj.sin(360), new_fixed_i32(-2147483648));
    assert_eq!(prj.sin(180), new_fixed_i32(-2147483648));

    assert_eq!(prj.sin(16), new_fixed_i32(18064));
    assert_eq!(prj.sin(376), new_fixed_i32(18064));
    assert_eq!(prj.sin(164), new_fixed_i32(18064));
    assert_eq!(prj.sin(344), new_fixed_i32(-2147465584));
    assert_eq!(prj.sin(196), new_fixed_i32(-2147465584));

    assert_eq!(prj.sin(40), new_fixed_i32(42125));
    assert_eq!(prj.sin(400), new_fixed_i32(42125));
    assert_eq!(prj.sin(140), new_fixed_i32(42125));
    assert_eq!(prj.sin(320), new_fixed_i32(-2147441523));
    assert_eq!(prj.sin(220), new_fixed_i32(-2147441523));

    assert_eq!(prj.sin(64), new_fixed_i32(58903));
    assert_eq!(prj.sin(424), new_fixed_i32(58903));
    assert_eq!(prj.sin(116), new_fixed_i32(58903));
    assert_eq!(prj.sin(296), new_fixed_i32(-2147424745));
    assert_eq!(prj.sin(244), new_fixed_i32(-2147424745));

    assert_eq!(prj.sin(90), new_fixed_i32(65535), "sin(90)");
    assert_eq!(prj.sin(450), new_fixed_i32(65535), "sin(450)");
    assert_eq!(prj.sin(270), new_fixed_i32(-2147418112), "sin(270)");
}



