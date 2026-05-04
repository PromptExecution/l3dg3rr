//! cfg-gated modules requiring `#[cfg(feature = "b00t")]`.
//!
//! These implement b00t-native types and refer to `_b00t_` node files.
//! They are designed for upstreaming into the b00t repository.
//! Within l3dg3rr, they are maintained here with the intent to contribute.

// Re-export so the parent crate can use them when feature is enabled
pub mod autoresearch;
pub mod opencode_provider;

pub use autoresearch::*;
pub use opencode_provider::*;
