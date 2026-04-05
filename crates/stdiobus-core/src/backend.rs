// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Backend trait for stdio_bus implementations

use crate::{BusState, BusStats, Result};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Message received from the bus
#[derive(Debug, Clone)]
pub struct BusMessage {
    pub json: String,
}

/// Backend trait that all implementations must satisfy
#[async_trait]
pub trait Backend: Send + Sync {
    /// Start the backend
    async fn start(&self) -> Result<()>;

    /// Stop the backend gracefully
    async fn stop(&self, timeout_secs: u32) -> Result<()>;

    /// Send a message to workers
    async fn send(&self, message: &str) -> Result<()>;

    /// Get current state
    fn state(&self) -> BusState;

    /// Get statistics
    fn stats(&self) -> BusStats;

    /// Get number of running workers (-1 if unknown)
    fn worker_count(&self) -> i32;

    /// Get number of connected clients (-1 if unknown)
    fn client_count(&self) -> i32;

    /// Subscribe to incoming messages.
    ///
    /// Returns `Some(Receiver)` on the first call. Subsequent calls return `None`
    /// because the receiver can only have one owner. To share messages across
    /// multiple consumers, use a broadcast channel on top of the returned receiver.
    fn subscribe(&self) -> Option<mpsc::Receiver<BusMessage>>;

    /// Get backend type name
    fn backend_type(&self) -> &'static str;
}
