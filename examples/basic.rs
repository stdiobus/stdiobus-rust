// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Basic example of using stdio_bus Rust SDK with Rust echo-worker
//!
//! Prerequisites:
//!   cd examples/echo-worker && cargo build --release
//!
//! Run with:
//!   cargo run --example basic --features native
//!
//! Or with Docker backend (no build needed for worker):
//!   cargo run --example basic

use stdiobus_client::{StdioBus, BackendMode};
use std::time::Duration;

#[tokio::main]
async fn main() -> stdiobus_core::Result<()> {
    println!("stdio_bus Rust SDK Example");
    println!("==========================\n");

    // Resolve echo-worker binary path (Rust worker, no Node.js dependency)
    let echo_worker = std::env::current_dir()
        .unwrap()
        .join("examples/echo-worker/target/release/echo-worker");

    if !echo_worker.exists() {
        eprintln!("Echo worker not found at: {}", echo_worker.display());
        eprintln!("Build it first: cd examples/echo-worker && cargo build --release");
        std::process::exit(1);
    }

    let worker_path = echo_worker.to_string_lossy().to_string();

    // Create a bus instance
    let bus = StdioBus::builder()
        .config(stdiobus_core::BusConfig {
            pools: vec![stdiobus_core::PoolConfig {
                id: "echo".into(),
                command: worker_path,
                args: vec![],
                instances: 1,
            }],
            limits: None,
        })
        .backend(BackendMode::Auto)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Bus created with {} backend", bus.backend_type());
    println!("State: {:?}", bus.state());

    // Start the bus (spawns workers)
    println!("\nStarting bus...");
    bus.start().await?;
    println!("State: {:?}", bus.state());

    // Wait for worker to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send a request
    println!("\nSending echo request...");
    match bus.request("echo", serde_json::json!({"message": "hello from Rust"})).await {
        Ok(result) => println!("Response: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Get statistics
    let stats = bus.stats();
    println!("\nStatistics:");
    println!("  Messages in:  {}", stats.messages_in);
    println!("  Messages out: {}", stats.messages_out);
    println!("  Bytes in:     {}", stats.bytes_in);
    println!("  Bytes out:    {}", stats.bytes_out);

    // Stop the bus
    println!("\nStopping bus...");
    bus.stop().await?;
    println!("State: {:?}", bus.state());

    println!("\nDone!");
    Ok(())
}
