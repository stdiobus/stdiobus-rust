// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Docker backend for stdio_bus
//!
//! Runs stdio_bus in a Docker container and communicates via TCP.

use async_trait::async_trait;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use stdiobus_core::{Backend, BusMessage, BusState, BusStats, DockerOptions, Error, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex, RwLock};

/// Docker backend implementation
pub struct DockerBackend {
    config_path: String,
    options: DockerOptions,
    state: RwLock<BusState>,
    container_id: RwLock<Option<String>>,
    writer: RwLock<Option<OwnedWriteHalf>>,
    message_tx: mpsc::Sender<BusMessage>,
    message_rx: Mutex<Option<mpsc::Receiver<BusMessage>>>,
    stats: Arc<Stats>,
}

struct Stats {
    messages_in: AtomicU64,
    messages_out: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
}

impl DockerBackend {
    /// Create a new Docker backend
    pub fn new(config_path: &str, options: DockerOptions) -> Result<Self> {
        // Verify config file exists
        if !Path::new(config_path).exists() {
            return Err(Error::InvalidArgument {
                message: format!("Config file not found: {}", config_path),
            });
        }

        let (tx, rx) = mpsc::channel(1000);

        Ok(Self {
            config_path: config_path.to_string(),
            options,
            state: RwLock::new(BusState::Created),
            container_id: RwLock::new(None),
            writer: RwLock::new(None),
            message_tx: tx,
            message_rx: Mutex::new(Some(rx)),
            stats: Arc::new(Stats {
                messages_in: AtomicU64::new(0),
                messages_out: AtomicU64::new(0),
                bytes_in: AtomicU64::new(0),
                bytes_out: AtomicU64::new(0),
            }),
        })
    }

    /// Pull Docker image if needed
    async fn pull_image(&self) -> Result<()> {
        if self.options.pull_policy == "never" {
            return Ok(());
        }

        if self.options.pull_policy == "if-missing" {
            let output = Command::new(&self.options.engine_path)
                .args(["image", "inspect", &self.options.image])
                .output()
                .await?;

            if output.status.success() {
                return Ok(());
            }
        }

        tracing::info!("Pulling Docker image: {}", self.options.image);

        let status = Command::new(&self.options.engine_path)
            .args(["pull", &self.options.image])
            .status()
            .await?;

        if !status.success() {
            return Err(Error::TransportError {
                message: format!("Failed to pull image: {}", self.options.image),
            });
        }

        Ok(())
    }

    /// Start the Docker container
    async fn start_container(&self) -> Result<String> {
        let container_name = format!(
            "{}-{}",
            self.options.container_name_prefix,
            uuid::Uuid::new_v4()
        );

        let config_path = std::fs::canonicalize(&self.config_path)?;
        let config_mount = format!("{}:/config.json:ro", config_path.display());

        // Bind to port 0 to let the OS assign an available port, then read it back
        let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| {
            Error::TransportError {
                message: format!("Failed to find available port: {}", e),
            }
        })?;
        let port = listener.local_addr().map_err(|e| {
            Error::TransportError {
                message: format!("Failed to get local address: {}", e),
            }
        })?.port();
        // Drop the listener to free the port before Docker binds to it
        drop(listener);

        let mut args = vec![
            "run".to_string(),
            "-d".to_string(),
            "--rm".to_string(),
            "--name".to_string(),
            container_name.clone(),
            "-v".to_string(),
            config_mount,
            "-p".to_string(),
            format!("127.0.0.1:{}:8080", port),
        ];

        for (key, value) in &self.options.env {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        args.extend(self.options.extra_args.clone());
        args.push(self.options.image.clone());
        args.push("--config".to_string());
        args.push("/config.json".to_string());
        args.push("--tcp".to_string());
        args.push("0.0.0.0:8080".to_string());

        tracing::info!("Starting container: {}", container_name);

        let output = Command::new(&self.options.engine_path)
            .args(&args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::TransportError {
                message: format!("Failed to start container: {}", stderr),
            });
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        tracing::info!("Container started: {}", container_id);

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let addr = format!("127.0.0.1:{}", port);
        let stream = tokio::time::timeout(
            self.options.startup_timeout,
            Self::connect_with_retry(&addr),
        )
        .await
        .map_err(|_| Error::Timeout {
            timeout_ms: self.options.startup_timeout.as_millis() as u64,
        })??;

        let (reader, writer) = stream.into_split();
        *self.writer.write().await = Some(writer);

        let tx = self.message_tx.clone();
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(n) => {
                        stats.messages_out.fetch_add(1, Ordering::Relaxed);
                        stats.bytes_out.fetch_add(n as u64, Ordering::Relaxed);

                        let json = line.trim().to_string();
                        if !json.is_empty() {
                            let _ = tx.send(BusMessage { json }).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Read error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(container_id)
    }

    async fn connect_with_retry(addr: &str) -> Result<TcpStream> {
        let mut attempts = 0;
        loop {
            match TcpStream::connect(addr).await {
                Ok(stream) => return Ok(stream),
                Err(_e) if attempts < 30 => {
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    return Err(Error::TransportError {
                        message: format!("Failed to connect to {}: {}", addr, e),
                    });
                }
            }
        }
    }

    async fn stop_container(&self, timeout_secs: u32) -> Result<()> {
        let container_id = self.container_id.read().await.clone();

        if let Some(id) = container_id {
            tracing::info!("Stopping container: {}", id);
            let _ = Command::new(&self.options.engine_path)
                .args(["stop", "-t", &timeout_secs.to_string(), &id])
                .status()
                .await;
        }

        Ok(())
    }
}

#[async_trait]
impl Backend for DockerBackend {
    async fn start(&self) -> Result<()> {
        {
            let state = self.state.read().await;
            if !state.can_start() {
                return Err(Error::InvalidState {
                    expected: "CREATED or STOPPED".to_string(),
                    actual: state.to_string(),
                });
            }
        }

        *self.state.write().await = BusState::Starting;
        self.pull_image().await?;

        let container_id = self.start_container().await?;
        *self.container_id.write().await = Some(container_id);

        *self.state.write().await = BusState::Running;
        Ok(())
    }

    async fn stop(&self, timeout_secs: u32) -> Result<()> {
        *self.state.write().await = BusState::Stopping;
        self.stop_container(timeout_secs).await?;
        *self.state.write().await = BusState::Stopped;
        Ok(())
    }

    async fn send(&self, message: &str) -> Result<()> {
        let mut writer_guard = self.writer.write().await;
        if let Some(ref mut writer) = *writer_guard {
            let msg = format!("{}\n", message);
            writer.write_all(msg.as_bytes()).await?;
            self.stats.messages_in.fetch_add(1, Ordering::Relaxed);
            self.stats.bytes_in.fetch_add(msg.len() as u64, Ordering::Relaxed);
            Ok(())
        } else {
            Err(Error::InvalidState {
                expected: "RUNNING with active connection".to_string(),
                actual: "no connection".to_string(),
            })
        }
    }

    fn state(&self) -> BusState {
        self.state.try_read().map(|s| *s).unwrap_or(BusState::Created)
    }

    fn stats(&self) -> BusStats {
        BusStats {
            messages_in: self.stats.messages_in.load(Ordering::Relaxed),
            messages_out: self.stats.messages_out.load(Ordering::Relaxed),
            bytes_in: self.stats.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.stats.bytes_out.load(Ordering::Relaxed),
            ..Default::default()
        }
    }

    fn worker_count(&self) -> i32 {
        -1
    }

    fn client_count(&self) -> i32 {
        -1
    }

    fn subscribe(&self) -> Option<mpsc::Receiver<BusMessage>> {
        self.message_rx.try_lock().ok().and_then(|mut rx| rx.take())
    }

    fn backend_type(&self) -> &'static str {
        "docker"
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_config() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"pools": [{{"id": "test", "command": "echo", "args": ["hello"], "instances": 1}}]}}"#).unwrap();
        file
    }

    #[test]
    fn test_docker_backend_new_missing_config() {
        let result = DockerBackend::new("/nonexistent/path.json", DockerOptions::default());
        assert!(result.is_err());
        
        if let Err(Error::InvalidArgument { message }) = result {
            assert!(message.contains("not found"));
        } else {
            panic!("Expected InvalidArgument error");
        }
    }

    #[test]
    fn test_docker_backend_new_valid_config() {
        let config = create_test_config();
        let result = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_docker_backend_initial_state() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        assert_eq!(backend.state(), BusState::Created);
        assert_eq!(backend.worker_count(), -1); // Unknown for docker
        assert_eq!(backend.client_count(), -1); // Unknown for docker
        assert_eq!(backend.backend_type(), "docker");
    }

    #[test]
    fn test_docker_backend_stats_initial() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        let stats = backend.stats();
        assert_eq!(stats.messages_in, 0);
        assert_eq!(stats.messages_out, 0);
        assert_eq!(stats.bytes_in, 0);
        assert_eq!(stats.bytes_out, 0);
    }

    #[test]
    fn test_docker_backend_subscribe() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        // First subscribe should succeed
        let rx = backend.subscribe();
        assert!(rx.is_some());
        
        // Second subscribe should fail (already taken)
        let rx2 = backend.subscribe();
        assert!(rx2.is_none());
    }

    #[tokio::test]
    async fn test_docker_backend_start_invalid_state() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        // Manually set state to Running (simulating already started)
        *backend.state.write().await = BusState::Running;
        
        let result = backend.start().await;
        assert!(result.is_err());
        
        if let Err(Error::InvalidState { expected, actual }) = result {
            assert!(expected.contains("CREATED"));
            assert!(actual.contains("RUNNING"));
        } else {
            panic!("Expected InvalidState error");
        }
    }

    #[tokio::test]
    async fn test_docker_backend_send_not_connected() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        let result = backend.send(r#"{"test": true}"#).await;
        assert!(result.is_err());
        
        if let Err(Error::InvalidState { .. }) = result {
            // Expected
        } else {
            panic!("Expected InvalidState error");
        }
    }

    #[tokio::test]
    async fn test_docker_backend_stop_from_created() {
        let config = create_test_config();
        let backend = DockerBackend::new(config.path().to_str().unwrap(), DockerOptions::default()).unwrap();
        
        // Stop from Created state should work (no container to stop)
        let result = backend.stop(1).await;
        assert!(result.is_ok());
        assert_eq!(backend.state(), BusState::Stopped);
    }

    #[test]
    fn test_docker_options_in_backend() {
        let config = create_test_config();
        let opts = DockerOptions {
            image: "custom:latest".to_string(),
            pull_policy: "never".to_string(),
            ..Default::default()
        };
        
        let backend = DockerBackend::new(config.path().to_str().unwrap(), opts).unwrap();
        assert_eq!(backend.backend_type(), "docker");
    }
}
