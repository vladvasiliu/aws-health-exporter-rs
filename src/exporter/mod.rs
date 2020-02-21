use prometheus::{gather, Encoder, TextEncoder};

pub struct Exporter {}

impl Exporter {
    pub fn work() {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        println!("{}", String::from_utf8(buffer).unwrap());
    }
}
