//! Error types surfaced by the `hosanna-rs-secret` crate.
//!
//! The crate deliberately exposes a single, narrow error enum: secret types
//! are simple value wrappers, so the surface of things that can fail is
//! correspondingly small. Keeping every variant in one place makes it easy
//! to pattern-match at call sites and to audit every failure mode the
//! library can produce.

use thiserror::Error;

/// Errors that can arise when manipulating a secret value.
///
/// This enum is intentionally non-exhaustive in spirit — new variants may be
/// added in future minor versions as new fallible operations are introduced.
/// Callers that wish to be forward-compatible should match with a catch-all
/// arm (`_ => ...`).
#[derive(Debug, Error)]
pub enum SecretError {
    /// The caller attempted to reveal a secret in a context where exposure
    /// is forbidden (for example, inside a serializer).
    ///
    /// This variant exists so that higher-level code can distinguish an
    /// explicit policy refusal from a parsing failure.
    #[error("attempted to expose secret in an invalid context")]
    ExposureDenied,

    /// Parsing a secret from a textual representation failed.
    ///
    /// The `reason` payload carries a human-readable explanation suitable for
    /// logs. It **must not** contain the secret value itself — callers are
    /// responsible for stripping any sensitive material before constructing
    /// this variant.
    #[error("failed to parse secret from string: {reason}")]
    ParseError {
        /// Human-readable explanation of the parse failure. Safe to log.
        reason: String,
    },
}
