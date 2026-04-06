// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Unit tests for stdiobus-core

use super::*;
use std::time::Duration;

// ============================================================================
// BusState Tests
// ============================================================================

#[test]
fn test_bus_state_transitions() {
    assert!(BusState::Created.can_start());
    assert!(BusState::Stopped.can_start());
    assert!(!BusState::Running.can_start());
    assert!(!BusState::Starting.can_start());
    assert!(!BusState::Stopping.can_start());
}

#[test]
fn test_bus_state_display() {
    assert_eq!(BusState::Created.to_string(), "CREATED");
    assert_eq!(BusState::Starting.to_string(), "STARTING");
    assert_eq!(BusState::Running.to_string(), "RUNNING");
    assert_eq!(BusState::Stopping.to_string(), "STOPPING");
    assert_eq!(BusState::Stopped.to_string(), "STOPPED");
}

#[test]
fn test_bus_state_copy() {
    let state = BusState::Running;
    let copied = state;
    assert_eq!(state, copied);
}

#[test]
fn test_bus_state_debug() {
    let state = BusState::Running;
    let debug = format!("{:?}", state);
    assert!(debug.contains("Running"));
}

// ============================================================================
// ErrorCode Tests
// ============================================================================

#[test]
fn test_error_codes_values() {
    assert_eq!(ErrorCode::InvalidArgument as i32, -1);
    assert_eq!(ErrorCode::InvalidState as i32, -2);
    assert_eq!(ErrorCode::Timeout as i32, -3);
    assert_eq!(ErrorCode::Cancelled as i32, -4);
    assert_eq!(ErrorCode::TransportError as i32, -5);
    assert_eq!(ErrorCode::NegotiationFailed as i32, -6);
    assert_eq!(ErrorCode::ExtensionUnavailable as i32, -7);
    assert_eq!(ErrorCode::PolicyDenied as i32, -8);
    assert_eq!(ErrorCode::InternalError as i32, -9);
    assert_eq!(ErrorCode::ResourceExhausted as i32, -10);
}

#[test]
fn test_error_code_display() {
    assert_eq!(ErrorCode::InvalidArgument.to_string(), "INVALID_ARGUMENT");
    assert_eq!(ErrorCode::InvalidState.to_string(), "INVALID_STATE");
    assert_eq!(ErrorCode::Timeout.to_string(), "TIMEOUT");
    assert_eq!(ErrorCode::Cancelled.to_string(), "CANCELLED");
    assert_eq!(ErrorCode::TransportError.to_string(), "TRANSPORT_ERROR");
    assert_eq!(ErrorCode::NegotiationFailed.to_string(), "NEGOTIATION_FAILED");
    assert_eq!(ErrorCode::ExtensionUnavailable.to_string(), "EXTENSION_UNAVAILABLE");
    assert_eq!(ErrorCode::PolicyDenied.to_string(), "POLICY_DENIED");
    assert_eq!(ErrorCode::InternalError.to_string(), "INTERNAL_ERROR");
    assert_eq!(ErrorCode::ResourceExhausted.to_string(), "RESOURCE_EXHAUSTED");
}

// ============================================================================
// Error Tests
// ============================================================================

#[test]
fn test_error_code_from_error() {
    assert_eq!(Error::Timeout { timeout_ms: 5000 }.code(), ErrorCode::Timeout);
    assert_eq!(Error::InvalidArgument { message: "bad".into() }.code(), ErrorCode::InvalidArgument);
    assert_eq!(Error::InvalidState { expected: "A".into(), actual: "B".into() }.code(), ErrorCode::InvalidState);
    assert_eq!(Error::Cancelled.code(), ErrorCode::Cancelled);
    assert_eq!(Error::TransportError { message: "err".into() }.code(), ErrorCode::TransportError);
    assert_eq!(Error::NegotiationFailed { message: "err".into() }.code(), ErrorCode::NegotiationFailed);
    assert_eq!(Error::ExtensionUnavailable { extension: "x".into() }.code(), ErrorCode::ExtensionUnavailable);
    assert_eq!(Error::PolicyDenied { message: "denied".into() }.code(), ErrorCode::PolicyDenied);
    assert_eq!(Error::InternalError { message: "err".into() }.code(), ErrorCode::InternalError);
    assert_eq!(Error::ResourceExhausted { resource: "mem".into() }.code(), ErrorCode::ResourceExhausted);
}

#[test]
fn test_error_is_retryable() {
    // Retryable errors
    assert!(Error::Timeout { timeout_ms: 1000 }.is_retryable());
    assert!(Error::TransportError { message: "err".into() }.is_retryable());
    assert!(Error::ResourceExhausted { resource: "mem".into() }.is_retryable());
    
    // Non-retryable errors
    assert!(!Error::InvalidArgument { message: "bad".into() }.is_retryable());
    assert!(!Error::InvalidState { expected: "A".into(), actual: "B".into() }.is_retryable());
    assert!(!Error::Cancelled.is_retryable());
    assert!(!Error::PolicyDenied { message: "denied".into() }.is_retryable());
}

#[test]
fn test_error_display() {
    let err = Error::Timeout { timeout_ms: 5000 };
    assert!(err.to_string().contains("5000"));
    
    let err = Error::InvalidState { expected: "RUNNING".into(), actual: "STOPPED".into() };
    assert!(err.to_string().contains("RUNNING"));
    assert!(err.to_string().contains("STOPPED"));
}

#[test]
fn test_error_from_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let err: Error = json_err.into();
    assert_eq!(err.code(), ErrorCode::InvalidArgument);
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: Error = io_err.into();
    assert_eq!(err.code(), ErrorCode::TransportError);
}

// ============================================================================
// JSON-RPC Message Tests
// ============================================================================

#[test]
fn test_json_rpc_request_new() {
    let req = JsonRpcRequest::new("test/method", Some(serde_json::json!({"key": "value"})));
    
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test/method");
    assert!(req.id.is_some());
    assert!(req.params.is_some());
}

#[test]
fn test_json_rpc_request_no_params() {
    let req = JsonRpcRequest::new("test/method", None);
    
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test/method");
    assert!(req.id.is_some());
    assert!(req.params.is_none());
}

#[test]
fn test_json_rpc_notification() {
    let req = JsonRpcRequest::notification("test/notify", None);
    
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test/notify");
    assert!(req.id.is_none());
}

#[test]
fn test_json_rpc_with_session_id() {
    let req = JsonRpcRequest::new("test/method", None)
        .with_session_id("session-123");
    
    assert_eq!(req.session_id, Some("session-123".to_string()));
}

#[test]
fn test_json_rpc_serialization() {
    let req = JsonRpcRequest::new("test/method", Some(serde_json::json!({"foo": "bar"})));
    let json = serde_json::to_string(&req).unwrap();
    
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"test/method\""));
    assert!(json.contains("\"foo\":\"bar\""));
}

#[test]
fn test_json_rpc_deserialization() {
    let json = r#"{"jsonrpc":"2.0","method":"test","id":"123","params":{"x":1}}"#;
    let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
    
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test");
    assert_eq!(req.id, Some(serde_json::json!("123")));
}

#[test]
fn test_json_rpc_response_success() {
    let json = r#"{"jsonrpc":"2.0","id":"123","result":{"status":"ok"}}"#;
    let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
    
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn test_json_rpc_response_error() {
    let json = r#"{"jsonrpc":"2.0","id":"123","error":{"code":-32600,"message":"Invalid Request"}}"#;
    let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
    
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32600);
    assert_eq!(err.message, "Invalid Request");
}

#[test]
fn test_json_rpc_error_with_data() {
    let err = JsonRpcError {
        code: -32000,
        message: "Server error".to_string(),
        data: Some(serde_json::json!({"details": "more info"})),
    };
    
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("\"code\":-32000"));
    assert!(json.contains("\"details\":\"more info\""));
}

// ============================================================================
// BusStats Tests
// ============================================================================

#[test]
fn test_bus_stats_default() {
    let stats = BusStats::default();
    
    assert_eq!(stats.messages_in, 0);
    assert_eq!(stats.messages_out, 0);
    assert_eq!(stats.bytes_in, 0);
    assert_eq!(stats.bytes_out, 0);
    assert_eq!(stats.worker_restarts, 0);
    assert_eq!(stats.routing_errors, 0);
    assert_eq!(stats.client_connects, 0);
    assert_eq!(stats.client_disconnects, 0);
}

#[test]
fn test_bus_stats_clone() {
    let stats = BusStats {
        messages_in: 100,
        messages_out: 50,
        bytes_in: 1000,
        bytes_out: 500,
        ..Default::default()
    };
    
    let cloned = stats.clone();
    assert_eq!(cloned.messages_in, 100);
    assert_eq!(cloned.messages_out, 50);
}

// ============================================================================
// Identity Tests
// ============================================================================

#[test]
fn test_identity_self_asserted() {
    let identity = Identity::self_asserted("agent-1", "executor");
    
    assert_eq!(identity.subject_id, "agent-1");
    assert_eq!(identity.role, "executor");
    assert_eq!(identity.asserted_by, "self");
}

#[test]
fn test_identity_serialization() {
    let identity = Identity::self_asserted("agent-1", "executor");
    let json = serde_json::to_string(&identity).unwrap();
    
    assert!(json.contains("\"subjectId\":\"agent-1\""));
    assert!(json.contains("\"role\":\"executor\""));
    assert!(json.contains("\"assertedBy\":\"self\""));
}

#[test]
fn test_identity_deserialization() {
    let json = r#"{"subjectId":"agent-2","role":"auditor","assertedBy":"bus"}"#;
    let identity: Identity = serde_json::from_str(json).unwrap();
    
    assert_eq!(identity.subject_id, "agent-2");
    assert_eq!(identity.role, "auditor");
    assert_eq!(identity.asserted_by, "bus");
}

// ============================================================================
// RequestOptions Tests
// ============================================================================

#[test]
fn test_request_options_default() {
    let opts = RequestOptions::default();
    
    assert!(opts.timeout.is_none());
    assert!(opts.session_id.is_none());
    assert!(opts.idempotency_key.is_none());
    assert!(opts.required_extensions.is_empty());
}

#[test]
fn test_request_options_with_timeout() {
    let opts = RequestOptions::with_timeout(Duration::from_secs(30));
    
    assert_eq!(opts.timeout, Some(Duration::from_secs(30)));
}

#[test]
fn test_request_options_builder_chain() {
    let opts = RequestOptions::with_timeout(Duration::from_secs(30))
        .session_id("session-123")
        .idempotency_key("idem-456")
        .require_extension("identity")
        .require_extension("audit");
    
    assert_eq!(opts.timeout, Some(Duration::from_secs(30)));
    assert_eq!(opts.session_id, Some("session-123".to_string()));
    assert_eq!(opts.idempotency_key, Some("idem-456".to_string()));
    assert_eq!(opts.required_extensions, vec!["identity".to_string(), "audit".to_string()]);
}

// ============================================================================
// DockerOptions Tests
// ============================================================================

#[test]
fn test_docker_options_default() {
    let opts = DockerOptions::default();
    
    assert_eq!(opts.image, "stdiobus/stdiobus:node20");
    assert_eq!(opts.pull_policy, "if-missing");
    assert_eq!(opts.engine_path, "docker");
    assert_eq!(opts.startup_timeout, Duration::from_secs(15));
    assert_eq!(opts.container_name_prefix, "stdiobus");
    assert!(opts.extra_args.is_empty());
    assert!(opts.env.is_empty());
}

#[test]
fn test_docker_options_custom() {
    use std::collections::HashMap;
    
    let mut env = HashMap::new();
    env.insert("DEBUG".to_string(), "1".to_string());
    
    let opts = DockerOptions {
        image: "custom/image:latest".to_string(),
        pull_policy: "always".to_string(),
        engine_path: "/usr/local/bin/docker".to_string(),
        startup_timeout: Duration::from_secs(30),
        container_name_prefix: "my-bus".to_string(),
        extra_args: vec!["--memory=512m".to_string()],
        env,
    };
    
    assert_eq!(opts.image, "custom/image:latest");
    assert_eq!(opts.pull_policy, "always");
    assert_eq!(opts.extra_args.len(), 1);
    assert_eq!(opts.env.get("DEBUG"), Some(&"1".to_string()));
}

// ============================================================================
// BackendMode Tests
// ============================================================================

#[test]
fn test_backend_mode_display() {
    assert_eq!(BackendMode::Auto.to_string(), "auto");
    assert_eq!(BackendMode::Native.to_string(), "native");
    assert_eq!(BackendMode::Docker.to_string(), "docker");
}

#[test]
fn test_backend_mode_default() {
    let mode = BackendMode::default();
    assert_eq!(mode, BackendMode::Auto);
}

#[test]
fn test_backend_mode_equality() {
    assert_eq!(BackendMode::Auto, BackendMode::Auto);
    assert_ne!(BackendMode::Auto, BackendMode::Native);
    assert_ne!(BackendMode::Native, BackendMode::Docker);
}

// ============================================================================
// Extensions Tests
// ============================================================================

#[test]
fn test_extension_info_new() {
    let info = ExtensionInfo::new("1.0.0");
    
    assert_eq!(info.version, "1.0.0");
    assert!(!info.required);
    assert!(!info.active);
}

#[test]
fn test_extension_info_required() {
    let info = ExtensionInfo::new("1.0.0").required();
    
    assert_eq!(info.version, "1.0.0");
    assert!(info.required);
}

#[test]
fn test_extensions_default() {
    let ext = Extensions::default();
    assert!(ext.extensions.is_empty());
}

// ============================================================================
// BusMessage Tests
// ============================================================================

#[test]
fn test_bus_message_clone() {
    let msg = BusMessage {
        json: r#"{"test": true}"#.to_string(),
    };
    
    let cloned = msg.clone();
    assert_eq!(cloned.json, msg.json);
}

#[test]
fn test_bus_message_debug() {
    let msg = BusMessage {
        json: r#"{"test": true}"#.to_string(),
    };
    
    let debug = format!("{:?}", msg);
    assert!(debug.contains("test"));
}

// ============================================================================
// BusConfig Documentation Examples — verified by test execution
// ============================================================================

#[test]
fn test_doc_example_basic_config() {
    // From README: basic programmatic config
    let config = BusConfig {
        pools: vec![PoolConfig {
            id: "mcp-worker".into(),
            command: "node".into(),
            args: vec!["./worker.js".into()],
            instances: 4,
        }],
        limits: None,
    };

    assert!(config.validate().is_ok());
    let json = config.to_json().unwrap();
    assert!(json.contains("\"id\":\"mcp-worker\""));
    assert!(json.contains("\"instances\":4"));
    // Verify no limits key when None
    assert!(!json.contains("\"limits\""));
}

#[test]
fn test_doc_example_config_with_limits() {
    // From README: config with explicit limits
    let config = BusConfig {
        pools: vec![PoolConfig {
            id: "worker".into(),
            command: "node".into(),
            args: vec!["./worker.js".into()],
            instances: 4,
        }],
        limits: Some(LimitsConfig {
            max_input_buffer: Some(2097152),
            max_restarts: Some(10),
            ..Default::default()
        }),
    };

    assert!(config.validate().is_ok());
    let json = config.to_json().unwrap();
    assert!(json.contains("\"max_input_buffer\":2097152"));
    assert!(json.contains("\"max_restarts\":10"));
    // Omitted fields should not appear (skip_serializing_if)
    assert!(!json.contains("\"drain_timeout_sec\""));
    assert!(!json.contains("\"max_output_queue\""));
}

#[test]
fn test_doc_example_config_roundtrip_json() {
    // Verify: config serialized to JSON can be parsed back identically
    let config = BusConfig {
        pools: vec![
            PoolConfig {
                id: "echo".into(),
                command: "/bin/cat".into(),
                args: vec!["--flag".into()],
                instances: 2,
            },
        ],
        limits: Some(LimitsConfig {
            max_input_buffer: Some(1048576),
            max_output_queue: Some(4194304),
            max_restarts: Some(5),
            restart_window_sec: Some(60),
            drain_timeout_sec: Some(30),
            backpressure_timeout_sec: Some(60),
        }),
    };

    let json = config.to_json().unwrap();
    let parsed: BusConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.pools.len(), 1);
    assert_eq!(parsed.pools[0].id, "echo");
    assert_eq!(parsed.pools[0].command, "/bin/cat");
    assert_eq!(parsed.pools[0].args, vec!["--flag"]);
    assert_eq!(parsed.pools[0].instances, 2);

    let limits = parsed.limits.unwrap();
    assert_eq!(limits.max_input_buffer, Some(1048576));
    assert_eq!(limits.max_output_queue, Some(4194304));
    assert_eq!(limits.max_restarts, Some(5));
    assert_eq!(limits.restart_window_sec, Some(60));
    assert_eq!(limits.drain_timeout_sec, Some(30));
    assert_eq!(limits.backpressure_timeout_sec, Some(60));
}

#[test]
fn test_doc_example_config_json_matches_c_schema() {
    // Verify: JSON output matches the C bus config.json schema exactly
    let config = BusConfig {
        pools: vec![PoolConfig {
            id: "acp-worker".into(),
            command: "/usr/bin/node".into(),
            args: vec!["./worker.js".into(), "--mode".into(), "acp".into()],
            instances: 4,
        }],
        limits: Some(LimitsConfig {
            max_input_buffer: Some(2097152),
            max_output_queue: Some(8388608),
            max_restarts: Some(10),
            restart_window_sec: Some(120),
            drain_timeout_sec: Some(60),
            backpressure_timeout_sec: Some(90),
        }),
    };

    let json = config.to_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify top-level structure matches C schema
    assert!(parsed.get("pools").unwrap().is_array());
    assert!(parsed.get("limits").unwrap().is_object());

    // Verify pool fields match C schema field names
    let pool = &parsed["pools"][0];
    assert_eq!(pool["id"].as_str().unwrap(), "acp-worker");
    assert_eq!(pool["command"].as_str().unwrap(), "/usr/bin/node");
    assert_eq!(pool["instances"].as_u64().unwrap(), 4);
    assert_eq!(pool["args"][0].as_str().unwrap(), "./worker.js");

    // Verify limits fields match C schema field names (snake_case)
    let limits = &parsed["limits"];
    assert_eq!(limits["max_input_buffer"].as_u64().unwrap(), 2097152);
    assert_eq!(limits["max_output_queue"].as_u64().unwrap(), 8388608);
    assert_eq!(limits["max_restarts"].as_u64().unwrap(), 10);
    assert_eq!(limits["restart_window_sec"].as_u64().unwrap(), 120);
    assert_eq!(limits["drain_timeout_sec"].as_u64().unwrap(), 60);
    assert_eq!(limits["backpressure_timeout_sec"].as_u64().unwrap(), 90);
}

#[test]
fn test_doc_example_validation_errors() {
    // Empty pools
    let config = BusConfig { pools: vec![], limits: None };
    assert!(config.validate().unwrap_err().contains("pool"));

    // Missing id
    let config = BusConfig {
        pools: vec![PoolConfig { id: "".into(), command: "node".into(), args: vec![], instances: 1 }],
        limits: None,
    };
    assert!(config.validate().unwrap_err().contains("id"));

    // Missing command
    let config = BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "".into(), args: vec![], instances: 1 }],
        limits: None,
    };
    assert!(config.validate().unwrap_err().contains("command"));

    // Zero instances
    let config = BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "node".into(), args: vec![], instances: 0 }],
        limits: None,
    };
    assert!(config.validate().unwrap_err().contains("instances"));
}

#[test]
fn test_config_source_variants() {
    // Path variant
    let source = ConfigSource::Path("/tmp/config.json".into());
    match &source {
        ConfigSource::Path(p) => assert_eq!(p, "/tmp/config.json"),
        _ => panic!("Expected Path"),
    }

    // Config variant
    let source = ConfigSource::Config(BusConfig {
        pools: vec![PoolConfig { id: "w".into(), command: "echo".into(), args: vec![], instances: 1 }],
        limits: None,
    });
    match &source {
        ConfigSource::Config(cfg) => {
            assert_eq!(cfg.pools[0].id, "w");
        }
        _ => panic!("Expected Config"),
    }
}
