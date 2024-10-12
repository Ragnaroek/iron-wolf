use super::setup_scaling;

#[test]
fn test_setup_scaling() {
    let compiled = setup_scaling(456, 152);
    assert_eq!(compiled.scalers.len(), 126);
    {
        let scaler_2 = &compiled.scalers[0];
        assert_eq!(scaler_2.for_height, 2);
        assert_eq!(scaler_2.pixel_scalers.len(), 64);
        assert_eq!(scaler_2.pixel_scalers[31].as_ref().unwrap().texture_src, 31);
        assert_eq!(scaler_2.pixel_scalers[31].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_2.pixel_scalers[31].as_ref().unwrap().mem_dests[0], 6000);
        assert_eq!(scaler_2.pixel_scalers[63].as_ref().unwrap().texture_src, 63);
        assert_eq!(scaler_2.pixel_scalers[63].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_2.pixel_scalers[63].as_ref().unwrap().mem_dests[0], 6080);
        for s in &scaler_2.pixel_scalers {
            if let Some(p) = s {
                assert!(p.texture_src == 31 || p.texture_src == 63)
            } else {
                assert!(s.is_none())
            }
        }
    }
    {
        let scaler_4 = &compiled.scalers[1];
        assert_eq!(scaler_4.for_height, 4);
        assert_eq!(scaler_4.pixel_scalers.len(), 64);
        assert_eq!(scaler_4.pixel_scalers[15].as_ref().unwrap().texture_src, 15);
        assert_eq!(scaler_4.pixel_scalers[15].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_4.pixel_scalers[15].as_ref().unwrap().mem_dests[0], 5920);
        assert_eq!(scaler_4.pixel_scalers[31].as_ref().unwrap().texture_src, 31);
        assert_eq!(scaler_4.pixel_scalers[31].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_4.pixel_scalers[31].as_ref().unwrap().mem_dests[0], 6000);
        assert_eq!(scaler_4.pixel_scalers[47].as_ref().unwrap().texture_src, 47); 
        assert_eq!(scaler_4.pixel_scalers[47].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_4.pixel_scalers[47].as_ref().unwrap().mem_dests[0], 6080);
        assert_eq!(scaler_4.pixel_scalers[63].as_ref().unwrap().texture_src, 63);
        assert_eq!(scaler_4.pixel_scalers[63].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_4.pixel_scalers[63].as_ref().unwrap().mem_dests[0], 6160);
        for s in &scaler_4.pixel_scalers {
            if let Some(p) = s {
                assert!(p.texture_src == 15 || p.texture_src == 31 || p.texture_src == 47 || p.texture_src == 63)
            } else {
                assert!(s.is_none())
            }
        }
    }
    {
        let scaler_150 = &compiled.scalers[74];
        assert_eq!(scaler_150.for_height, 150);
        assert_eq!(scaler_150.pixel_scalers.len(), 64);
        assert_eq!(scaler_150.pixel_scalers[0].as_ref().unwrap().texture_src, 0);
        assert_eq!(scaler_150.pixel_scalers[0].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_150.pixel_scalers[0].as_ref().unwrap().mem_dests[0], 80);
        assert_eq!(scaler_150.pixel_scalers[0].as_ref().unwrap().mem_dests[1], 160);
        assert_eq!(scaler_150.pixel_scalers[1].as_ref().unwrap().texture_src, 1);
        assert_eq!(scaler_150.pixel_scalers[1].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_150.pixel_scalers[1].as_ref().unwrap().mem_dests[0], 240);
        assert_eq!(scaler_150.pixel_scalers[1].as_ref().unwrap().mem_dests[1], 320);
        assert_eq!(scaler_150.pixel_scalers[32].as_ref().unwrap().texture_src, 32);
        assert_eq!(scaler_150.pixel_scalers[32].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_150.pixel_scalers[32].as_ref().unwrap().mem_dests[0], 6080);
        assert_eq!(scaler_150.pixel_scalers[32].as_ref().unwrap().mem_dests[1], 6160);
        assert_eq!(scaler_150.pixel_scalers[62].as_ref().unwrap().texture_src, 62);
        assert_eq!(scaler_150.pixel_scalers[62].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_150.pixel_scalers[62].as_ref().unwrap().mem_dests[0], 11680);
        assert_eq!(scaler_150.pixel_scalers[62].as_ref().unwrap().mem_dests[1], 11760);
        assert_eq!(scaler_150.pixel_scalers[63].as_ref().unwrap().texture_src, 63);
        assert_eq!(scaler_150.pixel_scalers[63].as_ref().unwrap().mem_dests.len(), 3);
        assert_eq!(scaler_150.pixel_scalers[63].as_ref().unwrap().mem_dests[0], 11840);
        assert_eq!(scaler_150.pixel_scalers[63].as_ref().unwrap().mem_dests[1], 11920);
        assert_eq!(scaler_150.pixel_scalers[63].as_ref().unwrap().mem_dests[2], 12000);
        for s in &scaler_150.pixel_scalers {
            assert!(s.is_some());
        }
    }
    {
        let scaler_152 = &compiled.scalers[75];
        assert_eq!(scaler_152.for_height, 152);
        assert_eq!(scaler_152.pixel_scalers.len(), 64);
        assert_eq!(scaler_152.pixel_scalers[0].as_ref().unwrap().texture_src, 0);
        assert_eq!(scaler_152.pixel_scalers[0].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_152.pixel_scalers[0].as_ref().unwrap().mem_dests[0], 0);
        assert_eq!(scaler_152.pixel_scalers[0].as_ref().unwrap().mem_dests[1], 80);
        assert_eq!(scaler_152.pixel_scalers[1].as_ref().unwrap().texture_src, 1);
        assert_eq!(scaler_152.pixel_scalers[1].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_152.pixel_scalers[1].as_ref().unwrap().mem_dests[0], 160);
        assert_eq!(scaler_152.pixel_scalers[1].as_ref().unwrap().mem_dests[1], 240);
        assert_eq!(scaler_152.pixel_scalers[32].as_ref().unwrap().texture_src, 32);
        assert_eq!(scaler_152.pixel_scalers[32].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_152.pixel_scalers[32].as_ref().unwrap().mem_dests[0], 6080);
        assert_eq!(scaler_152.pixel_scalers[32].as_ref().unwrap().mem_dests[1], 6160);
        assert_eq!(scaler_152.pixel_scalers[62].as_ref().unwrap().texture_src, 62);
        assert_eq!(scaler_152.pixel_scalers[62].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_152.pixel_scalers[62].as_ref().unwrap().mem_dests[0], 11760);
        assert_eq!(scaler_152.pixel_scalers[62].as_ref().unwrap().mem_dests[1], 11840);
        assert_eq!(scaler_152.pixel_scalers[63].as_ref().unwrap().texture_src, 63);
        assert_eq!(scaler_152.pixel_scalers[63].as_ref().unwrap().mem_dests.len(), 3);
        assert_eq!(scaler_152.pixel_scalers[63].as_ref().unwrap().mem_dests[0], 11920);
        assert_eq!(scaler_152.pixel_scalers[63].as_ref().unwrap().mem_dests[1], 12000);
        assert_eq!(scaler_152.pixel_scalers[63].as_ref().unwrap().mem_dests[2], 12080);
        for s in &scaler_152.pixel_scalers {
            assert!(s.is_some());
        }
    }
    {
        let scaler_158 = &compiled.scalers[76];
        assert_eq!(scaler_158.for_height, 158);
        assert_eq!(scaler_158.pixel_scalers.len(), 64);  
        assert_eq!(scaler_158.pixel_scalers[1].as_ref().unwrap().texture_src, 1);
        assert_eq!(scaler_158.pixel_scalers[1].as_ref().unwrap().mem_dests.len(), 1);
        assert_eq!(scaler_158.pixel_scalers[1].as_ref().unwrap().mem_dests[0], 0);
        assert_eq!(scaler_158.pixel_scalers[2].as_ref().unwrap().texture_src, 2);
        assert_eq!(scaler_158.pixel_scalers[2].as_ref().unwrap().mem_dests.len(), 3);
        assert_eq!(scaler_158.pixel_scalers[2].as_ref().unwrap().mem_dests[0], 80);
        assert_eq!(scaler_158.pixel_scalers[2].as_ref().unwrap().mem_dests[1], 160);
        assert_eq!(scaler_158.pixel_scalers[2].as_ref().unwrap().mem_dests[2], 240);
        assert_eq!(scaler_158.pixel_scalers[3].as_ref().unwrap().texture_src, 3);
        assert_eq!(scaler_158.pixel_scalers[3].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_158.pixel_scalers[3].as_ref().unwrap().mem_dests[0], 320);
        assert_eq!(scaler_158.pixel_scalers[3].as_ref().unwrap().mem_dests[1], 400);
        assert_eq!(scaler_158.pixel_scalers[32].as_ref().unwrap().texture_src, 32);
        assert_eq!(scaler_158.pixel_scalers[32].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_158.pixel_scalers[32].as_ref().unwrap().mem_dests[0], 6080);
        assert_eq!(scaler_158.pixel_scalers[32].as_ref().unwrap().mem_dests[1], 6160);
        assert_eq!(scaler_158.pixel_scalers[61].as_ref().unwrap().texture_src, 61);
        assert_eq!(scaler_158.pixel_scalers[61].as_ref().unwrap().mem_dests.len(), 3);
        assert_eq!(scaler_158.pixel_scalers[61].as_ref().unwrap().mem_dests[0], 11760);
        assert_eq!(scaler_158.pixel_scalers[61].as_ref().unwrap().mem_dests[1], 11840);
        assert_eq!(scaler_158.pixel_scalers[61].as_ref().unwrap().mem_dests[2], 11920);
        assert_eq!(scaler_158.pixel_scalers[62].as_ref().unwrap().texture_src, 62);
        assert_eq!(scaler_158.pixel_scalers[62].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_158.pixel_scalers[62].as_ref().unwrap().mem_dests[0], 12000);
        assert_eq!(scaler_158.pixel_scalers[62].as_ref().unwrap().mem_dests[1], 12080);
    }
    {
        let scaler_296 = &compiled.scalers[99];
        assert_eq!(scaler_296.for_height, 296);
        assert_eq!(scaler_296.pixel_scalers.len(), 64);
        assert_eq!(scaler_296.pixel_scalers[15].as_ref().unwrap().texture_src, 15);
        assert_eq!(scaler_296.pixel_scalers[15].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_296.pixel_scalers[15].as_ref().unwrap().mem_dests[0], 0);
        assert_eq!(scaler_296.pixel_scalers[15].as_ref().unwrap().mem_dests[1], 80);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().texture_src, 16);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().mem_dests.len(), 4);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().mem_dests[0], 160);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().mem_dests[1], 240);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().mem_dests[2], 320);
        assert_eq!(scaler_296.pixel_scalers[16].as_ref().unwrap().mem_dests[3], 400);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().texture_src, 17);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests.len(), 5);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests[0], 480);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests[1], 560);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests[2], 640);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests[3], 720);
        assert_eq!(scaler_296.pixel_scalers[17].as_ref().unwrap().mem_dests[4], 800);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().texture_src, 38);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests.len(), 5);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests[0], 8240);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests[1], 8320);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests[2], 8400);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests[3], 8480);
        assert_eq!(scaler_296.pixel_scalers[38].as_ref().unwrap().mem_dests[4], 8560);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().texture_src, 47);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests.len(), 5);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests[0], 11600);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests[1], 11680);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests[2], 11760);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests[3], 11840);
        assert_eq!(scaler_296.pixel_scalers[47].as_ref().unwrap().mem_dests[4], 11920);
        assert_eq!(scaler_296.pixel_scalers[48].as_ref().unwrap().texture_src, 48);
        assert_eq!(scaler_296.pixel_scalers[48].as_ref().unwrap().mem_dests.len(), 2);
        assert_eq!(scaler_296.pixel_scalers[48].as_ref().unwrap().mem_dests[0], 12000);
        assert_eq!(scaler_296.pixel_scalers[48].as_ref().unwrap().mem_dests[1], 12080);
    }
    {
        let scaler_452 = &compiled.scalers[125];
        assert_eq!(scaler_452.for_height, 452);
        assert_eq!(scaler_452.pixel_scalers.len(), 64);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().as_ref().unwrap().texture_src, 21);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests.len(), 5);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests[0], 0);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests[1], 80);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests[2], 160);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests[3], 240);
        assert_eq!(scaler_452.pixel_scalers[21].as_ref().unwrap().mem_dests[4], 320);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().texture_src, 22);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests.len(), 7);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[0], 400);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[1], 480);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[2], 560);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[3], 640);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[4], 720);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[5], 800);
        assert_eq!(scaler_452.pixel_scalers[22].as_ref().unwrap().mem_dests[6], 880);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().texture_src, 32);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests.len(), 7);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[0], 6080);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[1], 6160);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[2], 6240);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[3], 6320);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[4], 6400);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[5], 6480);
        assert_eq!(scaler_452.pixel_scalers[32].as_ref().unwrap().mem_dests[6], 6560);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().texture_src, 41);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests.len(), 7);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[0], 11120);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[1], 11200);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[2], 11280);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[3], 11360);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[4], 11440);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[5], 11520);
        assert_eq!(scaler_452.pixel_scalers[41].as_ref().unwrap().mem_dests[6], 11600);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().texture_src, 42);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests.len(), 6);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[0], 11680);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[1], 11760);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[2], 11840);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[3], 11920);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[4], 12000);
        assert_eq!(scaler_452.pixel_scalers[42].as_ref().unwrap().mem_dests[5], 12080);
    }
}