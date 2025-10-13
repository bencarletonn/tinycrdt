#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ID {
    pub client: u64,
    pub clock: u64,
}
