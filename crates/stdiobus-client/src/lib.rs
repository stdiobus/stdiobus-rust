// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

#![cfg_attr(docsrs, feature(doc_cfg))]

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
//!         .config(stdiobus_core::BusConfig {
//!             pools: vec![stdiobus_core::PoolConfig {
//!                 id: "worker".into(),
//!                 command: "node".into(),
//!                 args: vec!["./worker.js".into()],
//!                 instances: 2,
//!             }],
//!             limits: None,
//!         })
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
    Backend, BackendMode, BusConfig, BusMessage, BusState, BusStats, ConfigSource, Error,
    ErrorCode, Extensions, Identity, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse,
    LimitsConfig, PoolConfig, RequestOptions, Result,
};
