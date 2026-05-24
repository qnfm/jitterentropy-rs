# Validation plan

This rewrite is not considered production-valid until the following are complete:

1. API parity review against the exact target version of `jitterentropy.h`.
2. Differential error-code review against the upstream C API.
3. Platform timer validation on each target CPU/OS pair.
4. Raw-noise capture in initialization and runtime phases.
5. SP800-90B entropy assessment with the same timer path used in deployment.
6. Review and port of health-test thresholds against the upstream source and lab guidance.
7. Conditioning review: replace the SHAKE256 placeholder with a faithful XDRBG-256 port if exact 3.7.x behavior is required.
8. Fuzz FFI entry points with null, aliasing, and extreme-length inputs.
9. Compile core entropy path with controlled optimization strategy and inspect emitted code.
10. Independent cryptographic implementation audit.
11. Add cbindgen-generated headers for downstream C consumers.
12. Add raw-sample export utilities that are excluded from production builds.

## Non-claims

The crate does not claim that a 64-byte SHAKE256 block contains 512 bits of entropy. Conditioning cannot create more entropy than the measured raw source provides.
