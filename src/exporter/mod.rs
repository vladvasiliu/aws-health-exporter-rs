use prometheus::{gather, register_counter, Encoder, Opts, TextEncoder};

pub struct Exporter {}

impl Exporter {
    pub fn work() {
        // Create a counter
        let counter_opts = Opts::new("test_counter", "test counter help");
        let counter = register_counter!(counter_opts).unwrap();

        counter.inc();

        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
    }
}
