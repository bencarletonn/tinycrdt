use std::collections::HashMap;
use crate::{ConflictResolver, Crdt, SequenceCrdt, ID, Item, StateVector};

#[derive(Debug)]
pub struct Doc<R: ConflictResolver> {
    pub client_id: u64,
    pub clock: u64,
    pub items: HashMap<ID, Item>,
    pub pending: Vec<Item>,
    pub state_vector: StateVector,
    pub head: Option<ID>,
    pub resolver: R,
}

impl<R: ConflictResolver> Doc<R> {
    pub fn new (client_id: u64, resolver: R) -> Self {
        Self {
            client_id,
            clock: 0,
            items: HashMap::new(),
            pending: Vec::new(),
            state_vector: HashMap::new(),
            head: None,
            resolver,
        }
    }

    fn next_id(&mut self) -> ID {}

    /// Finds the insertion position in the linked list for a given character position.
    ///
    /// Traverses the list while skipping deleted items, returning the IDs of the
    /// items that should surround the insertion point.
    ///
    /// # Arguments
    ///
    /// * `pos` - The character position where insertion should occur (0-indexed)
    ///
    /// # Returns
    ///
    /// A tuple `(left, right)` where `left` is the item before the insertion point
    /// and `right` is the item at or after it. Either can be `None` if inserting at
    /// the beginning or end of the list.
    fn find_pos(&self, pos: usize) -> (Option<ID>, Option<ID>) {
        let mut index = 0;
        let mut left = None;
        let mut current = self.head;

        while let Some(id) = current {
            let item = &self.items[&id];
            if item.is_deleted {
                current = item.right;
                continue;
            }

            let len = item.content.chars().count();
            if index + len >= pos {
                return (left, Some(id));
            }

            index += len;
            left = Some(id);
            current = item.right;
        }

        (left, None)
    }

    fn try_link(&mut self, item: Item) {}
    fn link(&mut self, item: Item) {}
    fn resolve_pending(&mut self) {}
}

// impl Iter 

impl<R: ConflictResolver> Crdt for Doc<R> {
    type Update = Vec<Item>;

    fn apply(&mut self, update: Self::Update) {}
    fn diff(&self, remote: &StateVector) -> Self::Update {}
    fn state_vector(&self) -> StateVector { self.state_vector.clone() }
}

impl<R: ConflictResolver> SequenceCrdt for Doc<R> {
    fn insert(&mut self, pos: usize, text: &str) {
        // find_pos

    }
    fn delete(&mut self, pos: usize, len: usize) {}
    fn value(&self) -> String {}
}
