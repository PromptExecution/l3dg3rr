//! Executive — promise-event-driven subsystem that manages surface lifecycles.
//!
//! The executive is the operational harness: it chains lifecycle promises,
//! enforces governance constraints, and produces an audit trail of every
//! transition. It is the "executive process subsystem" that b00t uses
//! to govern all managed processes.

pub mod harness;

pub use harness::*;
