// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Runtime statistics for stdio_bus

use serde::{Deserialize, Serialize};

/// Runtime statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BusStats {
    /// Messages sent to workers
    pub messages_in: u64,
    /// Messages received from workers
    pub messages_out: u64,
    /// Total bytes sent
    pub bytes_in: u64,
    /// Total bytes received
    pub bytes_out: u64,
    /// Number of worker restarts
    pub worker_restarts: u64,
    /// Messages that couldn't be routed
    pub routing_errors: u64,
    /// Client connections (TCP/Unix modes)
    pub client_connects: u64,
    /// Client disconnections
    pub client_disconnects: u64,
}

impl BusStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all counters to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
