use super::calc_pixelangle;
use super::calc_projection;
use crate::fixed::new_fixed_i32;

#[test]
fn test_calc_sines() {
    let prj = calc_projection(19);
    
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

#[test]
fn test_calc_pixelangles() {
    let angles = calc_pixelangle(304, 44800.0);

    assert_eq!(angles[151], 0);
    assert_eq!(angles[152], 0);

    assert_eq!(angles[165], -35);
    assert_eq!(angles[137], 38);

    assert_eq!(angles[182], -82);
    assert_eq!(angles[120], 84);

    assert_eq!(angles[214], -166);
    assert_eq!(angles[88], 168);

    assert_eq!(angles[291], -337);
    assert_eq!(angles[11], 339);

    assert_eq!(angles[0], 360);
    assert_eq!(angles[303], -360);
}


