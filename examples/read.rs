use jitterentropy::{EntropyCollectorBuilder, Flags};

fn main() {
    let mut ec = EntropyCollectorBuilder::new()
        .osr(1)
        .flags(Flags::empty())
        .build()
        .expect("collector should initialize");

    let mut bytes = [0u8; 32];
    ec.fill_bytes(&mut bytes)
        .expect("entropy read should succeed");
    println!("{:02x?}", bytes);
}
