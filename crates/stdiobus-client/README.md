# stdiobus-client

[![Crates.io](https://img.shields.io/crates/v/stdiobus-client.svg)](https://crates.io/crates/stdiobus-client)
[![License](https://img.shields.io/crates/l/stdiobus-client)](LICENSE)

Async client for stdio_bus - the AI agent transport layer for MCP/ACP protocols.

This is the main crate you should use. It provides a high-level async API for communicating with stdio_bus workers.

## Installation

```toml
[dependencies]
stdiobus-client = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use stdiobus_client::{StdioBus, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = StdioBus::builder()
        .config_path("./config.json")
        .build()?;

    bus.start().await?;

    let result = bus.request("tools/list", json!({})).await?;
    println!("Tools: {:?}", result);

    bus.stop().await?;
    Ok(())
}
```

## Features

- `docker` (default) - Docker backend support
- `native` - Native FFI backend (requires libstdio_bus)

## Documentation

See the [main repository README](https://github.com/stdiobus/stdiobus-rust) for full documentation.

## License

Apache-2.0
