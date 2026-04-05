// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Bus state machine

use std::fmt;

/// State of the stdio_bus instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BusState {
    /// Created but not started
    Created = 0,
    /// Workers being spawned
    Starting = 1,
    /// Running and accepting messages
    Running = 2,
    /// Graceful shutdown in progress
    Stopping = 3,
    /// Fully stopped
    Stopped = 4,
}

impl BusState {
    /// Check if the bus is in a state that accepts messages
    pub fn accepts_messages(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the bus can be started
    pub fn can_start(&self) -> bool {
        matches!(self, Self::Created | Self::Stopped)
    }

    /// Check if the bus can be stopped
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Starting)
    }
}

impl fmt::Display for BusState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Starting => write!(f, "STARTING"),
            Self::Running => write!(f, "RUNNING"),
            Self::Stopping => write!(f, "STOPPING"),
            Self::Stopped => write!(f, "STOPPED"),
        }
    }
}

impl TryFrom<u8> for BusState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Created),
            1 => Ok(Self::Starting),
            2 => Ok(Self::Running),
            3 => Ok(Self::Stopping),
            4 => Ok(Self::Stopped),
            _ => Err(()),
        }
    }
}
