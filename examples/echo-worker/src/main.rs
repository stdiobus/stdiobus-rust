// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors
//
//! Minimal NDJSON echo worker for testing stdio Bus Rust SDK.
//!
//! Receives JSON-RPC requests on stdin, echoes params back as result on stdout.
//! Cross-platform: Linux, macOS, Windows.
//!
//! Build: cargo build --example echo-worker
//! Run:   cargo run --example echo-worker

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let shutting_down = Arc::new(AtomicBool::new(false));

    // Handle SIGTERM/SIGINT for graceful shutdown
    #[cfg(unix)]
    {
        let flag = shutting_down.clone();
        unsafe {
            libc::signal(libc::SIGTERM, handle_signal as *const () as libc::sighandler_t);
            libc::signal(libc::SIGINT, handle_signal as *const () as libc::sighandler_t);
        }
        SHUTDOWN_FLAG.store(flag.as_ref() as *const AtomicBool as usize, Ordering::SeqCst);
    }

    #[cfg(windows)]
    {
        let flag = shutting_down.clone();
        ctrlc_handler(flag);
    }

    eprintln!("[echo-worker-rs] Started, waiting for NDJSON messages on stdin...");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        if shutting_down.load(Ordering::Relaxed) {
            break;
        }

        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[echo-worker-rs] Parse error: {}", e);
                continue;
            }
        };

        let id = msg.get("id");
        let method = msg.get("method").and_then(|m| m.as_str());

        match (id, method) {
            // Request: has id + method → send response
            (Some(id), Some(method_name)) => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                let mut response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "echo": params,
                        "method": method_name,
                        "timestamp": timestamp_now()
                    }
                });

                // Preserve sessionId for routing
                if let Some(session_id) = msg.get("sessionId") {
                    response["sessionId"] = session_id.clone();
                }

                writeln!(stdout_lock, "{}", response).ok();
                stdout_lock.flush().ok();
            }
            // Notification: method but no id → optional notification back
            (None, Some(_method_name)) => {
                if let Some(session_id) = msg.get("sessionId") {
                    let notification = json!({
                        "jsonrpc": "2.0",
                        "method": "echo.notification",
                        "params": {
                            "original": _method_name,
                            "timestamp": timestamp_now()
                        },
                        "sessionId": session_id
                    });
                    writeln!(stdout_lock, "{}", notification).ok();
                    stdout_lock.flush().ok();
                }
            }
            // Ignore anything else
            _ => {}
        }
    }

    eprintln!("[echo-worker-rs] stdin closed, exiting");
}

fn timestamp_now() -> String {
    // Simple ISO-ish timestamp without external deps
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}Z", secs)
}

// --- Unix signal handling ---

#[cfg(unix)]
static SHUTDOWN_FLAG: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(unix)]
extern "C" fn handle_signal(_sig: libc::c_int) {
    let ptr = SHUTDOWN_FLAG.load(Ordering::SeqCst);
    if ptr != 0 {
        let flag = unsafe { &*(ptr as *const AtomicBool) };
        flag.store(true, Ordering::SeqCst);
    }
    // Write to stderr from signal handler (async-signal-safe on most platforms)
    let msg = b"[echo-worker-rs] Received signal, shutting down...\n";
    unsafe { libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len()) };
}

// --- Windows Ctrl+C handling ---

#[cfg(windows)]
fn ctrlc_handler(flag: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        // On Windows, we rely on stdin closing when parent terminates.
        // This is a fallback — the main loop exits on stdin EOF anyway.
        let _ = flag;
    });
}
