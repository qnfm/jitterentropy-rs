# jitterentropy-rs

A Rust-first rewrite scaffold of the CPU timing-jitter entropy collector design
used by the upstream Jitterentropy library.

This crate separates the collector, timer abstraction, health tests, memory
disturbance logic, conditioner, status reporting, and C ABI boundary into
focused modules so that each part can be reviewed and tested independently.

> **Security status:** this crate is not a certified replacement for the
> upstream C implementation. It must not be used as a production cryptographic
> entropy source until target-specific validation, raw-noise assessment,
> differential testing, fuzzing, and independent review have been completed.

## Goals

- Provide a Rust-native API for collecting timing-jitter-derived bytes.
- Keep unsafe code isolated to platform timer access and C ABI glue.
- Support deterministic test timers through a timer abstraction.
- Provide a callback timer path for embedded or platform-specific integrations.
- Preserve a C ABI compatibility layer for callers that need a C-facing API.
- Keep validation boundaries explicit rather than implying certification.

## Non-goals

- This is not a line-by-line translation of the upstream C implementation.
- This is not currently SP800-90B, FIPS, or NTG.1 validated.
- This does not claim bit-for-bit equivalence with upstream output behavior.
- This does not remove the need for per-platform timer and noise-source review.

## Crate layout

```text
src/
  lib.rs          Public exports and crate-level configuration
  collector.rs    Collector construction, startup tests, sampling, output
  timer.rs        Platform timer, callback timer, and Timer trait
  health.rs       Runtime health-test state
  memory.rs       Optional memory disturbance source
  conditioner.rs  Cryptographic conditioning interface
  status.rs       Status reporting structures
  flags.rs        Public configuration flags
  error.rs        Init/read error types
  ffi.rs          C ABI compatibility layer
```

## Features

The exact feature set may evolve, but the crate is structured around these
capabilities:

- `std`: enable standard-library integrations where available.
- `alloc`: enable heap-backed memory disturbance and owned output buffers.
- `ffi`: expose the C ABI compatibility functions.
- `serde`: enable serialization for status/reporting types.
- `force-fips`: make relevant health-test failures fatal during collection and
  startup paths used by tests or validation builds.

Check `Cargo.toml` for the authoritative feature list.

## Rust API example

```rust
use jitterentropy::{EntropyCollectorBuilder, Flags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = EntropyCollectorBuilder::new()
        .osr(1)
        .flags(Flags::empty())
        .build()?;

    let mut out = [0u8; 64];
    collector.fill_bytes(&mut out)?;

    println!("{:02x?}", out);
    Ok(())
}
```

## Callback timer example

A callback timer is useful for tests, embedded integrations, or platforms that
provide their own monotonic high-resolution timer.

```rust
use jitterentropy::{EntropyCollectorBuilder, Flags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = 0u64;
    let mut state = 0x1234_5678_9abc_def0u64;

    let mut collector = EntropyCollectorBuilder::new()
        .flags(Flags::DISABLE_MEMORY_ACCESS)
        .build_with_callback(move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            t = t.wrapping_add(37 + (state & 0x3f));
            t
        })?;

    let mut out = [0u8; 32];
    collector.fill_bytes(&mut out)?;

    Ok(())
}
```

The callback example uses a deterministic jitter-like sequence for demonstration
and testing. It is not a real entropy source.

## C ABI

When the `ffi` feature is enabled, the crate exposes a C-facing API layer in
`src/ffi.rs`.

The FFI boundary is intentionally kept separate from the safe Rust API. This
makes pointer validation, allocation ownership, status export, and read
semantics easier to audit.

Typical exported functions include:

- `jent_version`
- `jent_entropy_init`
- `jent_entropy_init_ex`
- `jent_entropy_collector_alloc`
- `jent_entropy_collector_free`
- `jent_read_entropy`
- `jent_read_entropy_safe`
- `jent_status`
- `jent_secure_memory_supported`

The C ABI should be fuzzed and reviewed before it is used by external callers.

## Build and test

```bash
cargo fmt --all
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Optional checks:

```bash
cargo test --no-default-features
cargo test --no-default-features --features alloc
cargo test --features serde
cargo build --release --all-features
```

## Validation requirements

Before production use, complete at least the following:

1. **Timer validation**
   - Confirm that the selected platform timer is monotonic.
   - Confirm that the timer has enough resolution and variation.
   - Review generated assembly for timer reads on each target architecture.

2. **Health-test parity**
   - Compare runtime health-test behavior with the intended upstream model.
   - Verify repetition count, adaptive proportion, lag, memory-related, and
     permanent-failure behavior.

3. **Raw-noise capture**
   - Add a controlled raw-noise capture mode.
   - Collect raw samples per target platform and configuration.
   - Evaluate the samples with appropriate SP800-90B tooling.

4. **Differential testing**
   - Compare initialization, status, error behavior, and read behavior against
     the upstream C implementation where behavior is intended to match.

5. **FFI testing**
   - Fuzz null pointers, invalid lengths, repeated frees, short buffers, and
     unusual allocation paths.
   - Verify that C callers cannot violate Rust ownership assumptions.

6. **Conditioner review**
   - Confirm the conditioning construction and domain separation.
   - If strict upstream compatibility is required, implement and test exact
     conditioner behavior against upstream vectors or differential traces.

7. **Independent audit**
   - Review unsafe code.
   - Review timer assumptions.
   - Review health-test thresholds.
   - Review generated code under release builds.

## Safety and security notes

Timing-jitter entropy collection is highly sensitive to compiler behavior,
microarchitecture, operating-system scheduling, virtualization, and target
hardware. Passing unit tests only shows that the crate API behaves as expected;
it does not prove that the output contains sufficient entropy.

Do not treat successful compilation or normal test success as cryptographic
validation.

## Development notes

Synthetic test timers should be deterministic but irregular enough to exercise
the startup and health-test paths without depending on live hardware timing.

Avoid using regular fixed-step counters in collector construction tests. Such
timers can correctly fail startup checks because they do not provide meaningful
variation.

## License

Set this to match the intended project license before publication.

```text
TODO: add license information
```

## Acknowledgment

This project is based on the design ideas of the upstream Jitterentropy CPU
timing-jitter entropy collector. It is an independent Rust rewrite scaffold and
does not inherit the upstream project's validation status.
