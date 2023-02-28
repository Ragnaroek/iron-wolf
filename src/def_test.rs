use super::{new_fixed, new_fixed_u32, new_fixed_i16};

#[test]
fn test_new_fixed() {
    assert_eq!(new_fixed(0,0).0, 0x0000_0000);
    assert_eq!(new_fixed(1,5).0, 0x0001_0005);
    assert_eq!(new_fixed(98,7539).0, 0x0062_1D73);
    
    assert_eq!(new_fixed(-98,7539).0, 0xFF9E_1D73u32 as i32);
    assert_eq!(new_fixed_u32(0xFF9E_1D73u32).0, 0xFF9E_1D73u32 as i32)
}

#[test]
fn test_print_fixed() {
    assert_eq!(format!("{}", new_fixed(0,0)), "0.0");
    assert_eq!(format!("{}", new_fixed(1,5)), "1.5");
    assert_eq!(format!("{}", new_fixed(98,7539)), "98.7539");
    
    assert_eq!(format!("{}", new_fixed(-98,7539)), "-98.7539");
    assert_eq!(format!("{}", new_fixed_u32(0xFF9E_1D73u32)), "-98.7539");
}

#[test]
fn test_neg_fixed() {
    assert_eq!(-new_fixed(0,0), new_fixed(0,0));
    assert_eq!(-new_fixed(-1,0), new_fixed(1,0));
    assert_eq!(-new_fixed(1,0), new_fixed(-1,0));
    assert_eq!(-new_fixed_i16(i16::MAX,0), new_fixed_i16(-i16::MAX,0));
}