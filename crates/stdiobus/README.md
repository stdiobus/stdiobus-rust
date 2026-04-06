# stdiobus

[![Crates.io](https://img.shields.io/crates/v/stdiobus?style=for-the-badge&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/stdiobus-client)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue?style=for-the-badge&logo=apache)](LICENSE)

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
use stdiobus::{StdioBus, BusConfig, PoolConfig, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = StdioBus::builder()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "worker".into(),
                command: "node".into(),
                args: vec!["./worker.js".into()],
                instances: 4,
            }],
            limits: None,
        })
        .backend_native()
        .build()?;

    bus.start().await?;

    let result = bus.request("tools/list", json!({})).await?;

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
