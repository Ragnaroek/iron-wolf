use std::time::Duration;

use crate::act1::{operate_door, push_wall};
use crate::act2::spawn_bj_victory;
use crate::assets::{GraphicNum, SoundName, face_pic, n_pic, weapon_pic};
use crate::def::{
    ALT_ELEVATOR_TILE, ANGLES, ANGLES_I32, Assets, At, Button, ClassType, ControlState, Difficulty,
    Dir, DirType, ELEVATOR_TILE, EXIT_TILE, EXTRA_POINTS, FL_NEVERMARK, FL_SHOOTABLE, FL_VISABLE,
    GameState, LevelState, MAP_SIZE, MIN_ACTOR_DIST, MIN_DIST, ObjKey, ObjType, PLAYER_SIZE,
    PUSHABLE_TILE, PlayState, SCREENLOC, STATUS_LINES, Sprite, StateType, StaticKind, StaticType,
    TILEGLOBAL, TILESHIFT, WeaponType,
};
use crate::draw::RayCast;
use crate::fixed::{ZERO, fixed_by_frac, new_fixed_i32};
use crate::game::AREATILE;
use crate::map;
use crate::play::{ProjectionConfig, start_bonus_flash, start_damage_flash};
use crate::sd::Sound;
use crate::state::{check_line, damage_actor};
use crate::us1::draw_string;
use crate::user::rnd_t;
use crate::vga_render::VGARenderer;

const ANGLE_SCALE: i32 = 20;
const MOVE_SCALE: i32 = 150;
const BACKMOVE_SCALE: i32 = 100;

pub static DUMMY_PLAYER: ObjType = ObjType {
    class: ClassType::Player,
    distance: 0,
    area_number: 0,
    active: crate::def::ActiveType::No,
    tic_count: 0,
    angle: 0,
    flags: FL_NEVERMARK,
    pitch: 0,
    tilex: 0,
    tiley: 0,
    view_x: 0,
    view_height: 0,
    trans_x: ZERO,
    trans_y: ZERO,
    x: 0,
    y: 0,
    speed: 0,
    dir: DirType::NoDir,
    temp1: 0,
    temp2: 0,
    temp3: 0,
    state: Some(&S_PLAYER),
    hitpoints: 0,
};

pub static S_PLAYER: StateType = StateType {
    id: 10,
    rotate: 0,
    sprite: None,
    tic_time: 0,
    think: Some(t_player),
    action: None,
    next: None,
};

pub static S_ATTACK: StateType = StateType {
    id: 11,
    rotate: 0,
    sprite: None,
    tic_time: 0,
    think: Some(t_attack),
    action: None,
    next: None,
};

struct AttackInfo {
    tics: i32,
    attack: i32, // TODO: use enum here
    frame: usize,
}

static ATTACK_INFO: [[AttackInfo; 4]; 4] = [
    [
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 1,
        },
        AttackInfo {
            tics: 6,
            attack: 2,
            frame: 2,
        },
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 3,
        },
        AttackInfo {
            tics: 6,
            attack: -1,
            frame: 4,
        },
    ],
    [
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 1,
        },
        AttackInfo {
            tics: 6,
            attack: 1,
            frame: 2,
        },
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 3,
        },
        AttackInfo {
            tics: 6,
            attack: -1,
            frame: 4,
        },
    ],
    [
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 1,
        },
        AttackInfo {
            tics: 6,
            attack: 1,
            frame: 2,
        },
        AttackInfo {
            tics: 6,
            attack: 3,
            frame: 3,
        },
        AttackInfo {
            tics: 6,
            attack: -1,
            frame: 4,
        },
    ],
    [
        AttackInfo {
            tics: 6,
            attack: 0,
            frame: 1,
        },
        AttackInfo {
            tics: 6,
            attack: 1,
            frame: 2,
        },
        AttackInfo {
            tics: 6,
            attack: 4,
            frame: 3,
        },
        AttackInfo {
            tics: 6,
            attack: -1,
            frame: 4,
        },
    ],
];

fn t_attack(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
    rc: &RayCast,
) {
    update_face(tics, game_state, sound, rdr);

    if game_state.victory_flag {
        victory_spin(tics, level_state);
        return;
    }

    if control_state.button_state[Button::Use as usize]
        && !control_state.button_held[Button::Use as usize]
    {
        control_state.button_state[Button::Use as usize] = false;
    }

    if control_state.button_state[Button::Attack as usize]
        && !control_state.button_held[Button::Attack as usize]
    {
        control_state.button_state[Button::Attack as usize] = false;
    }

    control_movement(
        k,
        level_state,
        game_state,
        sound,
        control_state,
        prj,
        assets,
    );

    if game_state.victory_flag {
        return;
    }
    {
        let player = level_state.mut_player();
        player.tilex = (player.x >> TILESHIFT) as usize;
        player.tiley = (player.y >> TILESHIFT) as usize;
    }

    game_state.attack_count -= tics as i32;
    while game_state.attack_count <= 0 {
        let cur =
            &ATTACK_INFO[game_state.weapon.expect("weapon") as usize][game_state.attack_frame];
        match cur.attack {
            -1 => {
                level_state.update_obj(k, |obj| obj.state = Some(&S_PLAYER));
                if game_state.ammo <= 0 {
                    game_state.weapon = Some(WeaponType::Knife);
                    draw_weapon(&game_state, rdr);
                } else {
                    if game_state.weapon.is_some()
                        && game_state.weapon.unwrap() != game_state.chosen_weapon
                    {
                        game_state.weapon = Some(game_state.chosen_weapon);
                        draw_weapon(&game_state, rdr);
                    }
                }
                game_state.attack_frame = 0;
                game_state.weapon_frame = 0;
                return;
            }
            4 => {
                if game_state.ammo == 0 {
                    // can only happen with chain gun
                    game_state.attack_frame += 1;
                    break;
                }
                if control_state.button_state[Button::Attack as usize] {
                    game_state.attack_frame -= 2;
                }
                weapon_attack(level_state, game_state, sound, rdr, prj, assets, rc);
            }
            1 => {
                weapon_attack(level_state, game_state, sound, rdr, prj, assets, rc);
            }
            2 => {
                knife_attack(level_state, game_state, rdr, prj, sound, assets, rc);
            }
            3 => {
                if game_state.ammo != 0 && control_state.button_state[Button::Attack as usize] {
                    game_state.attack_frame -= 2;
                }
            }
            _ => { /* do nothing */ }
        }

        game_state.attack_count += cur.tics;
        game_state.attack_frame += 1;
        game_state.weapon_frame =
            ATTACK_INFO[game_state.weapon.unwrap() as usize][game_state.attack_frame].frame;
    }
}

fn weapon_attack(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    assets: &Assets,
    rc: &RayCast,
) {
    if game_state.ammo == 0 {
        // can only happen with chain gun
        game_state.attack_frame += 1;
        return;
    }
    gun_attack(level_state, game_state, sound, rdr, prj, assets, rc);
    game_state.ammo -= 1;
    draw_ammo(&game_state, rdr);
}

fn knife_attack(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    sound: &mut Sound,
    assets: &Assets,
    rc: &RayCast,
) {
    sound.play_sound(SoundName::ATKKNIFE, assets);

    let mut dist = 0x7fffffff;
    let mut closest = None;
    for i in 1..level_state.actors.len() {
        let k = ObjKey(i);
        if level_state.actors.exists(k) {
            let check = &level_state.actors.obj(k);
            if check.flags & FL_SHOOTABLE != 0
                && check.flags & FL_VISABLE != 0
                && check.view_x.abs_diff(prj.center_x as i32) < prj.shoot_delta as u32
            {
                if check.trans_x.to_i32() < dist {
                    dist = check.trans_x.to_i32();
                    closest = Some(ObjKey(i));
                }
            }
        }
    }

    if closest.is_none() || dist > 0x18000 {
        return; //missed
    }
    damage_actor(
        closest.expect("closest enemy"),
        level_state,
        game_state,
        rdr,
        sound,
        assets,
        rc,
        (rnd_t() >> 4) as usize,
    )
}

fn gun_attack(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    prj: &ProjectionConfig,
    assets: &Assets,
    rc: &RayCast,
) {
    match game_state.weapon {
        Some(WeaponType::Pistol) => {
            sound.play_sound(SoundName::ATKPISTOL, assets);
        }
        Some(WeaponType::MachineGun) => {
            sound.play_sound(SoundName::ATKMACHINEGUN, assets);
        }
        Some(WeaponType::ChainGun) => {
            sound.play_sound(SoundName::ATKGATLING, assets);
        }
        _ => { /* ignore anything else */ }
    }
    game_state.made_noise = true;

    let mut view_dist = 0x7fffffff;
    let mut closest = None;
    let mut old_closest;
    loop {
        old_closest = closest;

        for i in 1..level_state.actors.len() {
            let k = ObjKey(i);
            if level_state.actors.exists(k) {
                let check = &level_state.actors.obj(k);
                if check.flags & FL_SHOOTABLE != 0
                    && check.flags & FL_VISABLE != 0
                    && check.view_x.abs_diff(prj.center_x as i32) < prj.shoot_delta as u32
                {
                    if check.trans_x.to_i32() < view_dist {
                        view_dist = check.trans_x.to_i32();
                        closest = Some(k);
                    }
                }
            }
        }
        if closest == old_closest {
            return; // no more targets, all missed
        }

        let obj = level_state.obj(closest.expect("closest enemy"));
        // trace a line from player to enemy
        if check_line(level_state, obj) {
            break;
        }
    }

    // hit something

    let k = closest.expect("closest enemy");
    let obj = level_state.obj(k);

    let dx = obj.tilex.abs_diff(level_state.player().tilex);
    let dy = obj.tiley.abs_diff(level_state.player().tiley);
    let dist = dx.max(dy);

    let damage;
    if dist < 2 {
        damage = rnd_t() / 4;
    } else if dist < 4 {
        damage = rnd_t() / 6;
    } else {
        if rnd_t() as usize / 12 < dist {
            // missed
            return;
        }
        damage = rnd_t() / 6;
    }

    damage_actor(
        k,
        level_state,
        game_state,
        rdr,
        sound,
        assets,
        rc,
        damage as usize,
    );
}

fn victory_spin(tics: u64, level_state: &mut LevelState) {
    let player = level_state.mut_player();

    if player.angle > 270 {
        player.angle -= tics as i32 * 3;
        if player.angle < 270 {
            player.angle = 270;
        }
    } else if player.angle < 270 {
        player.angle += tics as i32 * 3;
        if player.angle > 270 {
            player.angle = 270;
        }
    }

    let dest_y = (((player.tiley - 5) << TILESHIFT) - 0x3000) as i32;

    if player.y > dest_y {
        player.y -= tics as i32 * 4096;
        if player.y < dest_y {
            player.y = dest_y;
        }
    }
}

fn t_player(
    k: ObjKey,
    tics: u64,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
    rc: &RayCast,
) {
    if game_state.victory_flag {
        victory_spin(tics, level_state);
        return;
    }

    update_face(tics, game_state, sound, rdr);
    check_weapon_change(game_state, rdr, control_state);

    if control_state.button_state(Button::Use) {
        cmd_use(level_state, game_state, sound, assets, rc, control_state);
    }

    if control_state.button_state(Button::Attack) && !control_state.button_held(Button::Attack) {
        cmd_fire(level_state, game_state, control_state);
    }

    control_movement(
        k,
        level_state,
        game_state,
        sound,
        control_state,
        prj,
        assets,
    );
    if game_state.victory_flag {
        return;
    }
    // TODO plux and tilex update?
}

fn cmd_fire(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    control_state: &mut ControlState,
) {
    control_state.button_held[Button::Attack as usize] = true;

    game_state.weapon_frame = 0;

    level_state.mut_player().state = Some(&S_ATTACK);

    game_state.attack_frame = 0;
    if let Some(weapon) = game_state.weapon {
        game_state.attack_count = ATTACK_INFO[weapon as usize][game_state.attack_frame].tics;
        game_state.weapon_frame = ATTACK_INFO[weapon as usize][game_state.attack_frame].frame;
    }
}

fn cmd_use(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    assets: &Assets,
    rc: &RayCast,
    control_state: &mut ControlState,
) {
    let check_x;
    let check_y;
    let dir;
    let elevator_ok;
    // find which cardinal direction the player is facing
    let player = level_state.player();
    if player.angle < ANGLES_I32 / 8 || player.angle > 7 * ANGLES_I32 / 8 {
        check_x = player.tilex + 1;
        check_y = player.tiley;
        dir = Dir::East;
        elevator_ok = true;
    } else if player.angle < 3 * ANGLES_I32 / 8 {
        check_x = player.tilex;
        check_y = player.tiley - 1;
        dir = Dir::North;
        elevator_ok = false;
    } else if player.angle < 5 * ANGLES_I32 / 8 {
        check_x = player.tilex - 1;
        check_y = player.tiley;
        dir = Dir::West;
        elevator_ok = true;
    } else {
        check_x = player.tilex;
        check_y = player.tiley + 1;
        dir = Dir::South;
        elevator_ok = false;
    }

    if level_state.level.info_map[check_x][check_y] == PUSHABLE_TILE {
        push_wall(
            level_state,
            game_state,
            sound,
            assets,
            check_x,
            check_y,
            dir,
        );
        return;
    }

    let doornum = level_state.level.tile_map[check_x][check_y];
    if !control_state.button_held(Button::Use) && doornum == ELEVATOR_TILE && elevator_ok {
        // use elevator
        control_state.set_button_held(Button::Use, true);

        if level_state.level.map_segs.segs[0][player.tiley * MAP_SIZE + player.tilex]
            == ALT_ELEVATOR_TILE
        {
            game_state.play_state = PlayState::SecretLevel;
        } else {
            game_state.play_state = PlayState::Completed;
        }
        level_state.level.tile_map[check_x][check_y] += 1; // flip switch [to animate the lever to move up]

        sound.play_sound(SoundName::LEVELDONE, assets);
        while sound.is_any_sound_playing() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    if !control_state.button_held(Button::Use) && doornum & 0x80 != 0 {
        control_state.set_button_held(Button::Use, true);
        operate_door(
            (doornum & !0x80) as usize,
            level_state,
            game_state,
            sound,
            assets,
            rc,
        );
    } else {
        sound.play_sound(SoundName::DONOTHING, assets);
    }
}

pub fn spawn_player(
    tile_x: usize,
    tile_y: usize,
    map_data: &map::MapSegs,
    area_by_player: &mut Vec<bool>,
    dir: i32,
) -> ObjType {
    let area_number = (map_data.segs[0][tile_y * MAP_SIZE + tile_x] - AREATILE) as usize;
    let mut player = ObjType {
        class: ClassType::Player,
        distance: 0,
        area_number,
        active: crate::def::ActiveType::Yes,
        tic_count: 0,
        angle: (1 - dir) * 90,
        flags: FL_NEVERMARK,
        pitch: 0,
        tilex: tile_x,
        tiley: tile_y,
        view_x: 0,
        view_height: 0,
        trans_x: new_fixed_i32(0),
        trans_y: new_fixed_i32(0),
        x: ((tile_x as i32) << TILESHIFT) + TILEGLOBAL / 2,
        y: ((tile_y as i32) << TILESHIFT) + TILEGLOBAL / 2,
        speed: 0,
        dir: DirType::NoDir,
        temp1: 0,
        temp2: 0,
        temp3: 0,
        state: Some(&S_PLAYER),
        hitpoints: 0, // player hitpoints are maintained in GameState::health
    };

    if player.angle < 0 {
        player.angle += ANGLES as i32;
    }

    init_areas(player, area_by_player);

    player
}

fn init_areas(player: ObjType, area_by_player: &mut Vec<bool>) {
    area_by_player.fill(false); // clear it (makes a difference if there multiple player positions and only the last is kept)
    area_by_player[player.area_number] = true;
}

pub fn take_damage(
    attacker: ObjKey,
    points_param: i32,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    rdr: &VGARenderer,
) {
    if game_state.victory_flag {
        return;
    }

    let mut points = points_param;

    level_state.last_attacker = Some(attacker);
    if game_state.victory_flag {
        return;
    }
    if game_state.difficulty == Difficulty::Baby {
        points >>= 2;
    }
    if !game_state.god_mode {
        game_state.health -= points;
    }

    if game_state.health <= 0 {
        game_state.health = 0;
        game_state.play_state = PlayState::Died;
        game_state.killer_obj = Some(attacker);
    }

    start_damage_flash(game_state, points);

    game_state.got_gat_gun = false;

    draw_health(game_state, rdr);
    draw_face(game_state, rdr);

    // TODO SPEAR make eyes bug on major damage
}

fn check_weapon_change(
    game_state: &mut GameState,
    rdr: &VGARenderer,
    control_state: &mut ControlState,
) {
    if game_state.ammo == 0 {
        return; // must use knife with no ammo
    }

    for i in (WeaponType::Knife as usize)..=(game_state.best_weapon as usize) {
        let weapon_i = i - WeaponType::Knife as usize;

        if control_state.button_state(Button::from_usize(Button::ReadyKnife as usize + weapon_i)) {
            let weapon = WeaponType::from_usize(weapon_i);
            game_state.weapon = Some(weapon);
            game_state.chosen_weapon = weapon;
            draw_weapon(game_state, rdr);
            return;
        }
    }
}

fn control_movement(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    control_state: &mut ControlState,
    prj: &ProjectionConfig,
    assets: &Assets,
) {
    level_state.thrustspeed = 0;

    // side to side move
    if control_state.button_state(Button::Strafe) {
        // strafing
        let ob = level_state.obj(k);

        if control_state.control.x > 0 {
            let mut angle = ob.angle - ANGLES_I32 / 4;
            if angle < 0 {
                angle += ANGLES_I32;
            }
            thrust(
                k,
                level_state,
                game_state,
                sound,
                prj,
                assets,
                angle,
                control_state.control.x * MOVE_SCALE,
            ); // move to left
        } else if control_state.control.x < 0 {
            let mut angle = ob.angle + ANGLES_I32 / 4;
            if angle >= ANGLES_I32 {
                angle -= ANGLES_I32;
            }
            thrust(
                k,
                level_state,
                game_state,
                sound,
                prj,
                assets,
                angle,
                -control_state.control.x * MOVE_SCALE,
            ); // move to right
        }
    } else {
        // not strafing
        let control_x = control_state.control.x;

        control_state.angle_frac += control_x;
        let angle_units = control_state.angle_frac / ANGLE_SCALE;
        control_state.angle_frac -= angle_units * ANGLE_SCALE;

        {
            let ob = level_state.mut_obj(k);
            ob.angle -= angle_units;
            if ob.angle >= ANGLES as i32 {
                ob.angle -= ANGLES as i32;
            }
            if ob.angle < 0 {
                ob.angle += ANGLES as i32;
            }
        }
    }

    // forward/backwards move

    let control_y = control_state.control.y;
    let ob = level_state.obj(k);
    if control_y < 0 {
        thrust(
            k,
            level_state,
            game_state,
            sound,
            prj,
            assets,
            ob.angle,
            -control_y * MOVE_SCALE,
        )
    } else if control_y > 0 {
        let mut angle = ob.angle + ANGLES as i32 / 2;
        if angle >= ANGLES as i32 {
            angle -= ANGLES as i32;
        }
        thrust(
            k,
            level_state,
            game_state,
            sound,
            prj,
            assets,
            angle,
            control_y * BACKMOVE_SCALE,
        );
    }

    if game_state.victory_flag {
        return;
    }

    // TODO playerxmove?
}

fn victory_tile(level_state: &mut LevelState, game_state: &mut GameState) {
    spawn_bj_victory(level_state); // TODO don't do this in SPEAR
    game_state.victory_flag = true;
}

pub fn thrust_player(level_state: &mut LevelState) -> usize {
    let offset = {
        let player = level_state.mut_player();
        player.tilex = player.x as usize >> TILESHIFT; // scale to tile values
        player.tiley = player.y as usize >> TILESHIFT;
        player.tiley * MAP_SIZE + player.tilex
    };

    let (area, _) =
        (level_state.level.map_segs.segs[0][offset] as u8).overflowing_sub(AREATILE as u8);

    let player = level_state.mut_player();
    player.area_number = area as usize;

    offset
}

pub fn thrust(
    k: ObjKey,
    level_state: &mut LevelState,
    game_state: &mut GameState,
    sound: &mut Sound,
    prj: &ProjectionConfig,
    assets: &Assets,
    angle: i32,
    speed_param: i32,
) {
    //TODO reset funnyticount (only for Spear?)
    level_state.thrustspeed += speed_param;

    let speed = new_fixed_i32(if speed_param >= MIN_DIST * 2 {
        MIN_DIST * 2 - 1
    } else {
        speed_param
    });

    let x_move = fixed_by_frac(speed, prj.cos(angle as usize));
    let y_move = -fixed_by_frac(speed, prj.sin(angle as usize));

    clip_move(
        k,
        level_state,
        sound,
        assets,
        x_move.to_i32(),
        y_move.to_i32(),
    );

    let offset = thrust_player(level_state);
    if level_state.level.map_segs.segs[1][offset] == EXIT_TILE {
        victory_tile(level_state, game_state);
    }
}

pub fn give_points(
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    points: u32,
) {
    game_state.score += points;
    while game_state.score >= game_state.next_extra {
        game_state.next_extra += EXTRA_POINTS;
        give_extra_man(game_state, rdr, sound, assets);
    }
    draw_score(&game_state, rdr)
}

pub fn give_extra_man(
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
) {
    if game_state.lives < 9 {
        game_state.lives += 1;
    }
    draw_lives(game_state, rdr);
    sound.play_sound(SoundName::BONUS1UP, assets);
}

pub fn get_bonus(
    game_state: &mut GameState,
    rdr: &VGARenderer,
    sound: &mut Sound,
    assets: &Assets,
    check: &mut StaticType,
) {
    match check.item_number {
        StaticKind::BoFirstaid => {
            if game_state.health == 100 {
                return;
            }
            sound.play_sound(SoundName::HEALTH2, assets);
            heal_self(game_state, rdr, 25);
        }
        StaticKind::BoKey1 | StaticKind::BoKey2 | StaticKind::BoKey3 | StaticKind::BoKey4 => {
            give_key(game_state, rdr, check.item_number);
            sound.play_sound(SoundName::GETKEY, assets);
        }
        StaticKind::BoCross => {
            sound.play_sound(SoundName::BONUS1, assets);
            give_points(game_state, rdr, sound, assets, 100);
            game_state.treasure_count += 1;
        }
        StaticKind::BoChalice => {
            sound.play_sound(SoundName::BONUS2, assets);
            give_points(game_state, rdr, sound, assets, 500);
            game_state.treasure_count += 1;
        }
        StaticKind::BoBible => {
            sound.play_sound(SoundName::BONUS3, assets);
            give_points(game_state, rdr, sound, assets, 1000);
            game_state.treasure_count += 1;
        }
        StaticKind::BoCrown => {
            sound.play_sound(SoundName::BONUS4, assets);
            give_points(game_state, rdr, sound, assets, 5000);
            game_state.treasure_count += 1;
        }
        StaticKind::BoClip => {
            if game_state.ammo == 99 {
                return;
            }
            sound.play_sound(SoundName::GETAMMO, assets);
            give_ammo(game_state, rdr, 8);
        }
        StaticKind::BoClip2 => {
            if game_state.ammo == 99 {
                return;
            }
            sound.play_sound(SoundName::GETAMMO, assets);
            give_ammo(game_state, rdr, 4);
        }
        StaticKind::BoMachinegun => {
            sound.play_sound(SoundName::GETMACHINE, assets);
            give_weapon(game_state, rdr, WeaponType::MachineGun);
        }
        StaticKind::BoChaingun => {
            sound.play_sound(SoundName::GETGATLING, assets);
            give_weapon(game_state, rdr, WeaponType::ChainGun);

            status_draw_pic(rdr, 17, 4, GraphicNum::GOTGATLINGPIC);
            game_state.face_count = 0;
            game_state.got_gat_gun = true;
        }
        StaticKind::BoFullheal => {
            sound.play_sound(SoundName::BONUS1UP, assets);
            heal_self(game_state, rdr, 99);
            give_ammo(game_state, rdr, 25);
            give_extra_man(game_state, rdr, sound, assets);
            game_state.treasure_count += 1;
        }
        StaticKind::BoFood => {
            if game_state.health == 100 {
                return;
            }
            sound.play_sound(SoundName::HEALTH1, assets);
            heal_self(game_state, rdr, 10);
        }
        StaticKind::BoAlpo => {
            if game_state.health == 100 {
                return;
            }
            sound.play_sound(SoundName::HEALTH1, assets);
            heal_self(game_state, rdr, 4);
        }
        StaticKind::BoGibs => {
            if game_state.health > 10 {
                return;
            }
            sound.play_sound(SoundName::SLURPIE, assets);
            heal_self(game_state, rdr, 1);
        }
        StaticKind::BoSpear => {
            panic!("get spear");
        }
        _ => { /* ignore all other static kinds */ }
    }
    start_bonus_flash(game_state);
    check.sprite = Sprite::None; // remove from list
}

fn give_key(game_state: &mut GameState, rdr: &VGARenderer, key: StaticKind) {
    let key_value = match key {
        StaticKind::BoKey1 => 1 << 0,
        StaticKind::BoKey2 => 1 << 1,
        StaticKind::BoKey3 => 1 << 2,
        StaticKind::BoKey4 => 1 << 3,
        _ => panic!("give_key called with non key StaticKind: {:?}", key),
    };

    game_state.keys |= key_value;
    draw_keys(game_state, rdr);
}

fn heal_self(game_state: &mut GameState, rdr: &VGARenderer, points: i32) {
    game_state.health += points;
    if game_state.health > 100 {
        game_state.health = 100;
    }
    draw_health(&game_state, rdr);

    game_state.got_gat_gun = false;

    draw_face(&game_state, rdr);
}

fn give_weapon(game_state: &mut GameState, rdr: &VGARenderer, weapon: WeaponType) {
    give_ammo(game_state, rdr, 6);
    if game_state.best_weapon < weapon {
        game_state.best_weapon = weapon;
        game_state.weapon = Some(weapon);
        game_state.chosen_weapon = weapon;
    }
    draw_weapon(game_state, rdr);
}

fn give_ammo(game_state: &mut GameState, rdr: &VGARenderer, ammo: i32) {
    if game_state.ammo <= 0 {
        // knife was out
        if game_state.attack_frame <= 0 {
            game_state.weapon = Some(game_state.chosen_weapon);
            draw_weapon(&game_state, rdr)
        }
    }
    game_state.ammo += ammo;
    if game_state.ammo > 99 {
        game_state.ammo = 99;
    }
    draw_ammo(game_state, rdr);
}

fn clip_move(
    k: ObjKey,
    level_state: &mut LevelState,
    sound: &mut Sound,
    assets: &Assets,
    x_move: i32,
    y_move: i32,
) {
    let (base_x, base_y) = {
        let ob = level_state.obj(k);
        (ob.x, ob.y)
    };

    set_move(k, level_state, base_x + x_move, base_y + y_move);
    if try_move(k, level_state) {
        return;
    }
    // TODO add noclip check here (for cheats)

    if !sound.is_any_sound_playing() {
        sound.play_sound(SoundName::HITWALL, assets);
    }

    set_move(k, level_state, base_x + x_move, base_y);
    if try_move(k, level_state) {
        return;
    }

    set_move(k, level_state, base_x, base_y + y_move);
    if try_move(k, level_state) {
        return;
    }

    set_move(k, level_state, base_x, base_y);
}

fn try_move(k: ObjKey, level_state: &mut LevelState) -> bool {
    let ob = level_state.obj(k);

    let mut xl = (ob.x - PLAYER_SIZE) >> TILESHIFT;
    let mut yl = (ob.y - PLAYER_SIZE) >> TILESHIFT;
    let mut xh = (ob.x + PLAYER_SIZE) >> TILESHIFT;
    let mut yh = (ob.y + PLAYER_SIZE) >> TILESHIFT;

    // check for solid walls
    for y in yl..=yh {
        for x in xl..=xh {
            if let At::Wall(_) = level_state.actor_at[x as usize][y as usize] {
                return false;
            }
        }
    }

    // check for actors

    if yl > 0 {
        yl -= 1;
    }
    if yh < (MAP_SIZE - 1) as i32 {
        yh += 1;
    }
    if xl > 0 {
        xl -= 1;
    }
    if xh < (MAP_SIZE - 1) as i32 {
        xh += 1;
    }

    for y in yl..=yh {
        for x in xl..=xh {
            if let At::Obj(k) = level_state.actor_at[x as usize][y as usize] {
                let check = level_state.obj(k);
                if (check.flags & FL_SHOOTABLE) != 0 {
                    let delta_x = ob.x - check.x;
                    if delta_x < -MIN_ACTOR_DIST || delta_x > MIN_ACTOR_DIST {
                        continue;
                    }
                    let delta_y = ob.y - check.y;
                    if delta_y < -MIN_ACTOR_DIST || delta_y > MIN_ACTOR_DIST {
                        continue;
                    }

                    return false;
                }
            }
        }
    }

    return true;
}

fn set_move(k: ObjKey, level_state: &mut LevelState, dx: i32, dy: i32) {
    let obj = level_state.mut_obj(k);
    obj.x = dx;
    obj.y = dy;
}

pub fn draw_health(state: &GameState, rdr: &VGARenderer) {
    latch_number(rdr, 21, 16, 3, state.health);
}

pub fn draw_lives(state: &GameState, rdr: &VGARenderer) {
    latch_number(rdr, 14, 16, 1, state.lives);
}

pub fn draw_level(state: &GameState, rdr: &VGARenderer) {
    latch_number(rdr, 2, 16, 2, state.map_on as i32 + 1);
}

pub fn draw_ammo(state: &GameState, rdr: &VGARenderer) {
    latch_number(rdr, 27, 16, 2, state.ammo);
}

pub fn draw_face(state: &GameState, rdr: &VGARenderer) {
    if state.health > 0 {
        status_draw_pic(
            rdr,
            17,
            4,
            face_pic(3 * ((100 - state.health as usize) / 16) + state.face_frame),
        );
    } else {
        // TODO draw mutant face if last attack was needleobj
        status_draw_pic(rdr, 17, 4, GraphicNum::FACE8APIC)
    }
}

pub fn draw_fps(rdr: &VGARenderer, fps_str: &str) {
    let font = &rdr.fonts[1];

    let offset_prev = rdr.buffer_offset();
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        draw_string(rdr, font, fps_str, 0, 0, 5);
    }
    rdr.set_buffer_offset(offset_prev);
}

/// Calls draw face if time to change
fn update_face(tics: u64, state: &mut GameState, sound: &mut Sound, rdr: &VGARenderer) {
    if sound.is_sound_playing(SoundName::GETGATLING) {
        return;
    }

    state.face_count += tics;
    if state.face_count > rnd_t() as u64 {
        state.face_frame = rnd_t() as usize >> 6;
        if state.face_frame == 3 {
            state.face_frame = 1;
        }
        state.face_count = 0;
        draw_face(state, rdr);
    }
}

pub fn draw_keys(state: &GameState, rdr: &VGARenderer) {
    if state.keys & 1 != 0 {
        status_draw_pic(rdr, 30, 4, GraphicNum::GOLDKEYPIC);
    } else {
        status_draw_pic(rdr, 30, 4, GraphicNum::NOKEYPIC)
    }

    if state.keys & 2 != 0 {
        status_draw_pic(rdr, 30, 20, GraphicNum::SILVERKEYPIC);
    } else {
        status_draw_pic(rdr, 30, 20, GraphicNum::NOKEYPIC);
    }
}

pub fn draw_weapon(state: &GameState, rdr: &VGARenderer) {
    status_draw_pic(rdr, 32, 8, weapon_pic(state.weapon))
}

pub fn draw_score(state: &GameState, rdr: &VGARenderer) {
    latch_number(rdr, 6, 16, 6, state.score as i32);
}

fn latch_number(rdr: &VGARenderer, x_start: usize, y: usize, width: usize, num: i32) {
    let str = num.to_string();
    let mut w_cnt = width;
    let mut x = x_start;
    while str.len() < w_cnt {
        status_draw_pic(rdr, x, y, GraphicNum::NBLANKPIC);
        x += 1;
        w_cnt -= 1;
    }

    let mut c = if str.len() <= w_cnt {
        0
    } else {
        str.len() - w_cnt
    };
    let mut chars = str.chars();
    while c < str.len() {
        let ch = chars.next().unwrap();
        status_draw_pic(rdr, x, y, n_pic(ch.to_digit(10).unwrap() as usize));
        x += 1;
        c += 1;
    }
}

/// x in bytes
fn status_draw_pic(rdr: &VGARenderer, x: usize, y: usize, pic: GraphicNum) {
    let offset_prev = rdr.buffer_offset();
    for i in 0..3 {
        rdr.set_buffer_offset(SCREENLOC[i]);
        let y_status = (200 - STATUS_LINES) + y;
        rdr.pic(x * 8, y_status, pic);
    }
    rdr.set_buffer_offset(offset_prev);
}
