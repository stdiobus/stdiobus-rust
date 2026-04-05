# stdiobus-core

[![Crates.io](https://img.shields.io/crates/v/stdiobus-core?style=for-the-badge&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/stdiobus-core)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue?style=for-the-badge&logo=apache)](LICENSE)

Core types and protocol models for stdio_bus - the AI agent transport layer.

This crate provides shared types, error definitions, and protocol models used by other stdiobus crates.

## Installation

```toml
[dependencies]
stdiobus-core = "1.0"
```

## Contents

- `Error` - Comprehensive error types with `is_retryable()` support
- `ErrorCode` - Error code enumeration matching C library codes
- `BusState` - Bus lifecycle states (Created, Starting, Running, Stopping, Stopped)
- `BusStats` - Runtime statistics
- `Message` - JSON-RPC message types
- `generate_client_session_id()` - Generate unique session IDs for routing

## Usage

Most users should use `stdiobus-client` instead of this crate directly.

```rust
use stdiobus_core::{Error, ErrorCode, generate_client_session_id};

// Generate a client session ID for routing
let session_id = generate_client_session_id();

// Check if an error is retryable
match some_operation() {
    Err(e) if e.is_retryable() => {
        // Retry the operation
    }
    Err(e) => return Err(e),
    Ok(v) => v,
}
```

## License

Apache-2.0
