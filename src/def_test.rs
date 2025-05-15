use crate::{agent::DUMMY_PLAYER, def::ObjKey};

use super::{Actors, MAX_ACTORS};

#[test]
fn test_actors_add_and_drop() {
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
    assert_eq!(actors.len(), 150);

    actors.gc();
    //does not clean up, as nothing is old enough
    assert_eq!(actors.len(), 150);
}

#[test]
fn test_actor_gc() {
    let mut actors = Actors::new(MAX_ACTORS);
    for _ in 0..actors.len() {
        actors.add_obj(DUMMY_PLAYER.clone());
    }

    for i in 0..actors.len() {
        actors.drop_obj(ObjKey(i));
    }

    // first ones added get deleted first and the new obj
    // get the early places
    assert_eq!(actors.add_obj(DUMMY_PLAYER.clone()), ObjKey(0));
    assert_eq!(actors.add_obj(DUMMY_PLAYER.clone()), ObjKey(1));
    assert_eq!(actors.add_obj(DUMMY_PLAYER.clone()), ObjKey(2));
    assert_eq!(actors.add_obj(DUMMY_PLAYER.clone()), ObjKey(3));
}
