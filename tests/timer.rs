use jitterentropy::{EntropyCollectorBuilder, Flags};

#[test]
fn callback_timer_can_drive_collector() {
    let mut t = 0u64;
    let mut state = 0x6a09_e667_f3bc_c909u64;

    let mut ec = EntropyCollectorBuilder::new()
        .flags(Flags::DISABLE_MEMORY_ACCESS)
        .build_with_callback(move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            t = t.wrapping_add(37 + (state & 0x3f));
            t
        })
        .unwrap();

    let mut out = [0u8; 64];
    ec.fill_bytes(&mut out).unwrap();
    assert!(out.iter().any(|&b| b != 0));
}
