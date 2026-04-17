//! The [`ExposeSecret`] trait — the single, deliberately verbose gate through
//! which raw secret material is made accessible to the rest of the program.
//!
//! The crate publishes no other escape hatch: there is no `AsRef`, no
//! `Deref`, no `Into<String>`. If you see `expose_secret()` in a diff, it
//! should prompt a second look during code review.

/// Explicit, deliberate access to the raw value behind a secret wrapper.
///
/// This trait is intentionally the *only* sanctioned path from a secret
/// wrapper to its inner value. The name is verbose on purpose — it should
/// jump out during code review, and it should feel slightly uncomfortable to
/// type. If a caller reaches for `expose_secret()` more than once per
/// request path, that is usually a sign the abstraction boundary is wrong.
///
/// # Example
///
/// ```rust
/// use hosanna_rs_secret::{ExposeSecret, SecretString};
///
/// let token = SecretString::from("hunter2");
/// let raw: &str = token.expose_secret();
/// assert_eq!(raw, "hunter2");
/// ```
///
/// # Contract for implementors
///
/// - `expose_secret` must return a borrow of the *exact* stored value,
///   without transformation, copying, or logging.
/// - Implementors must not cache, leak, or otherwise retain the returned
///   reference beyond what the borrow checker already enforces.
pub trait ExposeSecret {
    /// The concrete type of the exposed value.
    ///
    /// The `?Sized` bound allows implementors to expose unsized views such as
    /// [`str`] or `[u8]`, which is the common case for string- and
    /// byte-shaped secrets.
    type Value: ?Sized;

    /// Returns a borrow of the raw secret value.
    ///
    /// Use this method only when the underlying value is strictly required —
    /// for example, to build an HTTP authorization header or to open a
    /// database connection. The returned reference must never be logged,
    /// cloned into long-lived storage, or otherwise persisted.
    fn expose_secret(&self) -> &Self::Value;
}
