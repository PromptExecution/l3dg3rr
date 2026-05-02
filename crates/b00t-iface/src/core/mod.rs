//! Core abstractions for the b00t process interface library.
//!
//! These primitives form the abstract syntax that solver-verified b00t governance
//! operates on. Each module is self-contained and can be independently reasoned about.
//!
//! # Architecture
//!
//! - `surface` — typed lifecycle: requirements → init → operate → terminate → maintain
//! - `governance` — policy: who can start, TTL, crash budget, agent roles
//! - `promise` — typed event: a value that will be produced at a future lifecycle point
//! - `machine` — abstract state machine over surface states

pub mod surface;
pub mod governance;
pub mod promise;
pub mod machine;

pub use surface::*;
pub use governance::*;
pub use promise::*;
pub use machine::*;
