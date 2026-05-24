use jitterentropy::{EntropyCollectorBuilder, Flags};

#[test]
fn callback_timer_can_drive_collector() {
    let mut counter = 0u64;
    let mut ec = EntropyCollectorBuilder::new()
        .flags(Flags::DISABLE_MEMORY_ACCESS)
        .build_with_callback(move || {
            counter = counter.wrapping_add(37 + (counter & 15));
            counter
        })
        .unwrap();
    let mut out = [0u8; 64];
    ec.fill_bytes(&mut out).unwrap();
    assert!(out.iter().any(|&b| b != 0));
}
