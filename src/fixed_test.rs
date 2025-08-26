use super::{Fixed, fixed_by_frac};

#[test]
fn test_fixed_by_frac() {
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(15750), Fixed::new_from_i32(0)),
        Fixed::new_from_i32(0)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(0), Fixed::new_from_i32(15750)),
        Fixed::new_from_i32(0)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(15750), Fixed::new_from_i32(13625)),
        Fixed::new_from_i32(3274)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(21000), Fixed::new_from_i32(13625)),
        Fixed::new_from_i32(4365)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(15750), Fixed::new_from_i32(65446)),
        Fixed::new_from_i32(15728)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(15750), Fixed::new_from_i32(-2147434188)),
        Fixed::new_from_i32(-11886)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(21000), Fixed::new_from_i32(65446)),
        Fixed::new_from_i32(20971)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(14000), Fixed::new_from_i32(-2147483648)),
        Fixed::new_from_i32(0)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(21000), Fixed::new_from_i32(-2147434188)),
        Fixed::new_from_i32(-15848)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(-2147434188), Fixed::new_from_i32(21000)),
        Fixed::new_from_i32(-688112151)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(21000), Fixed::new_from_i32(65535)),
        Fixed::new_from_i32(20999)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(21000), Fixed::new_from_i32(65536)),
        Fixed::new_from_i32(0)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(135055), Fixed::new_from_i32(65535)),
        Fixed::new_from_i32(135052)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(165891), Fixed::new_from_i32(65535)),
        Fixed::new_from_i32(165888)
    );
    assert_eq!(
        fixed_by_frac(Fixed::new_from_i32(14000), Fixed::new_from_i32(-2147418113)),
        Fixed::new_from_i32(-13999)
    );
}

#[test]
fn test_new_fixed() {
    assert_eq!(Fixed::new(0, 0).0, 0x0000_0000);
    assert_eq!(Fixed::new(1, 5).0, 0x0001_0005);
    assert_eq!(Fixed::new(98, 7539).0, 0x0062_1D73);

    assert_eq!(Fixed::new(-98, 7539).0, 0xFF9E_1D73u32 as i32);
    assert_eq!(Fixed::new_from_u32(0xFF9E_1D73u32).0, 0xFF9E_1D73u32 as i32)
}

#[test]
fn test_print_fixed() {
    assert_eq!(format!("{}", Fixed::new(0, 0)), "0.0");
    assert_eq!(format!("{}", Fixed::new(1, 5)), "1.5");
    assert_eq!(format!("{}", Fixed::new(98, 7539)), "98.7539");

    assert_eq!(format!("{}", Fixed::new(-98, 7539)), "-98.7539");
    assert_eq!(
        format!("{}", Fixed::new_from_u32(0xFF9E_1D73u32)),
        "-98.7539"
    );
}

#[test]
fn test_neg_fixed() {
    assert_eq!(-Fixed::new(0, 0), Fixed::new(0, 0));
    assert_eq!(-Fixed::new(-1, 0), Fixed::new(1, 0));
    assert_eq!(-Fixed::new(1, 0), Fixed::new(-1, 0));
    assert_eq!(
        -Fixed::new_from_i16(i16::MAX, 0),
        Fixed::new_from_i16(-i16::MAX, 0)
    );
}
