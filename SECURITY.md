# Security Policy

`hosanna-rs-secret` exists to prevent accidental leakage of in-process secrets. A vulnerability in this crate is therefore structural, not cosmetic — please treat reports accordingly.

## Supported versions

Only the latest minor line receives security fixes. Pre-1.0 releases follow semantic versioning with the caveat that breaking changes can ship in any minor bump.

| Version | Supported          |
| ------- | ------------------ |
| `0.1.x` | ✅ security fixes  |
| `< 0.1` | ❌ no support      |

## What counts as a vulnerability

Please report privately if you can demonstrate any of the following:

- A path that reveals the raw secret value without going through `ExposeSecret::expose_secret()` — for example a `Display`, `Debug`, `serde::Serialize`, or other trait impl that leaks it.
- A timing side-channel in `PartialEq` / `Eq` that makes comparison time depend on the secret's contents (beyond the already-public length).
- A way to observe the backing buffer **after** the wrapper is dropped, indicating zeroisation has been skipped or circumvented.
- Any addition of `unsafe`, or any compile-time escape from `#![forbid(unsafe_code)]`.
- A `Clone`, `Serialize`, `AsRef<str>`, `Deref`, or similar trait implementation that widens the surface unintentionally.

Bugs that do **not** qualify as vulnerabilities (they are still welcome as public issues):

- The redacted `Display` / `Debug` output doesn't match the documented format verbatim.
- Compilation failure on a new toolchain.
- Broken intra-doc links, typos in rustdoc.
- Ergonomic gaps (missing `From` impl, missing convenience method).

## How to report

**Do not open a public GitHub issue for anything in the "vulnerability" list above.**

Preferred channel — a private GitHub Security Advisory:

1. Go to the repository's **Security** tab.
2. Click **Report a vulnerability**.
3. Fill in the form with a reproducer.

Alternative channel — email the maintainer listed in [`Cargo.toml`](Cargo.toml) under `authors`. Use the subject line `[security][hosanna-rs-secret]` so the message is easy to route.

Include, at minimum:

- The crate version and commit SHA you tested against.
- The Rust toolchain version (`rustc --version`).
- A minimal reproducer — ideally a `#[test]` that fails on main.
- Your assessment of the severity and of any partial mitigations.

If you want, tell us how you'd like to be credited in the advisory (name / handle / link, or anonymous).

## What to expect from us

- **Acknowledgement within 72 hours.** Weekends and public holidays may push this slightly; you will at least receive a "received, looking at it" message.
- **Triage within 7 days.** We confirm (or refute) the issue, agree on severity, and propose an embargo window.
- **Fix + coordinated disclosure.** We prefer to ship the patch release and the public advisory together. Default embargo is 30 days from the confirmed triage; we are happy to adjust based on severity and on the reporter's constraints.
- **CVE assignment.** For anything that genuinely breaks one of the documented invariants we request a CVE through GitHub's advisory tooling.

We will keep you informed at each step and credit you in the advisory and in the `CHANGELOG.md` entry unless you explicitly ask to remain anonymous.

## Scope

This policy covers the `hosanna-rs-secret` crate itself. Vulnerabilities in upstream dependencies (`zeroize`, `serde`, `thiserror`) should be reported to their respective projects; if the issue is exploitable *through* this crate in a way that would not exist in the dependency alone, please still let us know.
