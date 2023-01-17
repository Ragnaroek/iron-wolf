use crate::def::new_fixed_raw;

use super::{fixed_by_frac};

#[test]
fn test_fixed_by_frac() {
    assert_eq!(fixed_by_frac(new_fixed_raw(15750), new_fixed_raw(0)), new_fixed_raw(0));
    assert_eq!(fixed_by_frac(new_fixed_raw(0), new_fixed_raw(15750)), new_fixed_raw(0));
    assert_eq!(fixed_by_frac(new_fixed_raw(15750), new_fixed_raw(13625)), new_fixed_raw(3274));
    assert_eq!(fixed_by_frac(new_fixed_raw(21000), new_fixed_raw(13625)), new_fixed_raw(4365));
    assert_eq!(fixed_by_frac(new_fixed_raw(15750), new_fixed_raw(65446)), new_fixed_raw(15728));
    assert_eq!(fixed_by_frac(new_fixed_raw(15750), new_fixed_raw(-2147434188)), new_fixed_raw(-11886));
    assert_eq!(fixed_by_frac(new_fixed_raw(21000), new_fixed_raw(65446)), new_fixed_raw(20971));
    assert_eq!(fixed_by_frac(new_fixed_raw(14000), new_fixed_raw(-2147483648)), new_fixed_raw(0));
    assert_eq!(fixed_by_frac(new_fixed_raw(21000), new_fixed_raw(-2147434188)), new_fixed_raw(-15848));
    assert_eq!(fixed_by_frac(new_fixed_raw(-2147434188), new_fixed_raw(21000)), new_fixed_raw(-688112151));
}