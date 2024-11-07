use crate::{
    assets::SoundName,
    def::{
        Assets, At, Dir, DoorAction, DoorLock, DoorType, GameState, LevelState, Sprite, StaticInfo,
        StaticKind, StaticType, FL_BONUS, MAP_SIZE, MAX_STATS, MIN_DIST, NUM_AREAS, TILESHIFT,
    },
    game::AREATILE,
    map::MapSegs,
    sd::Sound,
};

const OPENTICS: u32 = 300;
const NUM_STAT_INFO: usize = 49;

/*
=============================================================================

                            STATICS

=============================================================================
*/

static STAT_INFO: [StaticInfo; NUM_STAT_INFO] = [
    StaticInfo {
        sprite: Sprite::Stat0,
        kind: StaticKind::Dressing,
    }, // puddle          spr1v
    StaticInfo {
        sprite: Sprite::Stat1,
        kind: StaticKind::Block,
    }, // Green Barrel    "
    StaticInfo {
        sprite: Sprite::Stat2,
        kind: StaticKind::Block,
    }, // Table/chairs    "
    StaticInfo {
        sprite: Sprite::Stat3,
        kind: StaticKind::Block,
    }, // Floor lamp      "
    StaticInfo {
        sprite: Sprite::Stat4,
        kind: StaticKind::Dressing,
    }, // Chandelier      "
    StaticInfo {
        sprite: Sprite::Stat5,
        kind: StaticKind::Block,
    }, // Hanged man      "
    StaticInfo {
        sprite: Sprite::Stat6,
        kind: StaticKind::BoAlpo,
    }, // Bad food        "
    StaticInfo {
        sprite: Sprite::Stat7,
        kind: StaticKind::Block,
    }, // Red pillar      "
    StaticInfo {
        sprite: Sprite::Stat8,
        kind: StaticKind::Block,
    }, // Tree            spr2v
    StaticInfo {
        sprite: Sprite::Stat9,
        kind: StaticKind::Dressing,
    }, // Skeleton flat   "
    StaticInfo {
        sprite: Sprite::Stat10,
        kind: StaticKind::Block,
    }, // Sink            " (SOD:gibs)
    StaticInfo {
        sprite: Sprite::Stat11,
        kind: StaticKind::Block,
    }, // Potted plant    "
    StaticInfo {
        sprite: Sprite::Stat12,
        kind: StaticKind::Block,
    }, // Urn             "
    StaticInfo {
        sprite: Sprite::Stat13,
        kind: StaticKind::Block,
    }, // Bare table      "
    StaticInfo {
        sprite: Sprite::Stat14,
        kind: StaticKind::Dressing,
    }, // Ceiling light   "
    StaticInfo {
        sprite: Sprite::Stat15,
        kind: StaticKind::Dressing,
    }, // Kitchen stuff   "
    StaticInfo {
        sprite: Sprite::Stat16,
        kind: StaticKind::Block,
    }, // suit of armor   spr3v
    StaticInfo {
        sprite: Sprite::Stat17,
        kind: StaticKind::Block,
    }, // Hanging cage    "
    StaticInfo {
        sprite: Sprite::Stat18,
        kind: StaticKind::Block,
    }, // SkeletoninCage  "
    StaticInfo {
        sprite: Sprite::Stat19,
        kind: StaticKind::Dressing,
    }, // Skeleton relax  "
    StaticInfo {
        sprite: Sprite::Stat20,
        kind: StaticKind::BoKey1,
    }, // Key 1           "
    StaticInfo {
        sprite: Sprite::Stat21,
        kind: StaticKind::BoKey2,
    }, // Key 2           "
    StaticInfo {
        sprite: Sprite::Stat22,
        kind: StaticKind::Block,
    }, // stuff				(SOD:gibs)
    StaticInfo {
        sprite: Sprite::Stat23,
        kind: StaticKind::Dressing,
    }, // stuff
    StaticInfo {
        sprite: Sprite::Stat24,
        kind: StaticKind::BoFood,
    }, // Good food       spr4v
    StaticInfo {
        sprite: Sprite::Stat25,
        kind: StaticKind::BoFirstaid,
    }, // First aid       "
    StaticInfo {
        sprite: Sprite::Stat26,
        kind: StaticKind::BoClip,
    }, // Clip            "
    StaticInfo {
        sprite: Sprite::Stat27,
        kind: StaticKind::BoMachinegun,
    }, // Machine gun     "
    StaticInfo {
        sprite: Sprite::Stat28,
        kind: StaticKind::BoChaingun,
    }, // Gatling gun     "
    StaticInfo {
        sprite: Sprite::Stat29,
        kind: StaticKind::BoCross,
    }, // Cross           "
    StaticInfo {
        sprite: Sprite::Stat30,
        kind: StaticKind::BoChalice,
    }, // Chalice         "
    StaticInfo {
        sprite: Sprite::Stat31,
        kind: StaticKind::BoBible,
    }, // Bible           "
    StaticInfo {
        sprite: Sprite::Stat33,
        kind: StaticKind::BoCrown,
    }, // crown           spr5v
    StaticInfo {
        sprite: Sprite::Stat33,
        kind: StaticKind::BoFullheal,
    }, // one up          "
    StaticInfo {
        sprite: Sprite::Stat34,
        kind: StaticKind::BoGibs,
    }, // gibs            "
    StaticInfo {
        sprite: Sprite::Stat35,
        kind: StaticKind::Block,
    }, // barrel          "
    StaticInfo {
        sprite: Sprite::Stat36,
        kind: StaticKind::Block,
    }, // well            "
    StaticInfo {
        sprite: Sprite::Stat37,
        kind: StaticKind::Block,
    }, // Empty well      "
    StaticInfo {
        sprite: Sprite::Stat38,
        kind: StaticKind::BoGibs,
    }, // Gibs 2          "
    StaticInfo {
        sprite: Sprite::Stat39,
        kind: StaticKind::Block,
    }, // flag				"
    StaticInfo {
        sprite: Sprite::Stat40,
        kind: StaticKind::Block,
    }, // Call Apogee		spr7v
    StaticInfo {
        sprite: Sprite::Stat41,
        kind: StaticKind::Dressing,
    }, // junk            "		"
    StaticInfo {
        sprite: Sprite::Stat42,
        kind: StaticKind::Dressing,
    }, // junk            "
    StaticInfo {
        sprite: Sprite::Stat43,
        kind: StaticKind::Dressing,
    }, // junk            "
    StaticInfo {
        sprite: Sprite::Stat44,
        kind: StaticKind::Block,
    }, // pots            "
    StaticInfo {
        sprite: Sprite::Stat45,
        kind: StaticKind::Block,
    }, // stove           " (SOD:gibs)
    StaticInfo {
        sprite: Sprite::Stat46,
        kind: StaticKind::Block,
    }, // spears          " (SOD:gibs)
    StaticInfo {
        sprite: Sprite::Stat47,
        kind: StaticKind::Dressing,
    }, // vines			"
    StaticInfo {
        sprite: Sprite::Stat26,
        kind: StaticKind::BoClip2,
    }, // Clip            "
];

pub fn spawn_static(
    actor_at: &mut Vec<Vec<At>>,
    game_state: &mut GameState,
    tile_x: usize,
    tile_y: usize,
    stat_type: usize,
) -> StaticType {
    let info = &STAT_INFO[stat_type];

    let mut flags = 0;
    if info.kind == StaticKind::Block {
        actor_at[tile_x][tile_y] = At::Wall(1); // Blocked
    } else if info.kind == StaticKind::Dressing {
        flags = 0;
    } else {
        if info.kind == StaticKind::BoCross
            || info.kind == StaticKind::BoChalice
            || info.kind == StaticKind::BoBible
            || info.kind == StaticKind::BoCrown
            || info.kind == StaticKind::BoFullheal
        {
            if !game_state.loaded_game {
                game_state.treasure_total += 1;
            }
        }
        flags = FL_BONUS;
    }
    StaticType {
        tile_x,
        tile_y,
        sprite: info.sprite,
        flags,
        item_number: info.kind,
    }
}

/// Called during game play to drop actors' items.  It finds the proper
/// item number based on the item type (bo_???).  If there are no free item
/// spots, nothing is done.
pub fn place_item_type(
    level_state: &mut LevelState,
    item_type: StaticKind,
    tile_x: usize,
    tile_y: usize,
) {
    let mut found_info = None;
    for info in &STAT_INFO {
        if info.kind == item_type {
            found_info = Some(info);
            break;
        }
    }

    if level_state.statics.len() >= MAX_STATS {
        return; // no free spots anymore
    }

    if let Some(info) = found_info {
        level_state.statics.push(StaticType {
            tile_x,
            tile_y,
            sprite: info.sprite,
            flags: FL_BONUS,
            item_number: info.kind,
        });
    } else {
        panic!("PlaceItemType: couldn't find type!");
    }
}

/*
=============================================================================

                        DOORS

=============================================================================
*/

// Scans outward from playerarea, marking all connected areas
fn connect_areas(level_state: &mut LevelState) {
    level_state.area_by_player.fill(false);
    let area_num = level_state.player().area_number as usize;
    level_state.area_by_player[area_num] = true;
    recursive_connect(level_state, area_num);
}

fn recursive_connect(level_state: &mut LevelState, area_num: usize) {
    for i in 0..NUM_AREAS {
        if level_state.area_connect[area_num][i] != 0 && !level_state.area_by_player[i] {
            level_state.area_by_player[i] = true;
            recursive_connect(level_state, i);
        }
    }
}

pub fn spawn_door(
    tile_map: &mut Vec<Vec<u16>>,
    map_segs: &mut MapSegs,
    doornum: usize,
    tile_x: usize,
    tile_y: usize,
    vertical: bool,
    lock: DoorLock,
) -> DoorType {
    if doornum == 64 {
        panic!("64+ doors on level!") //TODO replace with Quit
    }

    tile_map[tile_x][tile_y] = (doornum | 0x80) as u16;
    let map_lookup = tile_y * MAP_SIZE + tile_x;
    if vertical {
        map_segs.segs[0][map_lookup] = map_segs.segs[0][map_lookup - 1]; // set area number
        tile_map[tile_x][tile_y - 1] |= 0x40;
        tile_map[tile_x][tile_y + 1] |= 0x40;
    } else {
        map_segs.segs[0][map_lookup] = map_segs.segs[0][map_lookup - MAP_SIZE]; // set area number
        tile_map[tile_x - 1][tile_y] |= 0x40;
        tile_map[tile_x + 1][tile_y] |= 0x40;
    }

    DoorType {
        num: doornum,
        tile_x,
        tile_y,
        vertical,
        lock,
        action: DoorAction::Closed,
        tic_count: 0,
        position: 0, /* start out fully closed */
    }
}

pub fn operate_door(
    doornum: usize,
    level_state: &mut LevelState,
    sound: &mut Sound,
    assets: &Assets,
) {
    // TODO handle locked door here (check for keys, play sound)
    let door: &mut DoorType = &mut level_state.doors[doornum];
    match door.action {
        DoorAction::Closed | DoorAction::Closing => open_door(door),
        DoorAction::Open | DoorAction::Opening => close_door(doornum, level_state, sound, assets),
    }
}

pub fn open_door(door: &mut DoorType) {
    if door.action == DoorAction::Open {
        door.tic_count = 0; // reset open time
    } else {
        door.action = DoorAction::Opening;
    }
}

fn close_door(doornum: usize, level_state: &mut LevelState, sound: &mut Sound, assets: &Assets) {
    // don't close on anything solid
    let (tile_x, tile_y) = {
        let door = &level_state.doors[doornum as usize];
        (door.tile_x, door.tile_y)
    };

    if level_state.actor_at[tile_x][tile_y] != At::Nothing {
        return;
    }

    let p_tile_x = level_state.player().tilex;
    let p_tile_y = level_state.player().tiley;
    let p_x = level_state.player().x;
    let p_y = level_state.player().y;

    if p_tile_x == tile_x && p_tile_y == tile_y {
        return;
    }

    let door = &level_state.doors[doornum];
    if door.vertical {
        if p_tile_y == tile_y {
            if (p_x + MIN_DIST >> TILESHIFT) as usize == tile_x {
                return;
            }
            if (p_y - MIN_DIST >> TILESHIFT) as usize == tile_x {
                return;
            }
        }
        let check = level_state.actor_at[tile_x - 1][tile_y];
        if let At::Obj(k) = check {
            if (level_state.obj(k).x + MIN_DIST >> TILESHIFT) as usize == tile_x {
                return;
            }
        }
        let check = level_state.actor_at[tile_x + 1][tile_y];
        if let At::Obj(k) = check {
            if (level_state.obj(k).x - MIN_DIST >> TILESHIFT) as usize == tile_x {
                return;
            }
        }
    } else {
        if p_tile_x == tile_x {
            if (p_y + MIN_DIST >> TILESHIFT) as usize == tile_y {
                return;
            }
            if (p_y - MIN_DIST >> TILESHIFT) as usize == tile_y {
                return;
            }
        }
        let check = level_state.actor_at[tile_x][tile_y - 1];
        if let At::Obj(k) = check {
            if (level_state.obj(k).y + MIN_DIST >> TILESHIFT) as usize == tile_y {
                return;
            }
        }
        let check = level_state.actor_at[tile_x][tile_y + 1];
        if let At::Obj(k) = check {
            if (level_state.obj(k).y - MIN_DIST >> TILESHIFT) as usize == tile_y {
                return;
            }
        }
    }

    let door = &mut level_state.doors[doornum];

    let area = (level_state.level.map_segs.segs[0][door.tile_y * MAP_SIZE + door.tile_x] - AREATILE)
        as usize;
    if level_state.area_by_player[area] {
        sound.play_sound_loc_tile(SoundName::CLOSEDOOR, assets, door.tile_x, door.tile_y);
    }

    door.action = DoorAction::Closing;
    level_state.actor_at[tile_x][tile_y] = At::Wall((door.num | 0x80) as u16);
}

// called from play_loop
pub fn move_doors(level_state: &mut LevelState, sound: &mut Sound, assets: &Assets, tics: u64) {
    for doornum in 0..level_state.doors.len() {
        match level_state.doors[doornum].action {
            DoorAction::Open => door_open(doornum, level_state, sound, assets, tics),
            DoorAction::Opening => door_opening(doornum, level_state, sound, assets, tics),
            DoorAction::Closing => door_closing(doornum, level_state, tics),
            DoorAction::Closed => { /* do nothing here */ }
        }
    }
}

fn door_open(
    doornum: usize,
    level_state: &mut LevelState,
    sound: &mut Sound,
    assets: &Assets,
    tics: u64,
) {
    level_state.doors[doornum as usize].tic_count += tics as u32;

    if level_state.doors[doornum as usize].tic_count >= OPENTICS {
        close_door(doornum, level_state, sound, assets);
    }
}

fn door_opening(
    doornum: usize,
    level_state: &mut LevelState,
    sound: &mut Sound,
    assets: &Assets,
    tics: u64,
) {
    let door = &level_state.doors[doornum as usize];
    let mut position = door.position as u64;
    if position == 0 {
        // door is just starting to open, so connect the areas
        let (area1, area2) = if door.vertical {
            vert_door_areas(level_state, door.tile_x, door.tile_y)
        } else {
            horiz_door_areas(level_state, door.tile_x, door.tile_y)
        };

        level_state.area_connect[area1][area2] += 1;
        level_state.area_connect[area2][area1] += 1;

        connect_areas(level_state);

        if level_state.area_by_player[area1] {
            let door = &level_state.doors[doornum as usize];
            sound.play_sound_loc_tile(SoundName::OPENDOOR, assets, door.tile_x, door.tile_y);
        }
    }

    let door = &mut level_state.doors[doornum as usize];

    // slide the door by an adaptive amount
    position += tics << 10;
    if position >= 0xFFFF {
        position = 0xFFFF;
        door.tic_count = 0;
        door.action = DoorAction::Open;
        level_state.actor_at[door.tile_x][door.tile_y] = At::Nothing;
    }

    door.position = position as u16;
}

fn door_closing(doornum: usize, level_state: &mut LevelState, tics: u64) {
    let p_tile_x = level_state.player().tilex;
    let p_tile_y = level_state.player().tiley;
    {
        let door = &mut level_state.doors[doornum as usize];
        if let At::Obj(_) = level_state.actor_at[door.tile_x][door.tile_y] {
            // something got inside the door
            open_door(door);
            return;
        }

        if p_tile_x == door.tile_x && p_tile_y == door.tile_y {
            // player got inside the door
            open_door(door);
            return;
        }
    }
    let door = &level_state.doors[doornum as usize];
    let mut position = door.position as u64;
    // slide the door by an adaptive amount
    position = position.saturating_sub(tics << 10);
    if position == 0 {
        // door is closed all the way, so disconnect the areas
        let (area1, area2) = if door.vertical {
            vert_door_areas(level_state, door.tile_x, door.tile_y)
        } else {
            horiz_door_areas(level_state, door.tile_x, door.tile_y)
        };

        level_state.area_connect[area1][area2] -= 1;
        level_state.area_connect[area2][area1] -= 1;

        connect_areas(level_state);

        level_state.doors[doornum as usize].action = DoorAction::Closed;
    }
    level_state.doors[doornum as usize].position = position as u16;
}

// extract the area information on a vertical door (from the horizontal connected door tiles)
fn vert_door_areas(level_state: &LevelState, tile_x: usize, tile_y: usize) -> (usize, usize) {
    let area1 = level_state.level.map_segs.segs[0][tile_y * MAP_SIZE + (tile_x - 1)] as usize;
    let area2 = level_state.level.map_segs.segs[0][tile_y * MAP_SIZE + (tile_x + 1)] as usize;
    (area1 - AREATILE as usize, area2 - AREATILE as usize)
}

// extract the area information on a horizontal door (from the vertical connected door tiles)
fn horiz_door_areas(level_state: &LevelState, tile_x: usize, tile_y: usize) -> (usize, usize) {
    let area1 = level_state.level.map_segs.segs[0][(tile_y - 1) * MAP_SIZE + tile_x] as usize;
    let area2 = level_state.level.map_segs.segs[0][(tile_y + 1) * MAP_SIZE + tile_x] as usize;
    (area1 - AREATILE as usize, area2 - AREATILE as usize)
}

/*
=============================================================================

                        PUSHABLE WALLS

=============================================================================
*/

pub fn push_wall(
    level_state: &mut LevelState,
    game_state: &mut GameState,
    check_x: usize,
    check_y: usize,
    dir: Dir,
) {
    if game_state.push_wall_state != 0 {
        return;
    }

    let old_tile = level_state.level.tile_map[check_x][check_y];
    if old_tile == 0 {
        return;
    }

    match dir {
        Dir::North => {
            if level_state.actor_at[check_x][check_y - 1] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x][check_y - 1] = At::Wall(old_tile);
            level_state.level.tile_map[check_x][check_y - 1] = old_tile;
        }
        Dir::East => {
            if level_state.actor_at[check_x + 1][check_y] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x + 1][check_y] = At::Wall(old_tile);
            level_state.level.tile_map[check_x + 1][check_y] = old_tile;
        }
        Dir::South => {
            if level_state.actor_at[check_x][check_y + 1] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x][check_y + 1] = At::Wall(old_tile);
            level_state.level.tile_map[check_x][check_y + 1] = old_tile;
        }
        Dir::West => {
            if level_state.actor_at[check_x - 1][check_y] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x - 1][check_y] = At::Wall(old_tile);
            level_state.level.tile_map[check_x - 1][check_y] = old_tile;
        }
    }

    game_state.secret_count += 1;
    game_state.push_wall_x = check_x;
    game_state.push_wall_y = check_y;
    game_state.push_wall_dir = dir;
    game_state.push_wall_state = 1;
    game_state.push_wall_pos = 0;
    level_state.level.tile_map[check_x][check_y] |= 0xC0;
    level_state.level.info_map[check_x][check_y] = 0; // remove P tile info
                                                      //TODO SD_PlaySound(PUSHWALLSND)
}

pub fn move_push_walls(level_state: &mut LevelState, game_state: &mut GameState, tics: u64) {
    if game_state.push_wall_state == 0 {
        return;
    }

    let old_block = game_state.push_wall_state / 128;
    game_state.push_wall_state += tics;

    if game_state.push_wall_state / 128 != old_block {
        // block crossed into a new block
        let old_tile =
            level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] & 63;
        // the tile can now be walked into
        level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] = 0;
        level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y] = At::Nothing;
        level_state.level.map_segs.segs[0]
            [game_state.push_wall_y * MAP_SIZE + game_state.push_wall_x] =
            level_state.player().area_number as u16 + AREATILE; // fixup area to make it walkable

        // see if it should be pushed farther
        if game_state.push_wall_state > 256 {
            // the block has been pushed two tiles
            game_state.push_wall_state = 0;
            return;
        } else {
            match game_state.push_wall_dir {
                Dir::North => {
                    game_state.push_wall_y -= 1;
                    if level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y - 1]
                        != At::Nothing
                    {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y - 1] =
                        At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x]
                        [game_state.push_wall_y - 1] = old_tile;
                }
                Dir::East => {
                    game_state.push_wall_x += 1;
                    if level_state.actor_at[game_state.push_wall_x + 1][game_state.push_wall_y]
                        != At::Nothing
                    {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x + 1][game_state.push_wall_y] =
                        At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x + 1]
                        [game_state.push_wall_y] = old_tile;
                }
                Dir::South => {
                    game_state.push_wall_y += 1;
                    if level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y + 1]
                        != At::Nothing
                    {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y + 1] =
                        At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x]
                        [game_state.push_wall_y + 1] = old_tile;
                }
                Dir::West => {
                    game_state.push_wall_x -= 1;
                    if level_state.actor_at[game_state.push_wall_x - 1][game_state.push_wall_y]
                        != At::Nothing
                    {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x - 1][game_state.push_wall_y] =
                        At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x - 1]
                        [game_state.push_wall_y] = old_tile;
                }
            }
            level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] =
                old_tile | 0xC0;
        }
    }
    game_state.push_wall_pos = ((game_state.push_wall_state / 2) & 63) as i32;
}
