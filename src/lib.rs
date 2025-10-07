mod conflict;
mod doc;
mod id;
mod item;
mod state;
mod traits;

pub use id::ID;
pub use item::Item;
pub use state::StateVector;
pub use conflict::{ConflictResolver, YataResolver};
pub use traits::{Crdt, SequenceCrdt};
pub use doc::Doc;

// Future supporting structs/traits:
// 1. Transaction (batches multiple local operations before emitting single update)
// 2. Iterator on Doc 
// 3. GC?
// 4. IntegrationQueue (hold items whose deps, i.e. left + right, haven't arrived yet)
