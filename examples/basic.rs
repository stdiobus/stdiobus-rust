// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Basic example of using stdio_bus Rust SDK
//!
//! Run with: cargo run --example basic

use stdiobus_client::{StdioBus, BackendMode};
use std::time::Duration;

#[tokio::main]
async fn main() -> stdiobus_core::Result<()> {
    println!("stdio_bus Rust SDK Example");
    println!("==========================\n");

    // Create a bus instance with Docker backend
    let bus = StdioBus::builder()
        .config(stdiobus_core::BusConfig {
            pools: vec![stdiobus_core::PoolConfig {
                id: "echo".into(),
                command: "node".into(),
                args: vec!["./examples/echo-worker.js".into()],
                instances: 1,
            }],
            limits: None,
        })
        .backend(BackendMode::Docker)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Bus created with {} backend", bus.backend_type());
    println!("State: {:?}", bus.state());

    // Start the bus (spawns workers)
    println!("\nStarting bus...");
    bus.start().await?;
    println!("State: {:?}", bus.state());

    // Send a request
    println!("\nSending tools/list request...");
    match bus.request("tools/list", serde_json::json!({})).await {
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
