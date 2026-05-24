use jitterentropy::{EntropyCollector, Flags};

#[test]
fn version_is_upstream_compatible_shape() {
    assert!(jitterentropy::version() >= 3_070_100);
}

#[test]
fn status_json_is_available() {
    let ec = EntropyCollector::new(1, Flags::DISABLE_MEMORY_ACCESS).unwrap();
    let status = ec.status();
    assert_eq!(status.version, jitterentropy::version());
}
