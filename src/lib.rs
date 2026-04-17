#![forbid(unsafe_code)]
//! Safe in-memory secret types — the Rust counterpart to Pydantic's
//! `SecretStr` / `SecretBytes`.
//!
//! This crate provides two tiny new types, [`SecretString`] and
//! [`SecretBytes`], together with the [`ExposeSecret`] trait that gates
//! access to their underlying values. The goal is not to encrypt anything at
//! rest, but to make *accidental* leakage of in-process secrets — through
//! logs, panics, or `Debug` output — statically difficult.
//!
//! # Guarantees
//!
//! - The value is **never exposed** through [`Display`] or [`Debug`]; both
//!   print a fixed redaction marker.
//! - The backing buffer is **zeroed on drop** via the [`zeroize`] crate.
//! - With the `serde` feature, the types are **deserialize-only**:
//!   [`serde::Serialize`] is intentionally not implemented.
//! - Access to the raw value is explicit and reviewable: it goes through
//!   [`ExposeSecret::expose_secret`] and nothing else.
//! - Equality is evaluated in constant time to avoid leaking information
//!   through timing differences.
//! - The crate compiles under `#![forbid(unsafe_code)]`.
//!
//! # Example
//!
//! ```rust
//! use hosanna_rs_secret::{ExposeSecret, SecretString};
//!
//! let password = SecretString::from("hunter2");
//!
//! // Never exposed in logs.
//! println!("{password}");   // → **********
//! println!("{password:?}"); // → SecretString([REDACTED])
//!
//! // Explicit access to the raw value.
//! let raw: &str = password.expose_secret();
//! # assert_eq!(raw, "hunter2");
//! ```
//!
//! # Cargo features
//!
//! | Feature | Default | Effect |
//! |---------|---------|--------|
//! | `serde` | yes     | Enables [`serde::Deserialize`] for [`SecretString`] and [`SecretBytes`]. |
//!
//! [`Display`]: std::fmt::Display
//! [`Debug`]: std::fmt::Debug

pub mod error;
pub mod traits;
pub mod types;

pub use error::SecretError;
pub use traits::ExposeSecret;
pub use types::SecretBytes;
pub use types::SecretString;
