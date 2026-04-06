// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

#![cfg_attr(docsrs, feature(doc_cfg))]

//! Core types and protocol models for stdio_bus
//!
//! This crate provides the fundamental types used across all stdio_bus components:
//! - Error types with canonical error codes
//! - Bus state machine
//! - JSON-RPC message types
//! - Statistics and configuration types
//! - Backend trait for implementations

mod backend;
mod error;
mod message;
mod state;
mod stats;
mod types;

#[cfg(test)]
mod tests;

pub use backend::{Backend, BusMessage};
pub use error::{Error, ErrorCode, Result};
pub use message::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError};
pub use state::BusState;
pub use stats::BusStats;
pub use types::{BackendMode, RequestOptions, Identity, Extensions, ExtensionInfo, DockerOptions, PoolConfig, LimitsConfig, BusConfig, ConfigSource};

use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique client session ID for stdiobus routing.
///
/// Format: `client-{timestamp_ms}-{random_suffix}`
pub fn generate_client_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    
    // Use uuid for random suffix instead of rand crate
    let uuid = uuid::Uuid::new_v4();
    let suffix = &uuid.to_string()[0..6];
    
    format!("client-{}-{}", timestamp, suffix)
}
