# tinycrdt

A small sequence CRDT (Conflict-free Replicated Data Type) library for collaborative text editing, written in Rust with no dependencies. It is modelled on the YATA algorithm used by [Yjs](https://github.com/yjs/yjs).

> **Status: work in progress.** Local editing works. Syncing between replicas (`apply`, `diff`) and conflict resolution are not implemented yet. See [Roadmap](#roadmap).

## How it works

A document is a doubly linked list of **items**. Each item holds a run of text and has:

- a unique `ID` made of `(client, clock)`
- `left` and `right` neighbour IDs
- a `content` string
- an `is_deleted` flag (tombstone)

Every character consumes one clock tick, so an item's ID identifies its first character. Inserting in the middle of an item splits it. Deleting marks items as tombstones instead of removing them, so concurrent edits from other replicas can still refer to them. Positions are counted in Unicode characters, not bytes.

## Usage

```rust
use tinycrdt::{Doc, SequenceCrdt};

let mut doc = Doc::new(1); // client id

doc.insert(0, "Hello world");
doc.insert(5, ",");
doc.delete(6, 6);          // remove " world"
doc.insert(6, " there");

assert_eq!(doc.value(), "Hello, there");

// Iterate over live (non-deleted) items
for item in &doc {
    println!("{:?}: {}", item.id, item.content);
}
```

## API

| Item | Description |
| --- | --- |
| `Doc<R>` | The document. `Doc::new(client_id)` uses the default `YataResolver`. `Doc::with_resolver` accepts a custom one. |
| `SequenceCrdt` | `insert(pos, text)`, `delete(pos, len)`, `value()` |
| `Crdt` | `apply(update)`, `diff(&state_vector)`, `state_vector()` for replica sync |
| `ConflictResolver` | Trait for ordering concurrent inserts at the same position. `YataResolver` orders by `ID`. |
| `Item`, `ID`, `StateVector` | Core data types. `StateVector` maps client id to clock. |

## Development

```sh
cargo build
cargo test
```

## Roadmap

- [ ] `Crdt::apply` and `Crdt::diff` (exchanging updates between replicas)
- [ ] Integration of pending items whose `left`/`right` dependencies haven't arrived
- [ ] Full YATA conflict resolution for concurrent inserts
- [ ] Transactions that batch local operations into a single update
- [ ] Binary encoding of updates (`BinaryEncode`)
- [ ] Garbage collection of tombstones
- [ ] Thread-safety

## License

No license has been chosen yet.
