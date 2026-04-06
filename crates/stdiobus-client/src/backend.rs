// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Backend resolution for stdio_bus implementations

use stdiobus_core::{Backend, BackendMode, ConfigSource, DockerOptions, Error, Result};

/// Resolve backend based on mode and platform
pub fn resolve_backend(
    mode: BackendMode,
    config_source: ConfigSource,
    docker_options: Option<DockerOptions>,
) -> Result<Box<dyn Backend>> {
    match mode {
        BackendMode::Auto => {
            // On Windows, always use Docker
            #[cfg(windows)]
            {
                return create_docker_backend(config_source, docker_options);
            }

            // On Unix, try native first, fall back to docker
            #[cfg(not(windows))]
            {
                #[cfg(feature = "native")]
                {
                    match create_native_backend(&config_source) {
                        Ok(backend) => return Ok(backend),
                        Err(e) => {
                            tracing::warn!("Native backend unavailable: {}, falling back to docker", e);
                        }
                    }
                }

                create_docker_backend(config_source, docker_options)
            }
        }
        BackendMode::Native => {
            #[cfg(feature = "native")]
            {
                create_native_backend(&config_source)
            }
            #[cfg(not(feature = "native"))]
            {
                Err(Error::InvalidArgument {
                    message: "Native backend not available (compile with 'native' feature)".to_string(),
                })
            }
        }
        BackendMode::Docker => create_docker_backend(config_source, docker_options),
    }
}

#[cfg(feature = "docker")]
fn create_docker_backend(
    config_source: ConfigSource,
    options: Option<DockerOptions>,
) -> Result<Box<dyn Backend>> {
    // Docker backend needs a file path — resolve ConfigSource
    let config_path = match config_source {
        ConfigSource::Path(p) => p,
        ConfigSource::Config(cfg) => {
            // Materialize to temp file for Docker
            let json = cfg.to_json().map_err(|e| Error::InvalidArgument {
                message: format!("Failed to serialize config: {}", e),
            })?;
            let tmp = std::env::temp_dir().join(format!("stdiobus-{}.json", std::process::id()));
            std::fs::write(&tmp, &json).map_err(|e| Error::InternalError {
                message: format!("Failed to write temp config: {}", e),
            })?;
            tmp.to_string_lossy().into_owned()
        }
    };
    let backend = stdiobus_backend_docker::DockerBackend::new(
        &config_path,
        options.unwrap_or_default(),
    )?;
    Ok(Box::new(backend))
}

#[cfg(not(feature = "docker"))]
fn create_docker_backend(
    _config_source: ConfigSource,
    _options: Option<DockerOptions>,
) -> Result<Box<dyn Backend>> {
    Err(Error::InvalidArgument {
        message: "Docker backend not available (compile with 'docker' feature)".to_string(),
    })
}

#[cfg(feature = "native")]
fn create_native_backend(config_source: &ConfigSource) -> Result<Box<dyn Backend>> {
    let backend = stdiobus_backend_native::NativeBackend::from_config_source(config_source)?;
    Ok(Box::new(backend))
}
