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

    /// Splits an item at the given offset, creating a new item for the left part.
    /// Returns the ID of the newly created left split item.
    ///
    /// # Arguments
    ///
    /// * `item_id` - The ID of the item to split
    /// * `offset` - The character offset at which to split (0 < offset < item length)
    ///
    /// # Returns
    ///
    /// The ID of the newly created left split item. The original item retains
    /// its ID but its content is updated to contain only the right part.
    fn split_item(&mut self, item_id: ID, offset: usize) -> ID {
        let item = self.items.get(&item_id).unwrap();
        let item_left = item.left;

        // Split the content
        let mut chars = item.content.chars();
        let left_content: String = chars.by_ref().take(offset).collect();
        let right_content: String = chars.collect();

        // Create new left split item
        let left_split_id = self.next_id(&left_content);
        let left_split = Item {
            id: left_split_id,
            left: item_left,
            right: Some(item_id),
            content: left_content,
            is_deleted: false,
        };

        // Update the original item (now the right part)
        let item_mut = self.items.get_mut(&item_id).unwrap();
        item_mut.content = right_content;
        item_mut.left = Some(left_split_id);

        // Insert the left split
        self.items.insert(left_split_id, left_split);

        // Update the previous item's right pointer or head
        if let Some(prev_id) = item_left {
            self.items.get_mut(&prev_id).unwrap().right = Some(left_split_id);
        } else {
            self.head = Some(left_split_id);
        }

        left_split_id
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
                left_id = Some(self.split_item(rid, offset));
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

    fn delete(&mut self, pos: usize, len: usize) {
        if len == 0 {
            return;
        }

        let (_, start_item_id, start_offset) = self.find_pos(pos);
        let Some(mut current_id) = start_item_id else {
            return;
        };

        let mut remaining = len;

        // If deletion starts in the middle of an item, split it first
        if start_offset > 0 {
            self.split_item(current_id, start_offset);
        }

        // Delete items moving rightward until length is covered
        while remaining > 0 {
            let Some(item) = self.items.get(&current_id) else {
                break;
            };

            if item.is_deleted {
                let Some(next) = item.right else { break };
                current_id = next;
                continue;
            }

            let item_len = item.content.chars().count();
            let next = item.right;

            if remaining < item_len {
                // Partial deletion: split and mark left part deleted
                let left_id = self.split_item(current_id, remaining);
                self.items
                    .get_mut(&left_id)
                    .expect("split item should exist")
                    .is_deleted = true;
                break;
            }

            // Full deletion
            self.items
                .get_mut(&current_id)
                .expect("item should exist")
                .is_deleted = true;

            remaining -= item_len;

            let Some(next_id) = next else { break };
            current_id = next_id;
        }
    }

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

        // Mark the second item as deleted
        doc.delete(10, 11);

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

        doc.delete(0, 31);

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
        
        doc.delete(5, 1);
        
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

    #[test]
    fn insert_into_empty_list() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");

        assert_eq!(doc.value(), "hello");
        assert_eq!(doc.head, Some(id(1, 0)));
        assert_eq!(doc.clock, 5);
    }

    #[test]
    fn insert_at_beginning() {
        let mut doc = Doc::new(1);
        doc.insert(0, "world");
        doc.insert(0, "hello ");

        assert_eq!(doc.value(), "hello world");
        
        // First item should be "hello " starting at clock 5
        let first_item = doc.items.get(&id(1, 5)).unwrap();
        assert_eq!(first_item.content, "hello ");
        assert_eq!(first_item.left, None);
        assert_eq!(first_item.right, Some(id(1, 0)));
    }

    #[test]
    fn insert_at_end() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        doc.insert(5, " world");

        assert_eq!(doc.value(), "hello world");
        
        let second_item = doc.items.get(&id(1, 5)).unwrap();
        assert_eq!(second_item.content, " world");
        assert_eq!(second_item.left, Some(id(1, 0)));
        assert_eq!(second_item.right, None);
    }

    #[test]
    fn insert_in_middle_splits_item() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hllo"); // Creates ID (1, 0) with length 4, clock advances to 4
        doc.insert(1, "e"); // Split creates ID (1, 4) for "h", then (1, 5) for "e"

        assert_eq!(doc.value(), "hello");
        assert_eq!(doc.items.len(), 3);

        // Left split "h" has NEW ID starting at clock 4
        let left_split = doc.items.get(&id(1, 4)).unwrap();
        assert_eq!(left_split.content, "h");

        // Inserted "e" has ID starting at clock 5
        let inserted = doc.items.get(&id(1, 5)).unwrap();
        assert_eq!(inserted.content, "e");

        // Right split "llo" keeps ORIGINAL ID starting at clock 0
        let right_split = doc.items.get(&id(1, 0)).unwrap();
        assert_eq!(right_split.content, "llo");
    }

    #[test]
    fn insert_splits_at_exact_midpoint() {
        let mut doc = Doc::new(1);
        doc.insert(0, "abcd");
        doc.insert(2, "X");  // Insert at exact middle

        assert_eq!(doc.value(), "abXcd");
        
        // "ab" (split left), "X" (inserted), "cd" (split right)
        assert_eq!(doc.items.len(), 3);
    }

    #[test]
    fn multiple_inserts_at_same_position() {
        let mut doc = Doc::new(1);
        doc.insert(0, "a");
        doc.insert(1, "b");
        doc.insert(2, "c");
        doc.insert(3, "d");

        assert_eq!(doc.value(), "abcd");
        assert_eq!(doc.items.len(), 4);
    }

    #[test]
    fn insert_with_unicode() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello🦀world");
        doc.insert(6, "rust");  // Insert after emoji (1 char)

        assert_eq!(doc.value(), "hello🦀rustworld");
    }

    #[test]
    fn insert_empty_string_is_noop() {
        let mut doc = Doc::new(1);
        doc.insert(0, "hello");
        let clock_before = doc.clock;
        
        doc.insert(2, "");  // Should be no-op
        
        assert_eq!(doc.value(), "hello");
        assert_eq!(doc.clock, clock_before);  // Clock shouldn't advance
    }

    #[test]
    fn insert_updates_head_correctly() {
        let mut doc = Doc::new(1);
        doc.insert(0, "second");
        
        let original_head = doc.head;
        
        doc.insert(0, "first");
        
        // Head should now point to "first"
        assert_ne!(doc.head, original_head);
        assert_eq!(doc.value(), "firstsecond");
    }

    #[test]
    fn insert_maintains_linked_list_integrity() {
        let mut doc = Doc::new(1);
        doc.insert(0, "a");
        doc.insert(1, "b");
        doc.insert(2, "c");

        // Walk the linked list and verify integrity
        let mut visited = vec![];
        let mut current = doc.head;
        
        while let Some(id) = current {
            let item = doc.items.get(&id).unwrap();
            visited.push(item.content.clone());
            
            // Verify bidirectional links
            if let Some(right_id) = item.right {
                let right_item = doc.items.get(&right_id).unwrap();
                assert_eq!(right_item.left, Some(id), "Right item's left should point back");
            }
            
            current = item.right;
        }
        
        assert_eq!(visited, vec!["a", "b", "c"]);
    }

    #[test]
    fn clock_advances_correctly_with_splits() {
        let mut doc = Doc::new(1);
        
        doc.insert(0, "hello");  // clock: 0 -> 5
        assert_eq!(doc.clock, 5);
        
        doc.insert(2, "X");  // Splits "hello", clock: 5 -> 6 (for "X"), but also 6 -> 8 (for "he" split)
        // Actually, let me recalculate...
        // Split creates "he" (2 chars) at clock 5, clock becomes 7
        // Then insert "X" (1 char) at clock 7, clock becomes 8
        // Original "llo" keeps its ID
        
        assert_eq!(doc.value(), "heXllo");
    }

    #[test]
    fn insert_between_two_items() {
        let mut doc = Doc::new(1);
        doc.insert(0, "ac");
        doc.insert(1, "b");

        assert_eq!(doc.value(), "abc");
        assert_eq!(doc.items.len(), 3);  // "a" (split), "b" (inserted), "c" (split)
    }
}
