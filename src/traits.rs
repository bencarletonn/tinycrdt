use crate::state::StateVector;

pub trait Crdt {
    type Update;

    fn apply(&mut self, update: Self::Update);
    fn diff(&self, remote: &StateVector) -> Self::Update;
    fn state_vector(&self) -> StateVector;
}

pub trait SequenceCrdt {
    fn insert(&mut self, pos: usize, content: &str);
    fn delete(&mut self, pos: usize, len: usize);
    fn value(&self) -> String;
}
