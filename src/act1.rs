use crate::{def::{DoorType, StaticType, StaticKind, StaticInfo, Sprite, DoorAction, LevelState, At, FL_BONUS, MAX_STATS, GameState, Dir}, game};

const OPENTICS : u32 = 300;
const NUM_STAT_INFO : usize = 49;

/*
=============================================================================

							STATICS

=============================================================================
*/

static STAT_INFO : [StaticInfo; NUM_STAT_INFO] = [
    StaticInfo{sprite: Sprite::Stat0, kind: StaticKind::Dressing}, // puddle          spr1v
    StaticInfo{sprite: Sprite::Stat1, kind: StaticKind::Block},    // Green Barrel    "
    StaticInfo{sprite: Sprite::Stat2, kind: StaticKind::Block},    // Table/chairs    "
    StaticInfo{sprite: Sprite::Stat3, kind: StaticKind::Block},    // Floor lamp      "
    StaticInfo{sprite: Sprite::Stat4, kind: StaticKind::Dressing}, // Chandelier      "
    StaticInfo{sprite: Sprite::Stat5, kind: StaticKind::Block},    // Hanged man      "
    StaticInfo{sprite: Sprite::Stat6, kind: StaticKind::BoAlpo},   // Bad food        "
    StaticInfo{sprite: Sprite::Stat7, kind: StaticKind::Block},    // Red pillar      "
    
    StaticInfo{sprite: Sprite::Stat8, kind: StaticKind::Block},    // Tree            spr2v
    StaticInfo{sprite: Sprite::Stat9, kind: StaticKind::Dressing}, // Skeleton flat   "
    StaticInfo{sprite: Sprite::Stat10, kind: StaticKind::Block},   // Sink            " (SOD:gibs)
    StaticInfo{sprite: Sprite::Stat11, kind: StaticKind::Block},   // Potted plant    "
    StaticInfo{sprite: Sprite::Stat12, kind: StaticKind::Block},   // Urn             "
    StaticInfo{sprite: Sprite::Stat13, kind: StaticKind::Block},   // Bare table      "
    StaticInfo{sprite: Sprite::Stat14, kind: StaticKind::Dressing},// Ceiling light   "
    StaticInfo{sprite: Sprite::Stat15, kind: StaticKind::Dressing},// Kitchen stuff   "

    StaticInfo{sprite: Sprite::Stat16, kind: StaticKind::Block},   // suit of armor   spr3v
    StaticInfo{sprite: Sprite::Stat17, kind: StaticKind::Block},   // Hanging cage    "
    StaticInfo{sprite: Sprite::Stat18, kind: StaticKind::Block},   // SkeletoninCage  "
    StaticInfo{sprite: Sprite::Stat19, kind: StaticKind::Dressing},// Skeleton relax  "
    StaticInfo{sprite: Sprite::Stat20, kind: StaticKind::BoKey1},  // Key 1           "
    StaticInfo{sprite: Sprite::Stat21, kind: StaticKind::BoKey2},  // Key 2           "
    StaticInfo{sprite: Sprite::Stat22, kind: StaticKind::Block},   // stuff				(SOD:gibs)
    StaticInfo{sprite: Sprite::Stat23, kind: StaticKind::Dressing},// stuff

    StaticInfo{sprite: Sprite::Stat24, kind: StaticKind::BoFood},          // Good food       spr4v
    StaticInfo{sprite: Sprite::Stat25, kind: StaticKind::BoFirstaid},      // First aid       "
    StaticInfo{sprite: Sprite::Stat26, kind: StaticKind::BoClip},          // Clip            "
    StaticInfo{sprite: Sprite::Stat27, kind: StaticKind::BoMachinegun},    // Machine gun     "
    StaticInfo{sprite: Sprite::Stat28, kind: StaticKind::BoChaingun},      // Gatling gun     "
    StaticInfo{sprite: Sprite::Stat29, kind: StaticKind::BoCross},         // Cross           "
    StaticInfo{sprite: Sprite::Stat30, kind: StaticKind::BoChalice},       // Chalice         "
    StaticInfo{sprite: Sprite::Stat31, kind: StaticKind::BoBible},         // Bible           "

    StaticInfo{sprite: Sprite::Stat33, kind: StaticKind::BoCrown},         // crown           spr5v
    StaticInfo{sprite: Sprite::Stat33, kind: StaticKind::BoFullheal},      // one up          "
    StaticInfo{sprite: Sprite::Stat34, kind: StaticKind::BoGibs},          // gibs            "
    StaticInfo{sprite: Sprite::Stat35, kind: StaticKind::Block},           // barrel          "
    StaticInfo{sprite: Sprite::Stat36, kind: StaticKind::Block},           // well            "
    StaticInfo{sprite: Sprite::Stat37, kind: StaticKind::Block},           // Empty well      "
    StaticInfo{sprite: Sprite::Stat38, kind: StaticKind::BoGibs},          // Gibs 2          "
    StaticInfo{sprite: Sprite::Stat39, kind: StaticKind::Block},           // flag				"

    StaticInfo{sprite: Sprite::Stat40, kind: StaticKind::Block},           // Call Apogee		spr7v
    StaticInfo{sprite: Sprite::Stat41, kind: StaticKind::Dressing},        // junk            "		"
    StaticInfo{sprite: Sprite::Stat42, kind: StaticKind::Dressing},        // junk            "
    StaticInfo{sprite: Sprite::Stat43, kind: StaticKind::Dressing},        // junk            "
    StaticInfo{sprite: Sprite::Stat44, kind: StaticKind::Block},           // pots            "
    StaticInfo{sprite: Sprite::Stat45, kind: StaticKind::Block},           // stove           " (SOD:gibs)
    StaticInfo{sprite: Sprite::Stat46, kind: StaticKind::Block},           // spears          " (SOD:gibs)
    StaticInfo{sprite: Sprite::Stat47, kind: StaticKind::Dressing},        // vines			"
    StaticInfo{sprite: Sprite::Stat26, kind: StaticKind::BoClip2},         // Clip            "
]; 

pub fn spawn_static(actor_at: &mut Vec<Vec<At>>, game_state: &mut GameState, tile_x: usize, tile_y: usize, stat_type: usize) -> StaticType {
    let info = &STAT_INFO[stat_type];

    let mut flags = 0;
    if info.kind == StaticKind::Block {
        actor_at[tile_x][tile_y] = At::Blocked 
    } else if info.kind == StaticKind::Dressing {
        flags = 0;
    } else {
        if info.kind == StaticKind::BoCross || info.kind == StaticKind::BoChalice || info.kind == StaticKind::BoBible || info.kind == StaticKind::BoCrown || info.kind == StaticKind::BoFullheal {
            // TODO check loaded game from iw-ed?
            game_state.treasure_total += 1;
        }
        flags = FL_BONUS;
    }
    StaticType{
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
pub fn place_item_type(level_state: &mut LevelState, item_type: StaticKind, tile_x: usize, tile_y: usize) {
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

pub fn spawn_door(tile_map: &mut Vec<Vec<u16>>, doornum: u16, tile_x: usize, tile_y: usize, vertical: bool, lock: u16) -> DoorType {
    if doornum == 64 {
        panic!("64+ doors on level!") //TODO replace with Quit
    }

    tile_map[tile_x][tile_y] = doornum | 0x80;
    if vertical {
        tile_map[tile_x][tile_y-1] |= 0x40;
        tile_map[tile_x][tile_y+1] |= 0x40;
    } else {
        tile_map[tile_x-1][tile_y] |= 0x40;
        tile_map[tile_x+1][tile_y] |= 0x40;
    }

    DoorType { num: doornum, tile_x, tile_y, vertical, lock, action: DoorAction::Closed, tic_count: 0, position: 0 /* start out fully closed */ }
}

pub fn operate_door(doornum: u16, level_state: &mut LevelState) {
    
    // TODO handle locked door here (check for keys, play sound)

    let door = &mut level_state.doors[doornum as usize];
    match door.action {
        DoorAction::Closed | DoorAction::Closing => open_door(door),
        DoorAction::Open | DoorAction::Opening => close_door(door, &mut level_state.actor_at),
    }
}

pub fn open_door(door: &mut DoorType) {
    if door.action == DoorAction::Open {
        door.tic_count = 0; // reset open time
    } else {
        door.action = DoorAction::Opening;
    }
}

fn close_door(door: &mut DoorType, actor_at: &mut Vec<Vec<At>>) {
    // TODO check if anything solid (player, actorat) gets stuck in door (again?)
    // TODO play door sound

    door.action = DoorAction::Closing;
    actor_at[door.tile_x][door.tile_y] = At::Wall(door.num | 0x80);
}

// called from play_loop
pub fn move_doors(level_state: &mut LevelState, tics: u64) {
    for door in &mut level_state.doors {
        match door.action {
            DoorAction::Open => door_open(door, &mut level_state.actor_at, tics),
            DoorAction::Opening => door_opening(door, &mut level_state.actor_at, tics),
            DoorAction::Closing => door_closing(door, tics),
            DoorAction::Closed => {/* do nothing here */},
        }       
    } 
}

fn door_open(door: &mut DoorType, actor_at: &mut Vec<Vec<At>>, tics: u64) {
    door.tic_count += tics as u32; //TODO XXX where to get tics from?

    if door.tic_count >= OPENTICS {
        close_door(door, actor_at);
    }
}

fn door_opening(door: &mut DoorType, actor_at: &mut Vec<Vec<At>>, tics: u64) {
    let mut position = door.position as u64;
    
    if position == 0 {
        // TODO connect areas if door just opened!
    }

    // slide the door by an adaptive amount
    position += tics << 10;
    if position >= 0xFFFF {
        position = 0xFFFF;
        door.tic_count = 0;
        door.action = DoorAction::Open;
        actor_at[door.tile_x][door.tile_y] = At::Nothing;
    }

    door.position = position as u16;
}

fn door_closing(door: &mut DoorType, tics: u64) {
    // TODO check if something gets stuck in the door
    let mut position = door.position as u64;
    position = position.saturating_sub(tics << 10);
    if position == 0 {
        // TODO disconnect areas
        door.action = DoorAction::Closed;
    }
    door.position = position as u16;
}

/*
=============================================================================

						PUSHABLE WALLS

=============================================================================
*/

pub fn push_wall(level_state: &mut LevelState, game_state: &mut GameState, check_x: usize, check_y: usize, dir: Dir) {
    if game_state.push_wall_state != 0 {
        return;
    }

    let old_tile = level_state.level.tile_map[check_x][check_y];
    if old_tile == 0 {
        return;
    }

    match dir {
        Dir::North => {
            if level_state.actor_at[check_x][check_y-1] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x][check_y-1] = At::Wall(old_tile);
            level_state.level.tile_map[check_x][check_y-1] = old_tile;
        },
        Dir::East => {
            if level_state.actor_at[check_x+1][check_y] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x+1][check_y] = At::Wall(old_tile);
            level_state.level.tile_map[check_x+1][check_y] = old_tile;
        },
        Dir::South => {
            if level_state.actor_at[check_x][check_y+1] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x][check_y+1] = At::Wall(old_tile);
            level_state.level.tile_map[check_x][check_y+1] = old_tile;
        },
        Dir::West => {
            if level_state.actor_at[check_x-1][check_y] != At::Nothing {
                // TODO SD_PlaySound(NOWAYSND)
                return;
            }
            level_state.actor_at[check_x-1][check_y] = At::Wall(old_tile);
            level_state.level.tile_map[check_x-1][check_y] = old_tile;
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

    let old_block = game_state.push_wall_state/128;
    game_state.push_wall_state += tics;

    if game_state.push_wall_state/128 != old_block {
        // block crossed into a new block
        let old_tile = level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] & 63;
        // the tile can now be walked into
        level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] = 0;
        level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y] = At::Nothing;
        // TODO mapsegs[0] is manipulated here with player->areanumber+AREATILE why??

        // see if it should be pushed farther
        if game_state.push_wall_state > 256 {
            // the block has been pushed two tiles
            game_state.push_wall_state = 0;
            return;
        } else {
            match game_state.push_wall_dir {
                Dir::North => {
                    game_state.push_wall_y -= 1;
                    if level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y-1] != At::Nothing {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y-1] = At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y-1] = old_tile;
                },
                Dir::East => {
                    game_state.push_wall_x += 1;
                    if level_state.actor_at[game_state.push_wall_x+1][game_state.push_wall_y] != At::Nothing {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x+1][game_state.push_wall_y] = At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x+1][game_state.push_wall_y] = old_tile;
                },
                Dir::South => {
                    game_state.push_wall_y += 1;
                    if level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y+1] != At::Nothing {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x][game_state.push_wall_y+1] = At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y+1] = old_tile;
                },
                Dir::West => {
                    game_state.push_wall_x -= 1;
                    if level_state.actor_at[game_state.push_wall_x-1][game_state.push_wall_y] != At::Nothing {
                        game_state.push_wall_state = 0;
                        return;
                    }
                    level_state.actor_at[game_state.push_wall_x-1][game_state.push_wall_y] = At::Wall(old_tile);
                    level_state.level.tile_map[game_state.push_wall_x-1][game_state.push_wall_y] = old_tile;
                },
            }
            level_state.level.tile_map[game_state.push_wall_x][game_state.push_wall_y] = old_tile | 0xC0;
        }
    }
    game_state.push_wall_pos = ((game_state.push_wall_state/2)&63) as i32;
}