# Contributing to hosanna-rs-secret

Thanks for taking the time to contribute. This crate is small on purpose — its whole value comes from the narrow guarantees it preserves — so contributions are held to a correspondingly strict bar. Reading this page before opening a PR will save everyone time.

## Before you start

- **Open an issue first** for anything larger than a typo or a missing doc link. The invariants are load-bearing, and some PRs are easier to reject than to review.
- **Security-sensitive reports** (timing side-channels, memory lifecycle, unintended leakage paths) should go through a private channel rather than a public issue. Open a GitHub security advisory or email the maintainer listed in `Cargo.toml`.
- **Scope discipline.** A bug fix fixes the bug; please do not bundle refactors, renames, or "while I'm here" cleanups. They make regressions hard to bisect.

## Development setup

Requirements:

- Rust `1.85+` (edition 2024 — declared as the MSRV in `Cargo.toml`).
- `cargo` (comes with rustup).

Clone and bootstrap:

```bash
git clone https://github.com/aartintelligent/hosanna-rs-secret.git
cd hosanna-rs-secret
cargo test
```

The first `cargo test` installs the git `pre-commit` hook from `.cargo-husky/hooks/pre-commit` into `.git/hooks/`. From that point on, every commit runs the same gates as CI.

## The local gates (what the hook runs)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo test --no-default-features --locked
cargo test --all-features --locked
```

A commit only passes once **all** of these succeed. In an emergency you can skip the hook with `git commit --no-verify`, but the CI gate will still block the PR — please use this only to unblock a WIP push, not to ship.

If you need to change the hook itself, edit `.cargo-husky/hooks/pre-commit` (the versioned copy). Edits to `.git/hooks/pre-commit` are local and get overwritten.

## Project conventions (hard rules)

These are not style preferences — PRs that break them will be asked to revise:

- **`#![forbid(unsafe_code)]`** at the crate root. No `unsafe`, anywhere.
- **No `as` in `use` statements.** If two names collide, rename at the definition site or wrap in a module.
- **No free functions in the public API.** Methods on structs or traits only. The one private `constant_time_eq` helper in `src/types.rs` is the single intentional exception.
- **No `unwrap()` in production code.** Tests may use `expect("message")` with a descriptive message.
- **Explicitly typed public signatures.** No reliance on inference for public items.
- **English comments and identifiers.** The rustdoc is the crate's user-facing surface; it stays in English.

## Invariants that must not regress

Any PR that changes `src/types.rs` must preserve every row of the public-contract table in `README.md`. In particular:

- `Display` prints the fixed redaction marker `**********`; `Debug` prints `SecretString([REDACTED])` or `SecretBytes([REDACTED], len=N)`.
- Equality is constant-time via the private `constant_time_eq` helper.
- The secret's backing buffer is zeroised on drop (`#[derive(Zeroize, ZeroizeOnDrop)]`).
- `Serialize` is **not** implemented. Do not add it, even "just for tests" — the presence of the impl is itself the leak.
- `Clone` is **not** implemented. If a caller needs a copy they can go through `ExposeSecret::expose_secret` and rebuild one deliberately.
- `serde::Deserialize` stays behind the `serde` feature flag.

If you have a concrete use-case that seems to require relaxing one of these, open an issue first — the answer is usually "a new type" rather than "weaken the existing one".

## Tests

- Unit tests live inline in `#[cfg(test)] mod tests` at the bottom of each module.
- Tests that exercise `serde` are gated with `#[cfg(feature = "serde")]`.
- New public behaviour needs a test. New redaction / zeroisation / constant-time behaviour needs a test that would have failed before the change.
- Rustdoc examples in doc-comments are executed by `cargo test --doc` — keep them honest (no ````ignore` to hide failures).

## Documentation

- The crate-level rustdoc in `src/lib.rs` is the canonical description of the crate. Update it whenever the public surface changes.
- `README.md` is the shop window; update the feature table and the contract table when you change behaviour.
- `CLAUDE.md` captures project-local context for AI-assisted editing; keep it in sync if you change conventions or tooling.

## Changelog

Changelog entries are authored with [Changie](https://github.com/miniscruff/changie):

```bash
changie new
```

This drops a fragment under `.changes/unreleased/`. Commit the fragment with your change. Do **not** edit `CHANGELOG.md` directly — it is regenerated at release time.

Pick the right kind:

| Kind         | Bumps    | Use for                                           |
| ------------ | -------- | ------------------------------------------------- |
| `Added`      | minor    | New public API, new feature flag                  |
| `Changed`    | major    | Breaking change to existing public API            |
| `Deprecated` | minor    | API marked `#[deprecated]` but still works        |
| `Removed`    | major    | API deletion                                      |
| `Fixed`      | patch    | Bug fixes that do not change the public contract  |
| `Security`   | patch    | Leak / side-channel fixes                         |

## Commit and pull-request etiquette

- **One logical change per PR.** Small PRs land faster and review more carefully.
- **Write commit messages in the imperative** (`Add …`, `Fix …`, `Refactor …`) and keep the subject under 70 characters. The body is the place for the *why*, not the *what*.
- **Rebase, don't merge**, when syncing with `main`. The history stays linear.
- **Mention the issue number** in the PR description when one exists.
- **CI must be green** before review. If a flaky test bites you, file an issue rather than re-running until it passes.

## License

The project is licensed under the [Apache License, Version 2.0](LICENSE). Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work — as defined in the Apache-2.0 license — shall be licensed as above, without any additional terms or conditions.
