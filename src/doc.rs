use std::collections::HashMap;
use crate::{ConflictResolver, Crdt, Item, SequenceCrdt, StateVector, YataResolver, ID};

#[derive(Debug)]
pub struct Doc<R: ConflictResolver = YataResolver> {
    pub client_id: u64,
    pub clock: u64,
    pub items: HashMap<ID, Item>,
    pub pending: Vec<Item>,
    pub state_vector: StateVector,
    pub head: Option<ID>,
    pub resolver: R,
}

impl Doc<YataResolver> {
    pub fn new(client_id: u64) -> Self {
        Self {
            client_id,
            clock: 0,
            items: HashMap::new(),
            pending: Vec::new(),
            state_vector: HashMap::new(),
            head: None,
            resolver: YataResolver,
        }
    }
}

impl<R: ConflictResolver> Doc<R> {
    pub fn with_resolver(client_id: u64, resolver: R) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_pos_in_empty_doc() {
        let doc = Doc::new(1);
        let (left, right) = doc.find_pos(0);
        assert!(left.is_none() && right.is_none());
    }

    #[test]
    fn find_pos_at_start() {
        // Need to implement apply as updates need to be applied.
        // left will always be None, right will point to first item
    }

    #[test]
    fn find_pos_at_end() {
    }


    #[test]
    fn find_pos_in_middle_of_item() {
        // The find_pos shouldn't split the item, the insert
        // algorithm will do that
    }

    #[test]
    fn find_pos_between_items() {
    }

    #[test]
    fn find_pos_skips_deleted_items() {
    }

    #[test]
    fn test_find_pos_with_unicode() {
        // We use chars().count() 
    }

    #[test]
    fn find_pos_all_items_deleted() {
    }

}
