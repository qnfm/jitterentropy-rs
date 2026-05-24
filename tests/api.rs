use jitterentropy::{EntropyCollectorBuilder, Flags, Timer};

#[derive(Clone)]
struct TestTimer {
    t: u64,
    state: u64,
}

impl TestTimer {
    fn new() -> Self {
        Self {
            t: 0,
            state: 0x1234_5678_9abc_def0,
        }
    }
}

impl Timer for TestTimer {
    fn now(&mut self) -> Option<u64> {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.t = self.t.wrapping_add(37 + (self.state & 0x1f));
        Some(self.t)
    }
}

#[test]
fn version_is_upstream_compatible_shape() {
    assert!(jitterentropy::version() >= 3_070_100);
}

#[test]
fn status_json_is_available() {
    let ec = EntropyCollectorBuilder::new()
        .flags(Flags::DISABLE_MEMORY_ACCESS)
        .build_with_timer(TestTimer::new())
        .unwrap();

    let status = ec.status();
    assert_eq!(status.version, jitterentropy::version());
}
