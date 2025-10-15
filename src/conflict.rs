use crate::{ID, Item};
use std::cmp::Ordering;
use std::collections::HashMap;

pub trait ConflictResolver {
    fn resolve(&self, a: &Item, b: &Item, doc: &HashMap<ID, Item>) -> Ordering;
}

#[derive(Debug)]
pub struct YataResolver;

impl ConflictResolver for YataResolver {
    fn resolve(&self, a: &Item, b: &Item, _doc: &HashMap<ID, Item>) -> Ordering {
        a.id.cmp(&b.id)
    }
}
