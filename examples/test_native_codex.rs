// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Test Rust SDK with Native backend connecting to codex-acp agent
//!
//! This test demonstrates the SDK starting stdio_bus internally via Native backend
//! and communicating with the codex-acp agent.
//!
//! Run with: cargo run --example test_native_codex --features native

use stdiobus_client::StdioBus;
use std::time::Duration;

#[tokio::main]
async fn main() -> stdiobus_core::Result<()> {
    // Initialize tracing for logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n========================================");
    println!("Rust SDK E2E Test - Native Backend");
    println!("Testing: SDK starts stdio_bus internally");
    println!("========================================\n");

    // Create a bus instance with Native backend
    // The SDK will start stdio_bus internally using libstdio_bus.a
    println!("[1] Creating StdioBus with Native backend...");
    let bus = StdioBus::builder()
        .config_path("./examples/config.json")
        .backend_native()
        .timeout(Duration::from_secs(60))
        .build()?;

    println!("    Backend type: {}", bus.backend_type());
    println!("    State: {:?}", bus.state());

    // Start the bus - this spawns workers internally
    println!("\n[2] Starting bus (spawning workers)...");
    bus.start().await?;
    println!("    State: {:?}", bus.state());
    println!("    Workers: {}", bus.worker_count());

    // Wait for workers to initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send initialize request to codex-acp
    println!("\n[3] Sending initialize to codex-acp...");
    let init_params = serde_json::json!({
        "protocolVersion": 1,
        "clientInfo": {
            "name": "rust-sdk-native-test",
            "version": "1.0.0"
        },
        "clientCapabilities": {},
        "agentId": "codex-acp"
    });
    
    match bus.request("initialize", init_params).await {
        Ok(result) => {
            println!("    [PASS] Initialize response received");
            if let Some(agent_info) = result.get("agentInfo") {
                println!("    Agent: {} v{}", 
                    agent_info.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    agent_info.get("version").and_then(|v| v.as_str()).unwrap_or("unknown")
                );
            }
        }
        Err(e) => {
            println!("    [FAIL] Initialize error: {}", e);
        }
    }

    // Create session
    println!("\n[4] Creating session...");
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    let session_params = serde_json::json!({
        "cwd": cwd,
        "mcpServers": [],
        "agentId": "codex-acp"
    });
    
    let session_id = match bus.request("session/new", session_params).await {
        Ok(result) => {
            if let Some(id) = result.get("sessionId").and_then(|v| v.as_str()) {
                println!("    [PASS] Session created: {}", id);
                id.to_string()
            } else {
                println!("    [FAIL] No sessionId in response");
                String::new()
            }
        }
        Err(e) => {
            println!("    [FAIL] Session error: {}", e);
            String::new()
        }
    };

    // Send prompt
    if !session_id.is_empty() {
        println!("\n[5] Sending prompt: \"What is 2+2?\"...");
        let prompt_params = serde_json::json!({
            "sessionId": session_id,
            "prompt": [{
                "type": "text",
                "text": "What is 2+2? Answer with just the number."
            }],
            "agentId": "codex-acp"
        });
        
        match bus.request("session/prompt", prompt_params).await {
            Ok(result) => {
                if result.get("stopReason").and_then(|v| v.as_str()) == Some("end_turn") {
                    println!("    [PASS] Prompt completed");
                } else {
                    println!("    [INFO] Response: {}", result);
                }
            }
            Err(e) => {
                println!("    [FAIL] Prompt error: {}", e);
            }
        }
    }

    // Get statistics
    let stats = bus.stats();
    println!("\n[6] Statistics:");
    println!("    Messages in:  {}", stats.messages_in);
    println!("    Messages out: {}", stats.messages_out);
    println!("    Bytes in:     {}", stats.bytes_in);
    println!("    Bytes out:    {}", stats.bytes_out);

    // Stop the bus
    println!("\n[7] Stopping bus...");
    bus.stop().await?;
    println!("    State: {:?}", bus.state());

    println!("\n========================================");
    println!("Test completed!");
    println!("========================================\n");
    
    Ok(())
}
