# stdiobus-backend-native

[![Crates.io](https://img.shields.io/crates/v/stdiobus-backend-native?style=for-the-badge&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/stdiobus-backend-native)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue?style=for-the-badge&logo=apache)](LICENSE)

Native FFI backend for stdio_bus - direct C library integration.

This crate provides a native backend that links directly to `libstdio_bus.a` via FFI. It offers the best performance but requires building the C library.

## Installation

```toml
[dependencies]
stdiobus-backend-native = "1.0"
```

## Prerequisites

Build `libstdio_bus.a` from the main repository:

```bash
git clone https://github.com/stdiobus/stdiobus-rust
cd stdiobus-rust
make lib

export STDIO_BUS_LIB_DIR=$(pwd)/build
```

## Usage

Most users should use `stdiobus-client` with the `native` feature:

```rust
use stdiobus_client::StdioBus;

let bus = StdioBus::builder()
    .config_path("./config.json")
    .backend_native()
    .build()?;
```

## Direct Usage

```rust
use stdiobus_backend_native::NativeBackend;

let backend = NativeBackend::new("./config.json")?;

backend.start().await?;
```

## Platform Support

| Platform | Status |
|----------|--------|
| Linux x64/arm64 | ✓ |
| macOS x64/arm64 | ✓ |
| Windows | ✘ (use Docker backend) |

## License

Apache-2.0
