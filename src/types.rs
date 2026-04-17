//! Concrete secret wrappers: [`SecretString`] and [`SecretBytes`].
//!
//! Both types are thin new types over their underlying storage. Their purpose
//! is not to transform the value but to *constrain the surface area* around
//! it: redacted formatters, zeroization on drop, constant-time equality, and
//! no serialization. The two types share a single private
//! `constant_time_eq` helper defined at the bottom of this module.

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::traits::ExposeSecret;

#[cfg(feature = "serde")]
use serde::Deserialize;

// ── SecretString ─────────────────────────────────────────────────────────────

/// A UTF-8 string whose contents are treated as sensitive material.
///
/// `SecretString` wraps a [`String`] and layers four guarantees on top of it:
///
/// 1. **Redacted formatting.** [`Display`] yields `**********` and [`Debug`]
///    yields `SecretString([REDACTED])`, so the value cannot leak into logs
///    through accidental `{}` / `{:?}` interpolation.
/// 2. **Zeroisation on drop.** The backing buffer is overwritten with zeroes
///    when the wrapper is dropped, via [`zeroize::ZeroizeOnDrop`].
/// 3. **No serialization.** Only [`Deserialize`] is implemented (gated by the
///    `serde` feature). Implementing [`serde::Serialize`] would reintroduce
///    the leakage this type is built to prevent.
/// 4. **Constant-time equality.** [`PartialEq`] compares byte-by-byte in
///    constant time, removing a common class of timing side-channels.
///
/// The type is intentionally **not** [`Clone`]. Each copy is an extra place
/// the secret can outlive its intended scope, and callers who truly need a
/// copy can always reconstruct one through [`ExposeSecret::expose_secret`].
///
/// # Example
///
/// ```rust
/// use hosanna_rs_secret::{ExposeSecret, SecretString};
///
/// let password = SecretString::from("hunter2");
///
/// // Safe to log — the value is masked.
/// assert_eq!(format!("{password}"),   "**********");
/// assert_eq!(format!("{password:?}"), "SecretString([REDACTED])");
///
/// // Deliberate, reviewable access to the raw value.
/// assert_eq!(password.expose_secret(), "hunter2");
/// ```
///
/// [`Display`]: std::fmt::Display
/// [`Debug`]: std::fmt::Debug
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    /// Wraps an owned [`String`] as a `SecretString`, taking ownership of the
    /// underlying buffer.
    ///
    /// Prefer this constructor — or the [`From`] impls — over any indirect
    /// route so that the intent to treat the value as sensitive is visible
    /// at the call site.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the length of the secret in bytes, without exposing its
    /// contents.
    ///
    /// The length itself is considered non-sensitive. If your threat model
    /// treats length as a side-channel, do not call this method.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the secret is the empty string.
    ///
    /// Exposing emptiness is considered safe for the same reason as
    /// [`Self::len`]: it reveals no information beyond a single bit of
    /// metadata.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ExposeSecret for SecretString {
    type Value = str;

    fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretString {
    /// Writes a fixed-length redaction marker, never the underlying value.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("**********")
    }
}

impl fmt::Debug for SecretString {
    /// Writes `SecretString([REDACTED])`. The length is **not** included so
    /// that structural logs do not inadvertently leak the size of the secret.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString([REDACTED])")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl PartialEq for SecretString {
    /// Compares two secrets in constant time.
    ///
    /// Equality is the single operation for which a naive implementation
    /// would leak information through the CPU's early-exit branch, so this
    /// impl delegates to `constant_time_eq`.
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(self.0.as_bytes(), other.0.as_bytes())
    }
}

impl Eq for SecretString {}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SecretString {
    /// Serializes a JSON (or other serde format) string into a
    /// `SecretString`. The intermediate owned [`String`] lives only for the
    /// duration of this call before being moved into the wrapper.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(SecretString::new)
    }
}

// Intentionally absent: impl Serialize — secrets are serialize-only.
// Intentionally absent: impl Clone     — each copy is an extra attack surface.

// ── SecretBytes ──────────────────────────────────────────────────────────────

/// An opaque sequence of bytes whose contents are treated as sensitive
/// material.
///
/// `SecretBytes` is the binary counterpart to [`SecretString`] and carries
/// the same four guarantees: redacted formatting, zeroization on drop, no
/// serialization, and constant-time equality. Use it for raw key material,
/// HMAC inputs, PEM-decoded blobs, or any byte buffer that should not be
/// echoed back through logs.
///
/// The [`Debug`] representation includes the byte length — unlike
/// [`SecretString`] — because knowing that a key is, say, 32 bytes long is
/// usually useful for debugging and is rarely a meaningful side-channel for
/// binary material.
///
/// Like [`SecretString`], this type is deliberately not [`Clone`].
///
/// # Example
///
/// ```rust
/// use hosanna_rs_secret::{ExposeSecret, SecretBytes};
///
/// let key = SecretBytes::from(&[0xDE, 0xAD, 0xBE, 0xEF][..]);
/// assert_eq!(format!("{key}"),   "**********");
/// assert_eq!(format!("{key:?}"), "SecretBytes([REDACTED], len=4)");
/// assert_eq!(key.expose_secret(), &[0xDE, 0xAD, 0xBE, 0xEF]);
/// ```
///
/// [`Debug`]: std::fmt::Debug
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Wraps an owned [`Vec<u8>`] as a `SecretBytes`, taking ownership of the
    /// underlying buffer.
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    /// Returns the length of the secret in bytes, without exposing its
    /// contents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the secret has zero length.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ExposeSecret for SecretBytes {
    type Value = [u8];

    fn expose_secret(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for SecretBytes {
    /// Writes a fixed-length redaction marker, never the underlying bytes.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("**********")
    }
}

impl fmt::Debug for SecretBytes {
    /// Writes `SecretBytes([REDACTED], len=N)`. The length is included
    /// because — unlike with text passwords — it is usually informative and
    /// rarely sensitive.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SecretBytes([REDACTED], len={})", self.0.len())
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for SecretBytes {
    fn from(value: &[u8]) -> Self {
        Self::new(value.to_vec())
    }
}

impl PartialEq for SecretBytes {
    /// Compares two byte secrets in constant time via `constant_time_eq`.
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(&self.0, &other.0)
    }
}

impl Eq for SecretBytes {}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SecretBytes {
    /// Serializes a `Vec<u8>`-compatible serde value (typically a JSON
    /// array of byte-sized integers) into a `SecretBytes`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer).map(SecretBytes::new)
    }
}

// Intentionally absent: impl Serialize — secrets are deserialise-only.
// Intentionally absent: impl Clone     — each copy is an extra attack surface.

// ── Shared internal helper ──────────────────────────────────────────────────

/// Constant-time byte-slice equality, used by both [`SecretString`] and
/// [`SecretBytes`].
///
/// A naive `==` on byte slices short-circuits on the first differing byte,
/// which makes the comparison time a function of the matching prefix length
/// and opens a well-known timing side-channel. This routine instead folds
/// the full XOR of both slices into a single accumulator so that its running
/// time depends only on the slice length, never on the slice contents.
///
/// The length check up-front is not itself constant-time, but unequal
/// lengths already imply inequality and the length of a secret is not
/// considered sensitive in this crate's threat model (see the doc on
/// [`SecretString::len`]).
///
/// Implemented by hand so that the crate pulls no additional dependency for
/// this single operation.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{SecretBytes, SecretString};
    use crate::traits::ExposeSecret;

    // ── SecretString ────────────────────────────────────────────────────

    #[test]
    fn secret_string_display_is_masked() {
        let secret = SecretString::new("my_password".to_string());
        assert_eq!(format!("{secret}"), "**********");
    }

    #[test]
    fn secret_string_debug_is_redacted() {
        let secret = SecretString::new("my_password".to_string());
        assert_eq!(format!("{secret:?}"), "SecretString([REDACTED])");
    }

    #[test]
    fn secret_string_expose_returns_value() {
        let secret = SecretString::new("my_password".to_string());
        assert_eq!(secret.expose_secret(), "my_password");
    }

    #[test]
    fn secret_string_from_string() {
        let secret = SecretString::from("value".to_string());
        assert_eq!(secret.expose_secret(), "value");
    }

    #[test]
    fn secret_string_from_str_slice() {
        let secret = SecretString::from("value");
        assert_eq!(secret.expose_secret(), "value");
    }

    #[test]
    fn secret_string_len_does_not_expose() {
        let secret = SecretString::new("hello".to_string());
        assert_eq!(secret.len(), 5);
        assert!(!secret.is_empty());
    }

    #[test]
    fn secret_string_empty_is_detected() {
        let secret = SecretString::new(String::new());
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn secret_string_equality_same_value() {
        let a = SecretString::new("password".to_string());
        let b = SecretString::new("password".to_string());
        assert_eq!(a, b);
    }

    #[test]
    fn secret_string_inequality_different_value() {
        let a = SecretString::new("password".to_string());
        let b = SecretString::new("other".to_string());
        assert_ne!(a, b);
    }

    #[test]
    fn secret_string_inequality_different_length() {
        let a = SecretString::new("abc".to_string());
        let b = SecretString::new("abcd".to_string());
        assert_ne!(a, b);
    }

    // ── SecretBytes ─────────────────────────────────────────────────────

    #[test]
    fn secret_bytes_display_is_masked() {
        let secret = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(format!("{secret}"), "**********");
    }

    #[test]
    fn secret_bytes_debug_shows_length_only() {
        let secret = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(format!("{secret:?}"), "SecretBytes([REDACTED], len=3)");
    }

    #[test]
    fn secret_bytes_expose_returns_slice() {
        let secret = SecretBytes::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(secret.expose_secret(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn secret_bytes_from_vec() {
        let secret = SecretBytes::from(vec![1, 2, 3]);
        assert_eq!(secret.expose_secret(), &[1, 2, 3]);
    }

    #[test]
    fn secret_bytes_from_slice() {
        let secret = SecretBytes::from([1u8, 2, 3].as_slice());
        assert_eq!(secret.expose_secret(), &[1, 2, 3]);
    }

    #[test]
    fn secret_bytes_len_does_not_expose() {
        let secret = SecretBytes::new(vec![1, 2, 3, 4]);
        assert_eq!(secret.len(), 4);
        assert!(!secret.is_empty());
    }

    #[test]
    fn secret_bytes_empty_is_detected() {
        let secret = SecretBytes::new(vec![]);
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[test]
    fn secret_bytes_equality_same_value() {
        let a = SecretBytes::new(vec![1, 2, 3]);
        let b = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn secret_bytes_inequality_different_value() {
        let a = SecretBytes::new(vec![1, 2, 3]);
        let b = SecretBytes::new(vec![1, 2, 4]);
        assert_ne!(a, b);
    }

    #[test]
    fn secret_bytes_inequality_different_length() {
        let a = SecretBytes::new(vec![1, 2, 3]);
        let b = SecretBytes::new(vec![1, 2, 3, 4]);
        assert_ne!(a, b);
    }

    // ── Serde ───────────────────────────────────────────────────────────

    #[cfg(feature = "serde")]
    #[test]
    fn secret_string_deserializes_from_json_string() {
        let json = r#""my_secret_value""#;
        let secret: SecretString =
            serde_json::from_str(json).expect("should deserialize from JSON string");
        assert_eq!(secret.expose_secret(), "my_secret_value");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn secret_string_in_struct_deserializes() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Config {
            password: SecretString,
            api_key: SecretString,
        }

        let json = r#"{"password": "hunter2", "api_key": "abc123"}"#;
        let config: Config =
            serde_json::from_str(json).expect("should deserialize config from JSON");

        assert_eq!(config.password.expose_secret(), "hunter2");
        assert_eq!(config.api_key.expose_secret(), "abc123");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn secret_bytes_deserializes_from_json_array() {
        let json = r#"[1, 2, 3, 4]"#;
        let secret: SecretBytes =
            serde_json::from_str(json).expect("should deserialize from JSON array");
        assert_eq!(secret.expose_secret(), &[1u8, 2, 3, 4]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn secret_string_display_unchanged_after_deserialization() {
        let json = r#""my_secret""#;
        let secret: SecretString = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(format!("{secret}"), "**********");
    }
}
