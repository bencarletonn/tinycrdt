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
            head: Option::None,
            resolver,
        }
    }

    fn next_id(&mut self) -> ID {}
    fn find_pos(&self, pos: usize) -> (Option<ID>, Option<ID>) {}
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
    fn insert(&mut self, pos: usize, text: &str) {}
    fn delete(&mut self, pos: usize, len: usize) {}
    fn value(&self) -> String {}
}
