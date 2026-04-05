# stdiobus-backend-docker

[![Crates.io](https://img.shields.io/crates/v/stdiobus-backend-docker.svg)](https://crates.io/crates/stdiobus-backend-docker)
[![License](https://img.shields.io/crates/l/stdiobus-backend-docker)](LICENSE)

Docker backend for stdio_bus - containerized transport layer.

This crate provides a Docker-based backend that runs stdio_bus inside a container. It works on all platforms including Windows.

## Installation

```toml
[dependencies]
stdiobus-backend-docker = "1.0"
```

## Prerequisites

- Docker installed and running
- Access to Docker socket

## Usage

Most users should use `stdiobus-client` with the default `docker` feature:

```rust
use stdiobus_client::StdioBus;

let bus = StdioBus::builder()
    .config_path("./config.json")
    .backend_docker()
    .docker_image("stdiobus/stdiobus:node20")
    .build()?;
```

## Direct Usage

```rust
use stdiobus_backend_docker::DockerBackend;
use stdiobus_core::DockerOptions;

let backend = DockerBackend::new(
    "./config.json",
    DockerOptions {
        image: "stdiobus/stdiobus:node20".to_string(),
        ..Default::default()
    },
)?;

backend.start().await?;
```

## Platform Support

| Platform | Status |
|----------|--------|
| Linux x64/arm64 | ✓ |
| macOS x64/arm64 | ✓ |
| Windows x64 | ✓ |

## License

Apache-2.0
