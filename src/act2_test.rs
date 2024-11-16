use std::collections::HashSet;

use super::STATES;

#[test]
fn test_unique_state_ids() {
    let mut ids = HashSet::new();
    for state in STATES {
        if ids.contains(&state.id) {
            panic!("duplicate state id: {}", state.id)
        }
        ids.insert(state.id);
    }
}
