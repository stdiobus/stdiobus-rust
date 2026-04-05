// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

#![cfg_attr(docsrs, feature(doc_cfg))]

//! # stdiobus
//!
//! AI agent transport layer - unified SDK for MCP/ACP protocols.
//!
//! This is the umbrella crate that re-exports everything you need.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use stdiobus::{StdioBus, Result, RequestOptions};
//! use serde_json::json;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let bus = StdioBus::builder()
//!         .config_path("./config.json")
//!         .backend_native()
//!         .timeout(Duration::from_secs(60))
//!         .build()?;
//!
//!     bus.start().await?;
//!
//!     let opts = RequestOptions::default().agent_id("my-agent");
//!     let result = bus.request_with_options("initialize", json!({
//!         "protocolVersion": 1,
//!         "clientInfo": {"name": "my-app", "version": "1.0.0"},
//!         "clientCapabilities": {}
//!     }), opts).await?;
//!
//!     bus.stop().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - `docker` (default) - Docker backend
//! - `native` - Native FFI backend (requires libstdio_bus.a)
//! - `full` - Both backends

// Re-export everything from stdiobus-client
pub use stdiobus_client::*;

// Re-export core types explicitly for convenience
pub use stdiobus_core::{
    Backend, BackendMode, BusMessage, BusState, BusStats,
    DockerOptions, Error, ErrorCode, Result,
};
