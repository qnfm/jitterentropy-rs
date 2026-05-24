# jitterentropy-rs

A Rust rewrite scaffold of the CPU timing-jitter entropy source originally implemented by `smuellerDD/jitterentropy-library`.

## Status

This crate is production-oriented **structure and API plumbing**, not a certified cryptographic replacement. It must not be marketed as SP800-90B, FIPS, or NTG.1 validated without fresh target-specific validation and audit.

## Comparison baseline

This archive was compared with `qnfm/jitterentropy-rs`. That repository is a useful scaffold with a `jent-core` / `jent-platform` / `jent-ffi` workspace split, `no_std` core, abstract timer, isolated platform timer, and explicit warnings that it is not security validated. This crate remains a single crate for easier vendoring, but incorporates the same separation principles internally.

See `COMPARE_QNFM.md` for the detailed comparison.

## Design goals

- Safe Rust public API.
- Small, auditable internal modules.
- Timer abstraction with default platform timer and callback timer support.
- Explicit unsafe boundary for timestamp intrinsics and C ABI only.
- Runtime health checks: RCT, APT, lag predictor, and memory repetition hook.
- SHAKE256 conditioner interface with room to replace by a faithful upstream XDRBG-256 port.
- C-compatible ABI mirroring the upstream public API names where practical.
- `no_std + alloc` direction, while current default non-x86 fallback requires `std`.

## Build

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

## Example

```rust
use jitterentropy::{EntropyCollectorBuilder, Flags};

let mut ec = EntropyCollectorBuilder::new()
    .osr(1)
    .flags(Flags::empty())
    .build()?;
let mut out = [0u8; 32];
ec.fill_bytes(&mut out)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For deterministic tests or embedded integration, use `build_with_callback` or `build_with_timer`.

## Validation

For entropy validation, capture raw timing samples with a dedicated instrumentation build and process them using the statistical workflow required for the deployment target. See `VALIDATION.md`.
