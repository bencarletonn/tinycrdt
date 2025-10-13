use std::collections::HashMap;
use crate::{ConflictResolver, Crdt, SequenceCrdt, ID, Item, StateVector};

#[derive(Debug)]
pub struct Doc<R: ConflictResolver> {
    pub client_id: u64,
    pub clock: u64,
    pub items: HashMap<ID, Item>,
    pub state_vector: StateVector,
    pub resolver: R,
}

impl<R: ConflictResolver> Doc<R> {
    pub fn new (client_id: u64, resolver: R) -> Self {
        Self {
            client_id,
            clock: 0,
            items: HashMap::new(),
            state_vector: HashMap::new(),
            resolver,
        }
    }
}

// TODO
impl<R: ConflictResolver> Crdt for Doc<R> {
    type Update = Vec<Item>;

    fn apply(&mut self, update: Self::Update) {}
    fn diff(&self, remote: &StateVector) -> Self::Update {}
    fn state_vector(&self) -> StateVector {}
}

// TODO
impl<R: ConflictResolver> SequenceCrdt for Doc<R> {
    fn insert(&mut self, pos: usize, content: &str) {}
    fn delete(&mut self, pos: usize, len: usize) {}
    fn value(&self) -> String {}
}
