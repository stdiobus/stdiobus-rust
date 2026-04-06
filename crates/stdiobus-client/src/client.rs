// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Main StdioBus client implementation

use crate::backend::resolve_backend;
use crate::builder::StdioBusBuilder;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use stdiobus_core::{
    Backend, BackendMode, BusMessage, BusState, BusStats, ConfigSource, DockerOptions, Error, JsonRpcRequest,
    JsonRpcResponse, RequestOptions, Result, generate_client_session_id,
};
use tokio::sync::{broadcast, oneshot, Mutex};

/// Pending request with response aggregation
struct PendingRequest {
    tx: oneshot::Sender<AggregatedResponse>,
    chunks: Vec<String>,
}

/// Aggregated response with collected chunks
struct AggregatedResponse {
    response: JsonRpcResponse,
    text: String,
}

/// Main stdio_bus client
pub struct StdioBus {
    backend: Box<dyn Backend>,
    default_timeout: Duration,
    client_session_id: String,
    pending_requests: Arc<Mutex<HashMap<String, PendingRequest>>>,
    notification_tx: broadcast::Sender<Value>,
}

impl StdioBus {
    /// Create a new builder
    pub fn builder() -> StdioBusBuilder {
        StdioBusBuilder::new()
    }

    /// Create a new StdioBus instance
    pub(crate) fn new(
        config_source: ConfigSource,
        backend_mode: BackendMode,
        default_timeout: Duration,
        docker_options: Option<DockerOptions>,
    ) -> Result<Self> {
        let backend = resolve_backend(backend_mode, config_source, docker_options)?;
        let (notification_tx, _) = broadcast::channel(100);

        Ok(Self {
            backend,
            default_timeout,
            client_session_id: generate_client_session_id(),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            notification_tx,
        })
    }

    /// Get the client session ID used for routing
    pub fn client_session_id(&self) -> &str {
        &self.client_session_id
    }

    /// Start the bus and spawn workers
    pub async fn start(&self) -> Result<()> {
        self.backend.start().await?;

        let pending = self.pending_requests.clone();
        let notif_tx = self.notification_tx.clone();
        
        if let Some(mut rx) = self.backend.subscribe() {
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    Self::handle_message(msg, &pending, &notif_tx).await;
                }
            });
        }

        Ok(())
    }

    /// Stop the bus gracefully
    pub async fn stop(&self) -> Result<()> {
        self.stop_with_timeout(30).await
    }

    /// Stop the bus with custom timeout
    pub async fn stop_with_timeout(&self, timeout_secs: u32) -> Result<()> {
        self.backend.stop(timeout_secs).await
    }

    /// Send a request and wait for response
    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        self.request_with_options(method, params, RequestOptions::default())
            .await
    }

    /// Send a request with custom options
    pub async fn request_with_options(
        &self,
        method: &str,
        params: Value,
        options: RequestOptions,
    ) -> Result<Value> {
        if !self.is_running() {
            return Err(Error::InvalidState {
                expected: "RUNNING".to_string(),
                actual: self.state().to_string(),
            });
        }

        let mut request = JsonRpcRequest::new(method, Some(params));
        let session_id = options.session_id.unwrap_or_else(|| self.client_session_id.clone());
        request = request.with_session_id(session_id);

        if let Some(agent_id) = options.agent_id {
            request = request.with_agent_id(agent_id);
        }

        let id = request
            .id
            .as_ref()
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InternalError {
                message: "Request ID not set".to_string(),
            })?
            .to_string();

        let json = serde_json::to_string(&request)?;

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), PendingRequest { tx, chunks: Vec::new() });
        }

        // Send request
        self.backend.send(&json).await?;

        // Wait for response with timeout
        let timeout = options.timeout.unwrap_or(self.default_timeout);
        let aggregated = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| Error::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|_| Error::InternalError {
                message: "Response channel closed".to_string(),
            })?;

        // Check for error response
        if let Some(error) = aggregated.response.error {
            return Err(Error::TransportError {
                message: format!("{}: {}", error.code, error.message),
            });
        }

        // Build final result with aggregated text
        let mut result = aggregated.response.result.unwrap_or(Value::Object(Default::default()));
        
        if !aggregated.text.is_empty() {
            if let Value::Object(ref mut map) = result {
                map.insert("text".to_string(), Value::String(aggregated.text));
            }
        }

        Ok(result)
    }

    /// Send a notification (no response expected)
    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        if !self.is_running() {
            return Err(Error::InvalidState {
                expected: "RUNNING".to_string(),
                actual: self.state().to_string(),
            });
        }

        let request = JsonRpcRequest::notification(method, Some(params))
            .with_session_id(&self.client_session_id);
        let json = serde_json::to_string(&request)?;
        self.backend.send(&json).await
    }

    /// Send a raw JSON message
    pub async fn send(&self, message: &str) -> Result<()> {
        self.backend.send(message).await
    }

    /// Subscribe to notifications
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<Value> {
        self.notification_tx.subscribe()
    }

    /// Get current bus state
    pub fn state(&self) -> BusState {
        self.backend.state()
    }

    /// Check if bus is running
    pub fn is_running(&self) -> bool {
        self.state() == BusState::Running
    }

    /// Get runtime statistics
    pub fn stats(&self) -> BusStats {
        self.backend.stats()
    }

    /// Get number of running workers
    pub fn worker_count(&self) -> i32 {
        self.backend.worker_count()
    }

    /// Get number of connected clients
    pub fn client_count(&self) -> i32 {
        self.backend.client_count()
    }

    /// Get backend type
    pub fn backend_type(&self) -> &'static str {
        self.backend.backend_type()
    }


    /// Handle incoming message — dispatch responses and aggregate ACP streaming chunks.
    ///
    /// Note: chunk aggregation for `agent_message_chunk` notifications is ACP-protocol
    /// specific. If you use this SDK with a non-ACP protocol, streaming chunks will
    /// simply be forwarded as notifications without aggregation.
    async fn handle_message(
        msg: BusMessage,
        pending: &Arc<Mutex<HashMap<String, PendingRequest>>>,
        notif_tx: &broadcast::Sender<Value>,
    ) {
        let parsed: Value = match serde_json::from_str(&msg.json) {
            Ok(v) => v,
            Err(_) => return,
        };

        // Check if it's a notification (has method, no id)
        if parsed.get("method").is_some() && parsed.get("id").is_none() {
            // Extract text from agent_message_chunk notifications
            if let Some(params) = parsed.get("params") {
                if let Some(update) = params.get("update") {
                    if update.get("sessionUpdate").and_then(|s| s.as_str()) == Some("agent_message_chunk") {
                        if let Some(text) = update.get("content")
                            .and_then(|c| c.get("text"))
                            .and_then(|t| t.as_str()) 
                        {
                            // Add chunk to all pending requests (typically one)
                            let mut guard = pending.lock().await;
                            for req in guard.values_mut() {
                                req.chunks.push(text.to_string());
                            }
                        }
                    }
                }
            }
            
            // Broadcast notification
            let _ = notif_tx.send(parsed);
            return;
        }

        // It's a response - find pending request by id
        if let Some(id) = parsed.get("id").and_then(|v| v.as_str()) {
            let mut guard = pending.lock().await;
            if let Some(req) = guard.remove(id) {
                let response: JsonRpcResponse = match serde_json::from_str(&msg.json) {
                    Ok(r) => r,
                    Err(_) => return,
                };
                
                let text = req.chunks.join("");
                let _ = req.tx.send(AggregatedResponse { response, text });
            }
        }
    }
}
