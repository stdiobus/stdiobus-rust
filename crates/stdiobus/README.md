# stdiobus

[![Crates.io](https://img.shields.io/crates/v/stdiobus.svg)](https://crates.io/crates/stdiobus)
[![License](https://img.shields.io/crates/l/stdiobus)](LICENSE)

AI agent transport layer - unified SDK for MCP/ACP protocols.

## Installation

```toml
[dependencies]
stdiobus = "1.0"
tokio = { version = "1", features = ["full"] }
```

For native backend:

```toml
[dependencies]
stdiobus = { version = "1.0", features = ["native"] }
```

## Usage

```rust
use stdiobus::{StdioBus, Result, RequestOptions};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = StdioBus::builder()
        .config_path("./config.json")
        .backend_native()
        .timeout(Duration::from_secs(60))
        .build()?;

    bus.start().await?;

    let opts = RequestOptions::default().agent_id("my-agent");
    let result = bus.request_with_options("initialize", json!({
        "protocolVersion": 1,
        "clientInfo": {"name": "my-app", "version": "1.0.0"},
        "clientCapabilities": {}
    }), opts).await?;

    bus.stop().await?;
    Ok(())
}
```

## Crate Structure

This is an umbrella crate. For granular control, use individual crates:

| Crate | Description |
|-------|-------------|
| `stdiobus` | This crate - re-exports everything |
| `stdiobus-client` | Client API |
| `stdiobus-core` | Core types |
| `stdiobus-backend-docker` | Docker backend |
| `stdiobus-backend-native` | Native FFI backend |

## License

Apache-2.0
