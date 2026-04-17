# hosanna-rs-secret

[![Crates.io](https://img.shields.io/crates/v/hosanna-rs-secret.svg)](https://crates.io/crates/hosanna-rs-secret)
[![Docs.rs](https://docs.rs/hosanna-rs-secret/badge.svg)](https://docs.rs/hosanna-rs-secret)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#minimum-supported-rust-version)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Safe in-memory secret types for Rust — the counterpart to Pydantic's `SecretStr` / `SecretBytes`.

The crate ships two tiny newtypes, `SecretString` and `SecretBytes`, whose job is to make **accidental** leakage of in-process secrets — through logs, panics, or `Debug` output — statically difficult. It does **not** encrypt anything at rest and is not a replacement for a proper secret manager; it is the last-mile hygiene that prevents `"{password}"` from showing up in your error reports.

## Why

Most real-world leaks don't come from broken crypto. They come from a `println!`, a `tracing::debug!`, an unhandled `panic!` with a config struct in scope, or a `Serialize` derive macro that quietly ships the field into a JSON response. `SecretString` and `SecretBytes` remove those footguns by construction:

- Their `Display` / `Debug` impls never reveal the value.
- They do not implement `Serialize`.
- They do not implement `Clone` — each copy is an extra place a secret can outlive its scope.
- The backing buffer is zeroised on drop.
- Equality is evaluated in constant time, so you can compare tokens or MACs without opening a timing side-channel.
- The whole crate compiles under `#![forbid(unsafe_code)]`.

## Install

```toml
[dependencies]
hosanna-rs-secret = "0.1"
```

Without the default `serde` feature (leaner dependency tree, no `Deserialize` impls):

```toml
[dependencies]
hosanna-rs-secret = { version = "0.1", default-features = false }
```

## Quick start

```rust
use hosanna_rs_secret::{ExposeSecret, SecretString};

let password = SecretString::from("hunter2");

// Safe to log — the value is masked.
println!("{password}");   // → **********
println!("{password:?}"); // → SecretString([REDACTED])

// Explicit, reviewable access to the raw value.
let raw: &str = password.expose_secret();
assert_eq!(raw, "hunter2");
```

Binary secrets work the same way:

```rust
use hosanna_rs_secret::{ExposeSecret, SecretBytes};

let key = SecretBytes::from(&[0xDE, 0xAD, 0xBE, 0xEF][..]);
assert_eq!(format!("{key:?}"), "SecretBytes([REDACTED], len=4)");
assert_eq!(key.expose_secret(), &[0xDE, 0xAD, 0xBE, 0xEF]);
```

### Loading from a config file

With the `serde` feature (enabled by default), both types deserialise from the usual formats. They **do not** serialise — that is intentional.

```rust
use hosanna_rs_secret::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    database_url: String,
    password: SecretString,
    api_key: SecretString,
}

let config: Config = serde_json::from_str(
    r#"{ "database_url": "postgres://…", "password": "hunter2", "api_key": "abc123" }"#,
)?;

// Safe to log the whole struct — only the non-secret fields appear.
println!("{:?}", config.database_url);
# Ok::<_, Box<dyn std::error::Error>>(())
```

## Public contract

| Behaviour                       | `SecretString`              | `SecretBytes`                       |
| ------------------------------- | --------------------------- | ----------------------------------- |
| `format!("{}", secret)`         | `**********`                | `**********`                        |
| `format!("{:?}", secret)`       | `SecretString([REDACTED])`  | `SecretBytes([REDACTED], len=N)`    |
| `.expose_secret()`              | `&str`                      | `&[u8]`                             |
| `.len()` / `.is_empty()`        | ✅ without exposing         | ✅ without exposing                 |
| `From<String>` / `From<&str>`   | ✅                          | —                                   |
| `From<Vec<u8>>` / `From<&[u8]>` | —                           | ✅                                  |
| `PartialEq` / `Eq`              | ✅ constant-time            | ✅ constant-time                    |
| `serde::Deserialize`            | ✅ (feature `serde`)        | ✅ (feature `serde`)                |
| `serde::Serialize`              | ❌ never                    | ❌ never                            |
| `Clone`                         | ❌ intentional              | ❌ intentional                      |
| Zeroised on drop                | ✅                          | ✅                                  |
| `unsafe`                        | ❌ forbidden crate-wide     | ❌ forbidden crate-wide             |

## Cargo features

| Feature | Default | Effect                                                                 |
| ------- | ------- | ---------------------------------------------------------------------- |
| `serde` | yes     | Enables `serde::Deserialize` for `SecretString` and `SecretBytes`.     |

## Minimum supported Rust version

This crate requires **Rust 1.85** (the stabilisation of edition 2024). Bumping the MSRV is treated as a minor-version change.

## Development

```bash
cargo test                       # default features
cargo test --no-default-features # verify crate builds without serde
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo doc --open                 # check the rustdoc examples
```

A git `pre-commit` hook is wired via [`cargo-husky`](https://github.com/rhysd/cargo-husky). The hook source lives in `.cargo-husky/hooks/pre-commit` and is copied into `.git/hooks/` automatically the first time you run `cargo test` in a fresh clone. It runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and the full test matrix before every commit. In an emergency you can bypass it with `git commit --no-verify`.

Changelog entries are authored with [Changie](https://github.com/miniscruff/changie): add an entry under `.changes/unreleased/` instead of editing `CHANGELOG.md` by hand.

## License

Licensed under the Apache License, Version 2.0 ([`LICENSE`](LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
