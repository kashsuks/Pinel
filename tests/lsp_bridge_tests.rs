#[path = "../src/features/lsp.rs"]
mod lsp;

use lsp::LspBridge;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

fn unique_abs_path(ext: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_nanos();

    std::env::temp_dir().join(format!("pinel_lsp_test_{nanos}.{ext}"))
}

fn wait_for_updates(bridge: &LspBridge, timeout: Duration) -> Vec<lsp::DiagnosticUpdate> {
    let start = Instant::now();
    loop {
        let updates = bridge.drain_updates();
        if !updates.is_empty() {
            return updates;
        }

        if start.elapsed() >= timeout {
            return Vec::new();
        }

        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn close_document_emits_clear_diagnostics_update() {
    let bridge = LspBridge::new(None);
    let path = unique_abs_path("txt");

    bridge.close_document(path.clone());

    let updates = wait_for_updates(&bridge, Duration::from_millis(300));
    assert!(!updates.is_empty(), "expected at least one diagnostics update");

    let found = updates
        .into_iter()
        .find(|u| u.path == path)
        .expect("expected diagnostics update for closed file path");

    assert!(
        found.diagnostics.is_empty(),
        "close should clear diagnostics for the path"
    );
}

#[test]
fn open_document_with_unsupported_extension_emits_no_updates() {
    let bridge = LspBridge::new(None);
    let path = unique_abs_path("txt");

    bridge.open_document(path, "hello world".to_string());

    thread::sleep(Duration::from_millis(150));
    let updates = bridge.drain_updates();
    assert!(
        updates.is_empty(),
        "unsupported extensions should not produce diagnostics updates"
    );
}

#[test]
fn change_and_save_without_open_are_noops() {
    let bridge = LspBridge::new(None);
    let path = unique_abs_path("txt");

    bridge.change_document(path.clone(), "edited".to_string());
    bridge.save_document(path);

    thread::sleep(Duration::from_millis(100));
    let updates = bridge.drain_updates();
    assert!(updates.is_empty(), "no updates expected when document was never opened");
}
