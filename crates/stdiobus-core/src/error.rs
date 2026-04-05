// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Error types for stdio_bus
//!
//! Canonical error codes matching spec/host-api.md

use thiserror::Error;

/// Canonical error codes for stdio_bus operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ErrorCode {
    /// Invalid argument provided
    InvalidArgument = -1,
    /// Operation not valid in current state
    InvalidState = -2,
    /// Request timed out
    Timeout = -3,
    /// Request was cancelled
    Cancelled = -4,
    /// Transport-level failure
    TransportError = -5,
    /// Protocol negotiation failed
    NegotiationFailed = -6,
    /// Required extension not available
    ExtensionUnavailable = -7,
    /// Operation denied by policy
    PolicyDenied = -8,
    /// Internal error
    InternalError = -9,
    /// Resource exhausted
    ResourceExhausted = -10,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument => write!(f, "INVALID_ARGUMENT"),
            Self::InvalidState => write!(f, "INVALID_STATE"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::TransportError => write!(f, "TRANSPORT_ERROR"),
            Self::NegotiationFailed => write!(f, "NEGOTIATION_FAILED"),
            Self::ExtensionUnavailable => write!(f, "EXTENSION_UNAVAILABLE"),
            Self::PolicyDenied => write!(f, "POLICY_DENIED"),
            Self::InternalError => write!(f, "INTERNAL_ERROR"),
            Self::ResourceExhausted => write!(f, "RESOURCE_EXHAUSTED"),
        }
    }
}

/// Main error type for stdio_bus operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("Invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Request cancelled")]
    Cancelled,

    #[error("Transport error: {message}")]
    TransportError { message: String },

    #[error("Negotiation failed: {message}")]
    NegotiationFailed { message: String },

    #[error("Extension unavailable: {extension}")]
    ExtensionUnavailable { extension: String },

    #[error("Policy denied: {message}")]
    PolicyDenied { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Get the canonical error code
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidArgument { .. } => ErrorCode::InvalidArgument,
            Self::InvalidState { .. } => ErrorCode::InvalidState,
            Self::Timeout { .. } => ErrorCode::Timeout,
            Self::Cancelled => ErrorCode::Cancelled,
            Self::TransportError { .. } => ErrorCode::TransportError,
            Self::NegotiationFailed { .. } => ErrorCode::NegotiationFailed,
            Self::ExtensionUnavailable { .. } => ErrorCode::ExtensionUnavailable,
            Self::PolicyDenied { .. } => ErrorCode::PolicyDenied,
            Self::InternalError { .. } => ErrorCode::InternalError,
            Self::ResourceExhausted { .. } => ErrorCode::ResourceExhausted,
            Self::Json(_) => ErrorCode::InvalidArgument,
            Self::Io(_) => ErrorCode::TransportError,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.code(),
            ErrorCode::Timeout | ErrorCode::TransportError | ErrorCode::ResourceExhausted
        )
    }
}

/// Result type alias for stdio_bus operations
pub type Result<T> = std::result::Result<T, Error>;
