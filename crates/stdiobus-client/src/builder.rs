// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Builder pattern for StdioBus client

use crate::client::StdioBus;
use stdiobus_core::{BackendMode, DockerOptions, Error, Result};
use std::time::Duration;

/// Builder for creating StdioBus instances
#[derive(Debug, Clone)]
pub struct StdioBusBuilder {
    config_path: Option<String>,
    backend: BackendMode,
    timeout: Duration,
    docker_options: Option<DockerOptions>,
}

impl Default for StdioBusBuilder {
    fn default() -> Self {
        Self {
            config_path: None,
            backend: BackendMode::Auto,
            timeout: Duration::from_secs(30),
            docker_options: None,
        }
    }
}

impl StdioBusBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration file path (required)
    pub fn config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set the backend mode
    pub fn backend(mut self, mode: BackendMode) -> Self {
        self.backend = mode;
        self
    }

    /// Use auto backend selection (default)
    pub fn backend_auto(mut self) -> Self {
        self.backend = BackendMode::Auto;
        self
    }

    /// Use native backend (requires libstdio_bus)
    pub fn backend_native(mut self) -> Self {
        self.backend = BackendMode::Native;
        self
    }

    /// Use Docker backend
    pub fn backend_docker(mut self) -> Self {
        self.backend = BackendMode::Docker;
        self
    }

    /// Set default request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set Docker options
    pub fn docker_options(mut self, options: DockerOptions) -> Self {
        self.docker_options = Some(options);
        self
    }

    /// Set Docker image
    pub fn docker_image(mut self, image: impl Into<String>) -> Self {
        let opts = self.docker_options.get_or_insert_with(DockerOptions::default);
        opts.image = image.into();
        self
    }

    /// Set Docker pull policy: "never", "if-missing", "always"
    pub fn docker_pull_policy(mut self, policy: impl Into<String>) -> Self {
        let opts = self.docker_options.get_or_insert_with(DockerOptions::default);
        opts.pull_policy = policy.into();
        self
    }

    /// Build the StdioBus instance
    pub fn build(self) -> Result<StdioBus> {
        let config_path = self.config_path.ok_or_else(|| Error::InvalidArgument {
            message: "config_path is required".to_string(),
        })?;

        StdioBus::new(config_path, self.backend, self.timeout, self.docker_options)
    }
}
