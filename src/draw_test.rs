use crate::def::new_fixed_i32;

use super::{fixed_by_frac};

#[test]
fn test_fixed_by_frac() {
    assert_eq!(fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(0)), new_fixed_i32(0));
    assert_eq!(fixed_by_frac(new_fixed_i32(0), new_fixed_i32(15750)), new_fixed_i32(0));
    assert_eq!(fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(13625)), new_fixed_i32(3274));
    assert_eq!(fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(13625)), new_fixed_i32(4365));
    assert_eq!(fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(65446)), new_fixed_i32(15728));
    assert_eq!(fixed_by_frac(new_fixed_i32(15750), new_fixed_i32(-2147434188)), new_fixed_i32(-11886));
    assert_eq!(fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65446)), new_fixed_i32(20971));
    assert_eq!(fixed_by_frac(new_fixed_i32(14000), new_fixed_i32(-2147483648)), new_fixed_i32(0));
    assert_eq!(fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(-2147434188)), new_fixed_i32(-15848));
    assert_eq!(fixed_by_frac(new_fixed_i32(-2147434188), new_fixed_i32(21000)), new_fixed_i32(-688112151));
    assert_eq!(fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65535)), new_fixed_i32(20999));
    assert_eq!(fixed_by_frac(new_fixed_i32(21000), new_fixed_i32(65536)), new_fixed_i32(0));
    assert_eq!(fixed_by_frac(new_fixed_i32(135055), new_fixed_i32(65535)), new_fixed_i32(135052));
    assert_eq!(fixed_by_frac(new_fixed_i32(165891), new_fixed_i32(65535)), new_fixed_i32(165888));
}