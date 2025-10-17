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
    /// Returns the neighboring items and offset for where to insert. If `offset > 0`,
    /// the `right` item should be split at that offset.
    ///
    /// # Arguments
    ///
    /// * `pos` - The 0-indexed character position for insertion
    ///
    /// # Returns
    ///
    /// `(left, right, offset)`:
    /// * `left` - Item before insertion point, or `None` if at start
    /// * `right` - Item at/after insertion point, or `None` if at end  
    /// * `offset` - Characters into `right` item (0 = before, >0 = split here)
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

    // Helper function to create test IDs
    fn id(client: u64, clock: u64) -> ID {
        ID { client, clock }
    }

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
            id: first_id,
            left: None,
            right: None,
            content: "Hello world!".to_owned(),
            is_deleted: false,
        });

        let (left, right, _) = doc.find_pos(12);
        assert!(left.is_some());
        assert!(right.is_none());
    }

    #[test]
    fn find_pos_in_middle_of_item() {
        let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id,
            left: None,
            right: None,
            content: "This is all one item!".to_owned(),
            is_deleted: false,
        });

        // Position 5 should have one item to the right, with
        // an offset of 5, indicating the right item will need
        // to be split on insertion
        let (left, right, offset) = doc.find_pos(5);
        assert!(left.is_none());
        assert!(right.is_some());
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
        assert_eq!(left, Some(id(1, 0)));
        assert_eq!(right, Some(id(1, 1)));
        assert_eq!(offset, 0);
    }

    #[test]
    fn find_pos_skips_deleted_items() {
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
            is_deleted: true,
        });

        // Position 10 should be right to the first item
        let (left, right, offset) = doc.find_pos(10);
        assert_eq!(left, Some(id(1, 0)));
        assert!(right.is_none());
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_find_pos_with_unicode() {
        let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        let second_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id,
            left: None,
            right: Some(second_id),
            content: "hello".to_owned(),
            is_deleted: false,
        });
        doc.insert_test_item(Item {
            id: second_id,
            left: Some(first_id),
            right: None,
            content: "🦀🦀".to_owned(),
            is_deleted: false,
        });

        // Position 6 should be in the emoji item
        let (left, right, offset) = doc.find_pos(6);
        assert_eq!(left, Some(id(1, 0)));
        assert_eq!(right, Some(id(1, 1)));
        assert_eq!(offset, 1);

        // Position 7 should be at the end
        let (left, right, offset) = doc.find_pos(7);
        assert_eq!(left, Some(id(1, 1)));
        assert_eq!(right, None);
        assert_eq!(offset, 0);
    }

    #[test]
    fn find_pos_all_items_deleted() {
        let mut doc = Doc::new(1);
        let first_id = doc.next_id();
        let second_id = doc.next_id();
        let third_id = doc.next_id();
        doc.insert_test_item(Item {
            id: first_id,
            left: None,
            right: Some(second_id),
            content: "First Item".to_owned(),
            is_deleted: true,
        });
        doc.insert_test_item(Item {
            id: second_id,
            left: Some(first_id),
            right: Some(third_id),
            content: "Second Item".to_owned(),
            is_deleted: true,
        });
        doc.insert_test_item(Item {
            id: third_id,
            left: Some(second_id),
            right: None,
            content: "Third Item".to_owned(),
            is_deleted: true,
        });

        // Position 5 should be at the start
        let (left, right, offset) = doc.find_pos(5);
        assert_eq!(left, None);
        assert_eq!(right, None);
        assert_eq!(offset, 0);

        // Position 10 should be at the start
        let (left, right, offset) = doc.find_pos(10);
        assert_eq!(left, None);
        assert_eq!(right, None);
        assert_eq!(offset, 0);

        // Position 15 should be at the start
        let (left, right, offset) = doc.find_pos(15);
        assert_eq!(left, None);
        assert_eq!(right, None);
        assert_eq!(offset, 0);
    }
}
