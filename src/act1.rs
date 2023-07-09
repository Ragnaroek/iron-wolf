use crate::def::{DoorType, DoorAction, LevelState, At};

const OPENTICS : u32 = 300;

pub fn spawn_door(tile_map: &mut Vec<Vec<u16>>, doornum: u16, tile_x: usize, tile_y: usize, vertical: bool, lock: u16) -> DoorType {
    //TODO why is the original map manipulated in the original code?

    if doornum == 64 {
        panic!("64+ doors on level!") //TODO replace with Quit
    }

    tile_map[tile_x][tile_y] = doornum | 0x80;
    if vertical {
        tile_map[tile_x][tile_y-1] |= 0x40;
        tile_map[tile_x][tile_y+1] |= 0x40;
    }    else {
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

fn open_door(door: &mut DoorType) {
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