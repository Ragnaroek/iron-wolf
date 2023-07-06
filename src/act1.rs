use crate::def::{DoorType, DoorAction, LevelState};

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

    DoorType { tile_x, tile_y, vertical, lock, action: DoorAction::Closed, tic_count: 0 }
}

pub fn operate_door(doornum: u16, level_state: &mut LevelState) {
    
    // TODO handle locked door here (check for keys, play sound)

    let door = &mut level_state.doors[doornum as usize];
    match door.action {
        DoorAction::Closed | DoorAction::Closing => open_door(door),
        DoorAction::Open | DoorAction::Opening => close_door(door),
    }
}

fn open_door(door: &mut DoorType) {
    if door.action == DoorAction::Open {
        door.tic_count = 0; // reset open time
    } else {
        door.action = DoorAction::Opening;
    }
}

fn close_door(door: &mut DoorType) {

}