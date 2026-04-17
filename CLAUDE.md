# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project purpose

`hosanna-rs-secret` is a Rust library crate that provides in-memory secret types — the Rust equivalent of Pydantic's `SecretStr` / `SecretBytes`. The public surface is two types (`SecretString`, `SecretBytes`) and one trait (`ExposeSecret`).

The crate-level rustdoc in `src/lib.rs` is the canonical description of guarantees and features; read it first when picking up the project.

## Commands

```bash
cargo check
cargo test                       # default features (serde enabled)
cargo test --no-default-features # verify crate builds without serde
cargo test --all-features
cargo test <test_name>           # run a single test, e.g. secret_string_debug_is_redacted
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check       # what the pre-commit hook runs
cargo doc --open                 # verify the lib.rs doc example compiles
```

Changelog entries use [Changie](https://github.com/miniscruff/changie) — add changes under `.changes/unreleased/` rather than editing `CHANGELOG.md` directly. Kinds and versioning rules are in `.changie.yaml`.

## Commit messages

Commits follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/). The two layers are complementary, not redundant: the Conventional Commits prefix documents the git history, Changie documents the user-facing changelog. A commit can have both, one, or neither — the table below says which.

Format:

```
<type>[optional scope][!]: <short summary>

[optional body explaining the *why*]

[optional footer(s), e.g. BREAKING CHANGE:, Refs: #123]
```

Mapping of Conventional Commit types to Changie kinds (kinds are declared in `.changie.yaml`):

| CC type    | Changie kind | SemVer bump | Fragment required? |
| ---------- | ------------ | ----------- | ------------------ |
| `feat`     | `Added`      | minor       | ✅ yes             |
| `fix`      | `Fixed`      | patch       | ✅ yes             |
| `perf`     | `Fixed` or `Changed` depending on user impact | patch / major | ✅ yes |
| `refactor` | (none)       | —           | ❌ no              |
| `docs`     | (none)       | —           | ❌ no              |
| `test`     | (none)       | —           | ❌ no              |
| `chore`    | (none)       | —           | ❌ no              |
| `ci`       | (none)       | —           | ❌ no              |
| `build`    | (none)       | —           | ❌ no              |
| `style`    | (none)       | —           | ❌ no              |
| `revert`   | matches the reverted commit's kind | matches | ✅ yes if the original required one |

Breaking changes — indicated either by the `!` suffix (`feat!:`, `fix!:`) or by a `BREAKING CHANGE:` footer — map to the Changie kind `Changed` (major) or `Removed` (major) depending on whether the API is altered or deleted. A `Deprecated` marker belongs on the commit that *adds* `#[deprecated]`, not on the later `Removed` one.

The `Security` Changie kind has no direct Conventional Commits equivalent: use `fix(security):` or `fix!:` and pick `Security` when authoring the fragment. This is intentional — security fixes want a dedicated changelog bucket even when the git type is a generic `fix`.

The `changelog-check` CI job enforces that any commit whose CC type appears with ✅ in the table above ships with a Changie fragment under `.changes/unreleased/*.yaml`. Use the `skip-changelog` PR label to bypass it for exceptional cases (e.g. a `fix:` that is purely internal and has no user-visible effect).

## Pre-commit hook

A git pre-commit hook is wired via [`cargo-husky`](https://github.com/rhysd/cargo-husky) (dev-dependency). The hook source lives in `.cargo-husky/hooks/pre-commit` and is copied to `.git/hooks/pre-commit` the first time a contributor runs `cargo test`. It runs — in order — `cargo fmt --check`, `cargo clippy -- -D warnings`, then `cargo test` across default / no-default / all-features, all with `--locked`.

Bypass only in emergencies with `git commit --no-verify`. If you modify the hook, update it in `.cargo-husky/hooks/` — edits to `.git/hooks/` are local and get overwritten.

## Architecture

Flat module layout, all under `src/`:

- `lib.rs` — sets `#![forbid(unsafe_code)]`, declares the three modules, and re-exports `SecretError`, `ExposeSecret`, `SecretString`, `SecretBytes` at the crate root. The crate-level doc example must compile (`cargo doc`).
- `error.rs` — `SecretError` enum (`thiserror`-derived).
- `traits.rs` — the `ExposeSecret` trait with an associated `type Value: ?Sized`. This is the **only** sanctioned path from a secret to its raw value; its name is deliberately verbose so code review can spot it.
- `types.rs` — `SecretString` and `SecretBytes` live in the same file and share a single private `constant_time_eq` helper at the bottom. Inline `#[cfg(test)] mod tests` covers both types plus `#[cfg(feature = "serde")]` JSON round-trip tests.

### Invariants the types must preserve

These are load-bearing — do not weaken them without a deliberate, reviewed change:

- `Display` always prints `**********`; `Debug` prints `SecretString([REDACTED])` or `SecretBytes([REDACTED], len=N)`. Never leak the value through a formatter.
- Memory is zeroised on drop via `#[derive(Zeroize, ZeroizeOnDrop)]`.
- `PartialEq` compares in constant time via the private `constant_time_eq` (implemented by hand, no extra dependency).
- **No `Serialize` impl** — secrets are deserialize-only. Adding `Serialize` is a breaking contract violation.
- **No `Clone` impl** — each copy widens the attack surface. If you think you need `Clone`, you don't.
- `serde::Deserialize` is gated behind the `serde` feature (enabled by default).
- `len()` / `is_empty()` are exposed because they don't leak the value.

## Project conventions (strict)

These are project-wide rules, not style preferences:

- **No `as` in `use` statements** — never `use foo::Bar as Baz;`.
- **No free functions in the public API** — always methods on a struct or trait. The private `constant_time_eq` helper in `types.rs` is the single intentional exception.
- **No `unwrap()`** in production code (tests may use `expect("…")` with a message).
- **No `unsafe`** — enforced by `#![forbid(unsafe_code)]` at the top of `lib.rs`.
- Public types are always explicitly typed (no inferred public signatures).
- `edition = "2024"`, MSRV `1.85` (declared in `Cargo.toml`).
