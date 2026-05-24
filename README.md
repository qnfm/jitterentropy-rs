# jitterentropy-rs scaffold

Initial Rust scaffold for a Jitterentropy-style CPU timing jitter collector.

Status: **not security validated**. This is a porting scaffold only.

Design goals:

- `jent-core` is `no_std`.
- Timer source is abstracted behind `HighResTimer`.
- Conditioning supports SHA3-512 and SHAKE256.
- Platform-specific timer access is isolated in `jent-platform`.
- C ABI compatibility starts in `jent-ffi`.

Do not use this as a cryptographic RNG until the algorithm is fully ported and validated against the upstream implementation and relevant SP800-90B tests.
