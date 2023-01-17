use super::new_projection_config;

#[test]
fn test_calc_sines() {
    let prj = new_projection_config(19);
    
    assert_eq!(prj.sin(0), 0);
    assert_eq!(prj.sin(360), -2147483648);
    assert_eq!(prj.sin(180), -2147483648);

    assert_eq!(prj.sin(16), 18064);
    assert_eq!(prj.sin(376), 18064);
    assert_eq!(prj.sin(164), 18064);
    assert_eq!(prj.sin(344), -2147465584);
    assert_eq!(prj.sin(196), -2147465584);

    assert_eq!(prj.sin(40), 42125);
    assert_eq!(prj.sin(400), 42125);
    assert_eq!(prj.sin(140), 42125);
    assert_eq!(prj.sin(320), -2147441523);
    assert_eq!(prj.sin(220), -2147441523);

    assert_eq!(prj.sin(64), 58903);
    assert_eq!(prj.sin(424), 58903);
    assert_eq!(prj.sin(116), 58903);
    assert_eq!(prj.sin(296), -2147424745);
    assert_eq!(prj.sin(244), -2147424745);

    // VANILLA this is off by one with the original
    // I assume different floating point behaviour on the
    // original hardware / DOSBOX, but did not investigate
    assert_eq!(prj.sin(90), 65536, "sin(90)");
    assert_eq!(prj.sin(450), 65536, "sin(450)");
    assert_eq!(prj.sin(270), -2147418112, "sin(270)");
}



