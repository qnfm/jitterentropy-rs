use jitterentropy::{EntropyCollectorBuilder, Flags};

fn main() {
    let mut ec = EntropyCollectorBuilder::new()
        .osr(1)
        .flags(Flags::empty())
        .build()
        .expect("collector should initialize");

    let bytes = ec.read_entropy(32).expect("entropy read should succeed");
    println!("{:02x?}", bytes);
}
