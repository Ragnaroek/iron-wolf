
pub fn spawn_door(tile_map: &mut Vec<Vec<u16>>, doornum: u16, tile_x: usize, tile_y: usize, vertical: bool, lock: u16) {
    
    //TODO create doorobj (and return it?)
    //TODO why is the original map manipulated in the original code?
    
    tile_map[tile_x][tile_y] = doornum | 0x80;
    if vertical {
        tile_map[tile_x][tile_y-1] |= 0x40;
        tile_map[tile_x][tile_y+1] |= 0x40;
    }    else {
        tile_map[tile_x-1][tile_y] |= 0x40;
        tile_map[tile_x+1][tile_y] |= 0x40;
    }
}