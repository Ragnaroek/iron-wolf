use super::{fixed_by_frac, new_fixed, new_fixed_i16, new_fixed_i32, new_fixed_u32};

#[test]
fn test_fixed_by_frac() {
    assert_eq!(
        fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(0)),
        new_fixed_i32(0)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(0), new_fixed_i32(15750)),
        new_fixed_i32(0)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(13625)),
        new_fixed_i32(3274)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(13625)),
        new_fixed_i32(4365)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(65446)),
        new_fixed_i32(15728)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(-2147434188)),
        new_fixed_i32(-11886)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65446)),
        new_fixed_i32(20971)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(14000), new_fixed_i32(-2147483648)),
        new_fixed_i32(0)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(-2147434188)),
        new_fixed_i32(-15848)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(-2147434188), new_fixed_i32(21000)),
        new_fixed_i32(-688112151)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65535)),
        new_fixed_i32(20999)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65536)),
        new_fixed_i32(0)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(135055), new_fixed_i32(65535)),
        new_fixed_i32(135052)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(165891), new_fixed_i32(65535)),
        new_fixed_i32(165888)
    );
    assert_eq!(
        fixed_by_frac(new_fixed_i32(14000), new_fixed_i32(-2147418113)),
        new_fixed_i32(-13999)
    );
}

#[test]
fn test_new_fixed() {
    assert_eq!(new_fixed(0, 0).0, 0x0000_0000);
    assert_eq!(new_fixed(1, 5).0, 0x0001_0005);
    assert_eq!(new_fixed(98, 7539).0, 0x0062_1D73);

    assert_eq!(new_fixed(-98, 7539).0, 0xFF9E_1D73u32 as i32);
    assert_eq!(new_fixed_u32(0xFF9E_1D73u32).0, 0xFF9E_1D73u32 as i32)
}

#[test]
fn test_print_fixed() {
    assert_eq!(format!("{}", new_fixed(0, 0)), "0.0");
    assert_eq!(format!("{}", new_fixed(1, 5)), "1.5");
    assert_eq!(format!("{}", new_fixed(98, 7539)), "98.7539");

    assert_eq!(format!("{}", new_fixed(-98, 7539)), "-98.7539");
    assert_eq!(format!("{}", new_fixed_u32(0xFF9E_1D73u32)), "-98.7539");
}

#[test]
fn test_neg_fixed() {
    assert_eq!(-new_fixed(0, 0), new_fixed(0, 0));
    assert_eq!(-new_fixed(-1, 0), new_fixed(1, 0));
    assert_eq!(-new_fixed(1, 0), new_fixed(-1, 0));
    assert_eq!(-new_fixed_i16(i16::MAX, 0), new_fixed_i16(-i16::MAX, 0));
}
