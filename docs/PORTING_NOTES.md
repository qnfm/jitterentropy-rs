# Porting notes

This scaffold intentionally does not claim entropy correctness.

Immediate next steps:

1. Port upstream health tests exactly.
2. Port upstream oversampling-rate behavior.
3. Port startup timer validation.
4. Add raw entropy capture utilities.
5. Differential-test against upstream C implementation.
6. Add per-platform timers beyond x86/x86_64 RDTSC.
7. Add FFI header generation with cbindgen.

Conditioner policy:

- SHA3-512 produces fixed 64-byte conditioned blocks.
- SHAKE256 is treated as an XOF and currently configured to produce 64-byte blocks in the FFI layer.
- Entropy accounting must not assume that longer SHAKE256 output creates more entropy than the raw source provides.
