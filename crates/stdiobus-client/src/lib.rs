// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Async client for stdio_bus - AI agent transport layer
//!
//! # Example
//!
//! ```rust,no_run
//! use stdiobus_client::StdioBus;
//!
//! #[tokio::main]
//! async fn main() -> stdiobus_core::Result<()> {
//!     let bus = StdioBus::builder()
//!         .config_path("./config.json")
//!         .build()?;
//!
//!     bus.start().await?;
//!
//!     let result = bus.request("tools/list", serde_json::json!({})).await?;
//!     println!("Tools: {:?}", result);
//!
//!     bus.stop().await?;
//!     Ok(())
//! }
//! ```

mod backend;
mod builder;
mod client;

#[cfg(test)]
mod tests;

pub use builder::StdioBusBuilder;
pub use client::StdioBus;

// Re-export core types
pub use stdiobus_core::{
    Backend, BackendMode, BusMessage, BusState, BusStats, Error, ErrorCode, Extensions, Identity,
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestOptions, Result,
};
