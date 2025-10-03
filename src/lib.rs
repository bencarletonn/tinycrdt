use std::collections::HashMap;

// YATA

struct ID {
    client_id: u32,
    clock: u32
}

struct Item {
    content: String,
    id: ID,
    is_deleted: bool,
    left: Option<Box<Item>>,
    right: Option<Box<Item>>,
}

struct Doc {
    id: ID,
    items: Vec<Item>,
    state_vector: HashMap<u32, u32>, // client_id -> max clock seen
}

impl Doc {

    fn insert(&mut self) {
        // Create new item(s) with (cliend_id, clock)
        // Record left and right neighbours
        // Splice into linked list
        // Merge if possible

        // Deterministic conflict resolution:
        //  - Compare (client_id, clock) of the new items
        //  - The smaller id comes first
    }

    fn delete(&mut self) {
        // Mark affected items (or parts of them) as deleted
        // Optionally merge consecutive deletions
    }

    fn get_delta(&mut self) {
    }

    fn apply_delta(&mut self) {
    }
}

// Syncing:
//  - Exchange state vectors
//  - Compute missing items (delta)
//  - Send/receive items
//  - Integrate into list by reconnecting neighbours left/right

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
