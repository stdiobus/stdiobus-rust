// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Common types for stdio_bus

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Backend mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendMode {
    /// Auto-select: native on Unix (if available), docker otherwise
    #[default]
    Auto,
    /// Force native backend (requires libstdio_bus)
    Native,
    /// Force Docker backend
    Docker,
}

impl std::fmt::Display for BackendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Native => write!(f, "native"),
            Self::Docker => write!(f, "docker"),
        }
    }
}

/// Options for individual requests
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Session ID for routing
    pub session_id: Option<String>,
    /// Agent ID for routing to specific agent
    pub agent_id: Option<String>,
    /// Idempotency key for replay protection
    pub idempotency_key: Option<String>,
    /// Required extensions
    pub required_extensions: Vec<String>,
}

impl RequestOptions {
    /// Create new options with timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..Default::default()
        }
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set agent ID
    pub fn agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set idempotency key
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Add required extension
    pub fn require_extension(mut self, extension: impl Into<String>) -> Self {
        self.required_extensions.push(extension.into());
        self
    }
}

/// Identity extension data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Stable subject identifier
    #[serde(rename = "subjectId")]
    pub subject_id: String,
    /// Role in the current process
    pub role: String,
    /// Who made the assertion (self, bus, issuer:<id>)
    #[serde(rename = "assertedBy")]
    pub asserted_by: String,
}

impl Identity {
    /// Create a self-asserted identity
    pub fn self_asserted(subject_id: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            role: role.into(),
            asserted_by: "self".to_string(),
        }
    }
}

/// Extension negotiation data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Extensions {
    /// Requested/active extensions with versions
    #[serde(flatten)]
    pub extensions: HashMap<String, ExtensionInfo>,
}

/// Information about a single extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Extension version
    pub version: String,
    /// Whether extension is required
    #[serde(default)]
    pub required: bool,
    /// Whether extension is active (in response)
    #[serde(default)]
    pub active: bool,
}

impl ExtensionInfo {
    /// Create new extension info
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            required: false,
            active: false,
        }
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Docker backend options
#[derive(Debug, Clone)]
pub struct DockerOptions {
    /// Docker image to use
    pub image: String,
    /// Pull policy: never, if-missing, always
    pub pull_policy: String,
    /// Path to docker CLI
    pub engine_path: String,
    /// Container startup timeout
    pub startup_timeout: Duration,
    /// Container name prefix
    pub container_name_prefix: String,
    /// Extra docker run arguments
    pub extra_args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
}

impl Default for DockerOptions {
    fn default() -> Self {
        Self {
            image: "stdiobus/stdiobus:node20".to_string(),
            pull_policy: "if-missing".to_string(),
            engine_path: "docker".to_string(),
            startup_timeout: Duration::from_secs(15),
            container_name_prefix: "stdiobus".to_string(),
            extra_args: Vec::new(),
            env: HashMap::new(),
        }
    }
}

// ============================================================================
// Bus Configuration Types
// ============================================================================

/// Worker pool configuration.
///
/// Matches the C bus JSON schema: `pools[].{id, command, args, instances}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Unique pool identifier
    pub id: String,
    /// Executable path
    pub command: String,
    /// Command-line arguments (default: empty)
    #[serde(default)]
    pub args: Vec<String>,
    /// Number of worker instances (must be ≥ 1)
    pub instances: u32,
}

/// Operational limits.
///
/// All fields optional — C bus applies defaults for omitted values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Per-connection input buffer limit in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_input_buffer: Option<usize>,
    /// Per-connection output queue limit in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_queue: Option<usize>,
    /// Max restarts within time window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_restarts: Option<u32>,
    /// Restart counting window in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_window_sec: Option<u32>,
    /// Graceful shutdown timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drain_timeout_sec: Option<u32>,
    /// Backpressure timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backpressure_timeout_sec: Option<u32>,
}

/// stdio_bus JSON configuration.
///
/// Matches the C bus config schema exactly. This is the primary way to
/// configure the bus programmatically — no config.json file needed.
///
/// # Example
/// ```
/// use stdiobus_core::BusConfig;
///
/// let config = BusConfig {
///     pools: vec![stdiobus_core::PoolConfig {
///         id: "worker".into(),
///         command: "node".into(),
///         args: vec!["./worker.js".into()],
///         instances: 4,
///     }],
///     limits: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusConfig {
    /// Worker pool definitions (at least one required)
    pub pools: Vec<PoolConfig>,
    /// Operational limits (optional, C bus applies defaults)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<LimitsConfig>,
}

impl BusConfig {
    /// Validate the configuration.
    ///
    /// Returns `Ok(())` if valid, or an error describing the problem.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.pools.is_empty() {
            return Err("at least one pool is required".into());
        }
        for (i, pool) in self.pools.iter().enumerate() {
            if pool.id.is_empty() {
                return Err(format!("pool {} missing id", i));
            }
            if pool.command.is_empty() {
                return Err(format!("pool '{}' missing command", pool.id));
            }
            if pool.instances == 0 {
                return Err(format!("pool '{}' instances must be ≥ 1", pool.id));
            }
        }
        Ok(())
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

/// Configuration source for the bus.
///
/// Exactly one variant is used. The SDK resolves this to JSON
/// before passing to the C bus.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Path to a JSON config file on disk
    Path(String),
    /// Typed configuration object (primary, recommended)
    Config(BusConfig),
}
