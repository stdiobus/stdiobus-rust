<h1 align="center" style="font-weight:500"><strong>stdio Bus Rust SDK for AI Agent Transport</strong></h1>

<p align="center">An async Rust SDK for <a href="https://stdiobus.com" target="_blank">stdio Bus</a> — a process-embedded message bus for AI agent orchestration. Manages child worker processes communicating over stdin/stdout using JSON-RPC (MCP/ACP protocols).</p>

<p align="center">
  <a href="https://crates.io/crates/stdiobus"><img src="https://img.shields.io/crates/v/stdiobus?style=for-the-badge&logo=rust&logoColor=white&color=orange" alt="Crates.io"></a>
  <a href="https://modelcontextprotocol.io"><img src="https://img.shields.io/badge/protocol-MCP-purple?style=for-the-badge&logo=jsonwebtokens" alt="MCP"></a>
  <a href="https://agentclientprotocol.com"><img src="https://img.shields.io/badge/protocol-ACP-purple?style=for-the-badge&logo=jsonwebtokens" alt="ACP"></a>
  <a href="https://github.com/stdiobus"><img src="https://img.shields.io/badge/ecosystem-stdio%20Bus-ff4500?style=for-the-badge" alt="stdioBus"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.70%2B-000000?style=for-the-badge&logo=rust" alt="Rust"></a>
  <a href="https://tokio.rs"><img src="https://img.shields.io/badge/async-Tokio-463e7c?style=for-the-badge" alt="Tokio"></a>
  <a href="https://github.com/stdiobus/stdiobus-rust"><img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey?style=for-the-badge&logo=linux" alt="Platform"></a>
  <a href="https://github.com/stdiobus/stdiobus-rust"><img src="https://img.shields.io/badge/arch-x86__64%20%7C%20arm64-blue?style=for-the-badge" alt="Architecture"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=for-the-badge&logo=opensourceinitiative" alt="License"></a>
  <a href="https://github.com/stdiobus/stdiobus-rust"><img src="https://img.shields.io/badge/tests-112%20passing-brightgreen?style=for-the-badge&logo=rust" alt="Tests"></a>
  <a href="https://github.com/stdiobus/stdiobus-rust"><img src="https://img.shields.io/badge/e2e-2%20passing-brightgreen?style=for-the-badge&logo=rust" alt="E2E"></a>
  <a href="https://github.com/stdiobus/stdiobus-rust"><img src="https://img.shields.io/badge/backend-Native%20FFI%20%7C%20Docker-orange?style=for-the-badge" alt="Backends"></a>
</p>

---

## Features

- **Async-first** — Built on Tokio for high-performance async I/O
- **Multiple backends** — Native (FFI to libstdio_bus) or Docker
- **Type-safe** — Full Rust type safety with proper error handling
- **Zero-copy where possible** — Efficient message handling
- **ACP streaming** — Automatic aggregation of `agent_message_chunk` notifications

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stdiobus-client = "1.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

For native backend (requires libstdio_bus):

```toml
[dependencies]
stdiobus-client = { version = "1.1", features = ["native"] }
```

## Quick Start

```rust
use stdiobus_client::{StdioBus, BusConfig, PoolConfig, Result};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = StdioBus::builder()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "echo".into(),
                command: "./examples/echo-worker/target/release/echo-worker".into(),
                args: vec![],
                instances: 1,
            }],
            limits: None,
        })
        .backend_native()
        .build()?;

    bus.start().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let result = bus.request("echo", json!({"message": "hello"})).await?;
    println!("Response: {}", result);

    bus.stop().await?;

    Ok(())
}
```

<details>
<summary>Verified output (from <code>cargo test --test readme_examples --features native</code>)</summary>

```
[INFO] Process manager created with 1 workers across 1 pools
[INFO] Router created
[INFO] Starting 1 workers for pool 'echo'
[INFO] [worker=0] Worker started (pool=echo, cmd=echo-worker)
[INFO] All 1 workers started successfully
[echo-worker-rs] Started, waiting for NDJSON messages on stdin...
Response: {"echo":{"message":"hello"},"method":"echo","timestamp":"..."}
[INFO] Stopping all workers
[echo-worker-rs] Received signal, shutting down...
[INFO] All workers stopped
```

</details>

## Real-World Usage (ACP Agent)

Full ACP protocol flow: initialize agent, create session, send prompt.
Requires an ACP-compatible worker (e.g., codex-acp) and appropriate credentials.

```rust
use stdiobus_client::{StdioBus, BusConfig, PoolConfig, Result, RequestOptions};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = StdioBus::builder()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "acp-worker".into(),
                command: "/path/to/acp-worker".into(),
                args: vec![],
                instances: 1,
            }],
            limits: None,
        })
        .backend_native()
        .timeout(Duration::from_secs(60))
        .build()?;

    bus.start().await?;

    // 1. Initialize agent
    let opts = RequestOptions::default().agent_id("my-agent");
    let init = bus.request_with_options("initialize", json!({
        "protocolVersion": 1,
        "clientInfo": {"name": "my-app", "version": "1.0.0"},
        "clientCapabilities": {}
    }), opts).await?;
    println!("Agent: {:?}", init.get("agentInfo"));

    // 2. Create session
    let opts = RequestOptions::default().agent_id("my-agent");
    let session = bus.request_with_options("session/new", json!({
        "cwd": std::env::current_dir()?.to_string_lossy(),
        "mcpServers": []
    }), opts).await?;
    let session_id = session["sessionId"].as_str().unwrap();

    // 3. Send prompt
    let opts = RequestOptions::default().agent_id("my-agent");
    let result = bus.request_with_options("session/prompt", json!({
        "sessionId": session_id,
        "prompt": [{"type": "text", "text": "What is 2+2?"}]
    }), opts).await?;
    println!("Response: {:?}", result.get("text"));

    bus.stop().await?;
    Ok(())
}
```

## Configuration

Configuration is passed programmatically via `BusConfig`:

```rust
use stdiobus_client::{BusConfig, PoolConfig, LimitsConfig};

let config = BusConfig {
    pools: vec![PoolConfig {
        id: "worker".into(),
        command: "/path/to/worker-binary".into(),
        args: vec![],
        instances: 4,
    }],
    limits: Some(LimitsConfig {
        max_input_buffer: Some(2097152),
        max_restarts: Some(10),
        ..Default::default()
    }),
};
```

File-based config is also supported for backward compatibility:

```rust
let bus = StdioBus::builder()
    .config_path("./config.json")
    .build()?;
```

`.config()` and `.config_path()` are mutually exclusive.

## Backend Selection

```rust
use stdiobus_client::{StdioBus, BusConfig, PoolConfig};

// Auto (default): native on Unix, docker on Windows
let bus = StdioBus::builder()
    .config(BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "/path/to/worker".into(), args: vec![], instances: 2 }],
        limits: None,
    })
    .backend_auto()
    .build()?;

// Force native backend
let bus = StdioBus::builder()
    .config(BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "/path/to/worker".into(), args: vec![], instances: 2 }],
        limits: None,
    })
    .backend_native()
    .build()?;

// Force Docker backend
let bus = StdioBus::builder()
    .config(BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "/path/to/worker".into(), args: vec![], instances: 2 }],
        limits: None,
    })
    .backend_docker()
    .docker_image("stdiobus/stdiobus:node")
    .build()?;
```

## API Reference

### StdioBus

```rust
// Lifecycle
bus.start().await?;
bus.stop().await?;
bus.stop_with_timeout(30).await?;

// Messaging
let result = bus.request("method", params).await?;
let result = bus.request_with_options("method", params, options).await?;
bus.notify("method", params).await?;
bus.send(raw_json).await?;

// State
bus.state();           // BusState
bus.is_running();      // bool
bus.stats();           // BusStats
bus.worker_count();    // i32
bus.client_count();    // i32
bus.backend_type();    // &str
```

### RequestOptions

```rust
use stdiobus_client::RequestOptions;
use std::time::Duration;

let options = RequestOptions::with_timeout(Duration::from_secs(60))
    .session_id("my-session")
    .idempotency_key("unique-key")
    .require_extension("identity");
```

### Error Handling

```rust
use stdiobus_client::{Error, ErrorCode};

match bus.request("method", params).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(Error::Timeout { timeout_ms }) => {
        println!("Request timed out after {}ms", timeout_ms);
    }
    Err(Error::PolicyDenied { message }) => {
        println!("Policy denied: {}", message);
    }
    Err(e) => {
        println!("Error ({}): {}", e.code(), e);
        if e.is_retryable() {
            // Retry logic
        }
    }
}
```

## Crate Structure

| Crate | Description |
|-------|-------------|
| `stdiobus-client` | Main client API (use this) |
| `stdiobus-core` | Core types and error definitions |
| `stdiobus-ffi` | Raw FFI bindings to libstdio_bus |
| `stdiobus-backend-docker` | Docker backend implementation |
| `stdiobus-backend-native` | Native FFI backend implementation |

## Building from Source

```bash
# Clone the repository
git clone https://github.com/stdiobus/stdiobus-rust
cd stdiobus-rust

# Build all crates
cargo build

# Build with native backend
cargo build --features native
```

## Testing

**Unit tests** are co-located with source code (`#[cfg(test)]`) and run with:

```bash
cargo test
```

**E2E tests** use the Rust echo-worker (`examples/echo-worker/`). Build it first, then run:

```bash
# Build the echo worker
cd examples/echo-worker && cargo build --release && cd ../..

# Run all tests with native backend
cargo test --features native
```

## Platform Support

| Platform | Docker Backend | Native Backend | Target Triple |
|----------|----------------|----------------|---------------|
| Linux x64 | ✓ | ✓ | `x86_64-unknown-linux-gnu` |
| Linux arm64 | ✓ | ✓ | `aarch64-unknown-linux-gnu` |
| macOS x64 | ✓ | ✓ | `x86_64-apple-darwin` |
| macOS arm64 | ✓ | ✓ | `aarch64-apple-darwin` |
| Windows x64 | ✓ | ✘ | — |

Native backend includes prebuilt `libstdio_bus.a` for supported targets. The correct library is selected automatically at build time.

**Linux requirements:** glibc 2.31+ (Ubuntu 20.04+, Debian 11+, RHEL 8+, Fedora 33+)

## License

Apache-2.0
