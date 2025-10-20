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

    /// Generates a new unique identifier for a local operation.
    ///
    /// Returns an [`ID`] with the current clock value, then advances the clock
    /// by the character count of `text`.
    fn next_id(&mut self, text: &str) -> ID {
        debug_assert!(!text.is_empty(), "next_id called with empty text");

        let id = ID {
            client: self.client_id,
            clock: self.clock,
        };
        self.clock += text.chars().count() as u64;
        self.state_vector.insert(self.client_id, self.clock - 1);
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

pub struct DocIterator<'a, R: ConflictResolver> {
    doc: &'a Doc<R>,
    current: Option<ID>,
}

impl<'a, R: ConflictResolver> IntoIterator for &'a Doc<R> {
    type Item = &'a Item;
    type IntoIter = DocIterator<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        DocIterator {
            doc: self,
            current: self.head,
        }
    }
}

impl<'a, R: ConflictResolver> Iterator for DocIterator<'a, R> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.current {
            let item = self.doc.items.get(&id).unwrap();
            self.current = item.right;

            if !item.is_deleted {
                return Some(item);
            }
        }
        None
    }
}

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
        if text.is_empty() {
            return;
        }
        let (mut left_id, right_id, offset) = self.find_pos(pos);

        // Handle splitting the right item if insertion is inside it
        if let Some(rid) = right_id {
            if offset > 0 {
                let right_item = self.items.get(&rid).unwrap();
                let right_item_left_id = right_item.left;

                // Could use a drain to avoid alloc, but we can't rely on byte offset for UTF-8
                // content
                let mut right_chars = right_item.content.chars();
                let left_content: String = right_chars.by_ref().take(offset).collect();
                let right_content: String = right_chars.collect();

                let left_split_id = self.next_id(&left_content);
                let left_split = Item {
                    id: left_split_id,
                    left: right_item_left_id,
                    right: Some(rid),
                    content: left_content,
                    is_deleted: false,
                };

                // Update the original right item
                let right_item_mut = self.items.get_mut(&rid).unwrap();
                right_item_mut.content = right_content;
                right_item_mut.left = Some(left_split_id);

                self.items.insert(left_split_id, left_split);

                // Update previous item's right pointer or head
                if let Some(prev_left_id) = right_item_left_id {
                    self.items.get_mut(&prev_left_id).unwrap().right = Some(left_split_id);
                } else {
                    self.head = Some(left_split_id);
                }

                left_id = Some(left_split_id);
            }
        }

        let new_id = self.next_id(text);
        let new_item = Item {
            id: new_id,
            left: left_id,
            right: right_id,
            content: text.to_string(),
            is_deleted: false,
        };

        self.items.insert(new_id, new_item);

        // Update links
        if let Some(lid) = left_id {
            self.items.get_mut(&lid).unwrap().right = Some(new_id);
        } else {
            self.head = Some(new_id);
        }

        if let Some(rid) = right_id {
            self.items.get_mut(&rid).unwrap().left = Some(new_id);
        }
    }

    fn delete(&mut self, pos: usize, len: usize) {}

    fn value(&self) -> String {
        self.into_iter().map(|item| item.content.as_str()).collect()
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
        let next_id = doc.next_id("abc");

        assert!(next_id.clock == 0);
    }

    #[test]
    fn next_id_advanced_clock_by_text_length() {
        let mut doc = Doc::new(1);
        let next_id = doc.next_id("abc");

        assert_eq!(doc.clock, 3);
        assert!(next_id.clock == doc.clock - 3);
    }

    #[test]
    fn next_id_updates_state_vector_to_last_used_clock() {
        let mut doc = Doc::new(1);
        let next_id = doc.next_id("hello");

        assert_eq!(next_id.clock, 0);

        assert_eq!(doc.clock, 5);

        // State vector stores last clock value used, not the next available
        let state = doc.state_vector.get(&1).unwrap();
        assert_eq!(*state, 4);
        assert_eq!(*state, doc.clock - 1);
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

        doc.insert(0, "hello");

        let (left, right, offset) = doc.find_pos(0);
        assert!(left.is_none());
        assert!(right.is_some());
        assert!(offset == 0);
    }

    #[test]
    fn find_pos_at_end() {
        let mut doc = Doc::new(1);

        doc.insert(0, "Hello world!");

        let (left, right, _) = doc.find_pos(12);
        assert!(left.is_some());
        assert!(right.is_none());
    }

    #[test]
    fn find_pos_in_middle_of_item() {
        let mut doc = Doc::new(1);

        doc.insert(0, "This is all one item!");

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

        doc.insert(0, "First Item");
        doc.insert(10, "Second Item");

        let (left, right, offset) = doc.find_pos(10);
        assert_eq!(left, Some(id(1, 0)));
        assert_eq!(right, Some(id(1, 10)));
        assert_eq!(offset, 0);
    }

    #[test]
    fn find_pos_skips_deleted_items() {
        let mut doc = Doc::new(1);

        doc.insert(0, "First Item");
        doc.insert(10, "Second Item");

        // Mark the middle item as deleted
        let space_id = doc.items.iter()
            .find(|(_, item)| item.content == "Second Item")
            .map(|(id, _)| *id)
            .unwrap();
        doc.items.get_mut(&space_id).unwrap().is_deleted = true;

        // Position 10 should be right to the first item
        let (left, right, offset) = doc.find_pos(10);
        assert_eq!(left, Some(id(1, 0)));
        assert!(right.is_none());
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_find_pos_with_unicode() {
        let mut doc = Doc::new(1);

        doc.insert(0, "hello");
        doc.insert(5, "🦀🦀");

        // Position 6 should be in the emoji item
        let (left, right, offset) = doc.find_pos(6);
        assert_eq!(left, Some(id(1, 0)));
        assert_eq!(right, Some(id(1, 5)));
        assert_eq!(offset, 1);

        // Position 7 should be at the end
        let (left, right, offset) = doc.find_pos(7);
        assert_eq!(left, Some(id(1, 5)));
        assert_eq!(right, None);
        assert_eq!(offset, 0);
    }

    #[test]
    fn find_pos_all_items_deleted() {
        let mut doc = Doc::new(1);

        doc.insert(0, "First Item");
        doc.insert(10, "Second Item");
        doc.insert(21, "Third Iten");

        for (_, item) in doc.items.iter_mut() {
            item.is_deleted = true;
        }

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

    #[test]
    fn test_iterator_empty_doc() {
        let doc = Doc::new(1);
        let mut iter = doc.into_iter();
        
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iterator_single_item() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "hello");
        assert_eq!(items[0].id.clock, 0);
    }

    #[test]
    fn iterator_empty_doc() {
        let doc = Doc::new(1);
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn iterator_single_item() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "hello");
    }

    #[test]
    fn iterator_multiple_items() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        doc.insert(5, " ");
        doc.insert(6, "world");
        
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].content, "hello");
        assert_eq!(items[1].content, " ");
        assert_eq!(items[2].content, "world");
    }

    #[test]
    fn iterator_skips_deleted_items() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        doc.insert(5, " ");
        doc.insert(6, "world");
        
        // Mark the middle item as deleted
        let space_id = doc.items.iter()
            .find(|(_, item)| item.content == " ")
            .map(|(id, _)| *id)
            .unwrap();
        doc.items.get_mut(&space_id).unwrap().is_deleted = true;
        
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "hello");
        assert_eq!(items[1].content, "world");
    }

    #[test]
    fn value_uses_iterator() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        doc.insert(5, " ");
        doc.insert(6, "world");
        
        assert_eq!(doc.value(), "hello world");
    }

    #[test]
    fn iterator_after_split() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        doc.insert(2, "X");  // Splits "hello" into "he", "X", "llo"
        
        let items: Vec<&Item> = doc.into_iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].content, "he");
        assert_eq!(items[1].content, "X");
        assert_eq!(items[2].content, "llo");
    }


}
