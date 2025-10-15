mod conflict;
mod doc;
mod id;
mod item;
mod state;
mod traits;

pub use conflict::{ConflictResolver, YataResolver};
pub use doc::Doc;
pub use id::ID;
pub use item::Item;
pub use state::StateVector;
pub use traits::{Crdt, SequenceCrdt};

// Future supporting structs/traits:
// 1. struct Transaction/Txn (batches multiple local operations before emitting single update)
// 2. impl Iterator on Doc
// 3. GC?
// 4. IntegrationQueue (hold items whose deps, i.e. left + right, haven't arrived yet)
// 5. trait DeltaSerializable (serialize/deserialize updates)
// 6. struct Update (encapsulates deltas between state vectors)
