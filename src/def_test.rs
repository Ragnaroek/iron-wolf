use crate::{agent::DUMMY_PLAYER, def::ObjKey};

use super::{Actors, MAX_ACTORS};

#[test]
fn test_actors() {
    let mut actors = Actors::new(MAX_ACTORS);
    assert_eq!(actors.len(), 0);
    actors.add_obj(DUMMY_PLAYER.clone()); //ix=0
    assert_eq!(actors.len(), 1);
    actors.add_obj(DUMMY_PLAYER.clone()); //ix=1
    assert_eq!(actors.len(), 2);
    actors.add_obj(DUMMY_PLAYER.clone()); //ix=2
    assert_eq!(actors.len(), 3);
    actors.put_obj(ObjKey(MAX_ACTORS - 1), DUMMY_PLAYER.clone());
    assert_eq!(actors.len(), MAX_ACTORS);

    actors.drop_obj(ObjKey(MAX_ACTORS - 1));
    assert_eq!(actors.len(), 3);

    // drop a whole into the sequence
    actors.drop_obj(ObjKey(1));
    assert_eq!(actors.len(), 3);

    actors.drop_obj(ObjKey(0));
    assert_eq!(actors.len(), 3);

    actors.drop_obj(ObjKey(2));
    assert_eq!(actors.len(), 0);
}
