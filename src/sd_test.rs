use crate::{
    draw::{RayCast, init_ray_cast},
    fixed::new_fixed_i32,
};

use super::sound_loc;

#[test]
pub fn test_sound_loc() {
    let rc = ray_cast_for_tests(1948046, 3800915, -2147482505, 65526);
    let (left, right) = sound_loc(&rc, new_fixed_i32(2129920), new_fixed_i32(3768320));
    assert_eq!(left, 7);
    assert_eq!(right, 0);

    let rc = ray_cast_for_tests(2163412, 3431469, 64331, 12504);
    let (left, right) = sound_loc(&rc, new_fixed_i32(2129920), new_fixed_i32(3768320));
    assert_eq!(left, 7);
    assert_eq!(right, 2);

    let rc = ray_cast_for_tests(2440460, 4093380, -2147454919, 58903);
    let (left, right) = sound_loc(&rc, new_fixed_i32(2129920), new_fixed_i32(3768320));
    assert_eq!(left, 6);
    assert_eq!(right, 6);

    let rc = ray_cast_for_tests(2310125, 3506502, -2147471144, 64331);
    let (left, right) = sound_loc(&rc, new_fixed_i32(2260992), new_fixed_i32(3309568));
    assert_eq!(left, 1);
    assert_eq!(right, 2);
}

fn ray_cast_for_tests(view_x: i32, view_y: i32, view_cos: i32, view_sin: i32) -> RayCast {
    let mut rc = init_ray_cast(19);
    rc.view_x = view_x;
    rc.view_y = view_y;
    rc.view_cos = new_fixed_i32(view_cos);
    rc.view_sin = new_fixed_i32(view_sin);
    rc
}
