use rrflow_core::digest;
use rrflow_db::LocalObjectStore;
use std::io::Write;
use std::sync::{Arc, Mutex};

struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn object_repair_reports_lock_wait_without_payload_or_digest() {
    let directory = tempfile::tempdir().unwrap();
    let store = LocalObjectStore::open(directory.path()).unwrap();
    let original = store.put(b"trace sentinel").unwrap();
    std::fs::write(
        store.root().join(&original.receipt.key),
        b"corrupt sentinel",
    )
    .unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::clone(&captured);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || Capture(Arc::clone(&writer)))
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        store.put(b"trace sentinel").unwrap();
    });
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert!(output.contains("object repair lock wait started"));
    assert!(output.contains("object repair lock acquired"));
    assert!(output.contains("elapsed_micros"));
    assert!(!output.contains(&digest::sha256_hex(b"trace sentinel")));
    assert!(!output.contains("trace sentinel"));
    assert!(!output.contains("corrupt sentinel"));
    assert!(!output.contains(&store.root().display().to_string()));
}
