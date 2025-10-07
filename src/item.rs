use crate::id::ID;

#[derive()]
pub struct Item {
    pub id: ID,
    pub left: Option<ID>,
    pub right: Option<ID>,
    pub content: String,
    pub is_deleted: bool,
}
