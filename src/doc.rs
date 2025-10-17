use crate::{ConflictResolver, Crdt, ID, Item, SequenceCrdt, StateVector, YataResolver};
use std::collections::HashMap;

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

    /// Minimal helper to insert a linked item sequence manually.
    /// TODO: remove this when `apply()` is available.
    fn insert_test_item(&mut self, item: Item) {
        let id = item.id;
        if self.head.is_none() {
            self.head = Some(id);
        }
        self.items.insert(id, item);
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

    /// Generates a new unique identifier for the next local operation.
    ///
    /// Increments the local clock and returns an [`ID`] combining this document's
    /// `client_id` and the previous clock value.
    ///
    /// Ensures all locally created items have monotonically increasing, unique IDs.
    fn next_id(&mut self) -> ID {
        let id = ID {
            client: self.client_id,
            clock: self.clock,
        };
        self.clock += 1;
        id 
    }

    /// Finds the insertion position in the linked list for a given character position.
    ///
    /// Traverses the list while skipping deleted items, returning the IDs of the
    /// items that should surround the insertion point and the offset at which the
    /// insertion should occur in the Item if it requires splitting
    ///
    /// # Arguments
    ///
    /// * `pos` - The character position where insertion should occur (0-indexed)
    ///
    /// # Returns
    ///
    /// A tuple `(left, right, right)` where `left` is the item before the insertion point
    /// and `right` is the item at or after it. Either can be `None` if inserting at
    /// the beginning or end of the list.
    fn find_pos(&self, pos: usize) -> (Option<ID>, Option<ID>, usize) {
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
            if index + len > pos {
                let offset = pos - index;
                return (left, Some(id), offset);
            }

            index += len;
            left = Some(id);
            current = item.right;
        }

        (left, None, 0)
    }

    fn try_link(&mut self, item: Item) {}
    fn link(&mut self, item: Item) {}
    fn resolve_pending(&mut self) {}
}

// impl Iter

impl<R: ConflictResolver> Crdt for Doc<R> {
    type Update = Vec<Item>;

    fn apply(&mut self, update: Self::Update) {}
    fn diff(&self, remote: &StateVector) -> Self::Update {
        Vec::<Item>::new()
    }
    fn state_vector(&self) -> StateVector {
        self.state_vector.clone()
    }
}

impl<R: ConflictResolver> SequenceCrdt for Doc<R> {
    fn insert(&mut self, pos: usize, text: &str) {
        // find_pos
    }
    fn delete(&mut self, pos: usize, len: usize) {}
    fn value(&self) -> String {
        "TODO".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn next_id_clock_starts_at_0() {
        let mut doc = Doc::new(1);
        let next_id = doc.next_id();
        assert!(next_id.clock == 0);
    }

    #[test]
    fn next_id_clock_one_less_than_doc_clock() {
        let mut doc = Doc::new(1);
        let next_id = doc.next_id();
        assert!(next_id.clock == doc.clock - 1)
    }

    #[test]
    fn find_pos_in_empty_doc() {
        let doc = Doc::new(1);
        let (left, right, offset) = doc.find_pos(0);
        assert!(left.is_none());
        assert!(right.is_none());
        assert!(offset == 0);
    }

    #[test]
    fn find_pos_at_start() {
        let mut doc = Doc::new(1);
        doc.insert_test_item(Item {
            id: ID {
                client: 1,
                clock: 0,
            },
            left: doc.head,
            right: None,
            content: "hello".to_owned(),
            is_deleted: false,
        });

        let (left, right, offset) = doc.find_pos(0);
        assert!(left.is_none());
        assert!(right.is_some());
        assert!(offset == 0);
    }

    #[test]
    fn find_pos_at_end() {
        let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id ,
            left: None,
            right: None,
            content: "Hello world!".to_owned(),
            is_deleted: false,
        });

        let (left, right, offset) = doc.find_pos(12);
        assert!(left.is_some());
        assert!(right.is_none());
    }

    #[test]
    fn find_pos_in_middle_of_item() {
        let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id ,
            left: None,
            right: None,
            content: "This is all one item!".to_owned(),
            is_deleted: false,
        });

        let (left, right, offset) = doc.find_pos(5);
        assert!(left.is_none());
        assert!(right.is_some());
        // When we obtain an offset > 0, we know where to split the right item
        assert!(offset == 5);
    }

    #[test]
    fn find_pos_between_items() {
         let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        let second_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id,
            left: None,
            right: Some(second_id),
            content: "First Item".to_owned(),
            is_deleted: false,
        });
        doc.insert_test_item(Item {
            id: second_id,
            left: Some(first_id),
            right: None,
            content: "Second Item".to_owned(),
            is_deleted: false,

        });

        let (left, right, offset) = doc.find_pos(10);
        assert!(left.is_some());
        assert!(right.is_some());
        assert!(offset == 0);
    }

    #[test]
    fn find_pos_skips_deleted_items() {}

    #[test]
    fn test_find_pos_with_unicode() {
        // We use chars().count()
    }

    #[test]
    fn find_pos_all_items_deleted() {}
}
