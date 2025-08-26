use super::calc_pixelangle;
use crate::fixed::Fixed;
use crate::start::new_view_size;

#[test]
fn test_calc_sines() {
    let prj = new_view_size(19);

    assert_eq!(prj.sin(0), Fixed::new_from_i32(0));
    assert_eq!(prj.sin(360), Fixed::new_from_i32(-2147483648));
    assert_eq!(prj.sin(180), Fixed::new_from_i32(-2147483648));

    assert_eq!(prj.sin(16), Fixed::new_from_i32(18064));
    assert_eq!(prj.sin(376), Fixed::new_from_i32(18064));
    assert_eq!(prj.sin(164), Fixed::new_from_i32(18064));
    assert_eq!(prj.sin(344), Fixed::new_from_i32(-2147465584));
    assert_eq!(prj.sin(196), Fixed::new_from_i32(-2147465584));

    assert_eq!(prj.sin(40), Fixed::new_from_i32(42125));
    assert_eq!(prj.sin(400), Fixed::new_from_i32(42125));
    assert_eq!(prj.sin(140), Fixed::new_from_i32(42125));
    assert_eq!(prj.sin(320), Fixed::new_from_i32(-2147441523));
    assert_eq!(prj.sin(220), Fixed::new_from_i32(-2147441523));

    assert_eq!(prj.sin(64), Fixed::new_from_i32(58903));
    assert_eq!(prj.sin(424), Fixed::new_from_i32(58903));
    assert_eq!(prj.sin(116), Fixed::new_from_i32(58903));
    assert_eq!(prj.sin(296), Fixed::new_from_i32(-2147424745));
    assert_eq!(prj.sin(244), Fixed::new_from_i32(-2147424745));

    assert_eq!(prj.sin(90), Fixed::new_from_i32(65535), "sin(90)");
    assert_eq!(prj.sin(450), Fixed::new_from_i32(65535), "sin(450)");
    assert_eq!(prj.sin(270), Fixed::new_from_i32(-2147418113), "sin(270)");
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
