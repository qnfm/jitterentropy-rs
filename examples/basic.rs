use jent_core::{JentCollector, Sha3_512Conditioner};
use jent_platform::RdtscTimer;

fn main() {
    let mut collector = JentCollector::new(RdtscTimer);
    let mut conditioner = Sha3_512Conditioner::new();
    let mut out = [0u8; 64];

    collector.fill_conditioned(&mut conditioner, &mut out).unwrap();
    println!("{:02x?}", out);
}
