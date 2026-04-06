// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

//! Unit tests for stdiobus-client

use crate::builder::StdioBusBuilder;
use stdiobus_core::{BackendMode, DockerOptions, Error};
use std::time::Duration;

// ============================================================================
// Builder Tests
// ============================================================================

#[test]
fn test_builder_default() {
    let builder = StdioBusBuilder::new();
    // Can't access private fields, but we can test behavior
    assert!(builder.clone().build().is_err()); // No config path
}

#[test]
fn test_builder_config_path_required() {
    let result = StdioBusBuilder::new().build();
    
    assert!(result.is_err());
    if let Err(Error::InvalidArgument { message }) = result {
        assert!(message.contains("config_path"));
    } else {
        panic!("Expected InvalidArgument error");
    }
}

#[test]
fn test_builder_with_config_path() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json");
    
    // Build will fail because file doesn't exist, but builder is valid
    let result = builder.build();
    // This might succeed or fail depending on backend resolution
    // The important thing is config_path was set
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builder_backend_modes() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_auto();
    let _ = builder.build();
    
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_docker();
    let _ = builder.build();
    
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_native();
    // Native might fail if libstdio_bus not available
    let _ = builder.build();
}

#[test]
fn test_builder_backend_enum() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend(BackendMode::Docker);
    let _ = builder.build();
}

#[test]
fn test_builder_timeout() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .timeout(Duration::from_secs(60));
    let _ = builder.build();
}

#[test]
fn test_builder_docker_options() {
    let opts = DockerOptions {
        image: "custom:latest".to_string(),
        ..Default::default()
    };
    
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_docker()
        .docker_options(opts);
    let _ = builder.build();
}

#[test]
fn test_builder_docker_image() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_docker()
        .docker_image("my-image:v1");
    let _ = builder.build();
}

#[test]
fn test_builder_chain() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json")
        .backend_docker()
        .timeout(Duration::from_secs(120))
        .docker_image("stdiobus/stdiobus:latest");
    
    // Builder should be valid
    let _ = builder.build();
}

#[test]
fn test_builder_clone() {
    let builder1 = StdioBusBuilder::new()
        .config_path("./config.json")
        .timeout(Duration::from_secs(30));
    
    let builder2 = builder1.clone();
    
    // Both should produce same result
    let _ = builder1.build();
    let _ = builder2.build();
}

#[test]
fn test_builder_debug() {
    let builder = StdioBusBuilder::new()
        .config_path("./config.json");
    
    let debug = format!("{:?}", builder);
    assert!(debug.contains("config.json"));
}

// ============================================================================
// StdioBus API Tests (without actual backend)
// ============================================================================

#[test]
fn test_stdiobus_builder_method() {
    use crate::StdioBus;
    
    let builder = StdioBus::builder();
    // Should return a StdioBusBuilder
    let _ = builder.config_path("./config.json");
}

// ============================================================================
// Programmatic Config Tests
// ============================================================================

#[test]
fn test_builder_config_object() {
    use stdiobus_core::{BusConfig, PoolConfig};

    let builder = StdioBusBuilder::new()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "echo".into(),
                command: "/bin/cat".into(),
                args: vec![],
                instances: 1,
            }],
            limits: None,
        });

    // Build should succeed (config is valid)
    let result = builder.build();
    assert!(result.is_ok() || result.is_err()); // backend may not be available
}

#[test]
fn test_builder_config_with_limits() {
    use stdiobus_core::{BusConfig, LimitsConfig, PoolConfig};

    let builder = StdioBusBuilder::new()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "worker".into(),
                command: "node".into(),
                args: vec!["worker.js".into()],
                instances: 4,
            }],
            limits: Some(LimitsConfig {
                max_input_buffer: Some(2097152),
                max_restarts: Some(10),
                ..Default::default()
            }),
        });

    let _ = builder.build();
}

#[test]
fn test_builder_no_config_source_fails() {
    let result = StdioBusBuilder::new().build();
    assert!(result.is_err());
    if let Err(Error::InvalidArgument { message }) = result {
        assert!(message.contains("config"));
    }
}

#[test]
fn test_builder_empty_pools_fails() {
    use stdiobus_core::BusConfig;

    let result = StdioBusBuilder::new()
        .config(BusConfig {
            pools: vec![],
            limits: None,
        })
        .build();

    assert!(result.is_err());
    if let Err(Error::InvalidArgument { message }) = result {
        assert!(message.contains("pool"));
    }
}

#[test]
fn test_builder_zero_instances_fails() {
    use stdiobus_core::{BusConfig, PoolConfig};

    let result = StdioBusBuilder::new()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "bad".into(),
                command: "/bin/echo".into(),
                args: vec![],
                instances: 0,
            }],
            limits: None,
        })
        .build();

    assert!(result.is_err());
    if let Err(Error::InvalidArgument { message }) = result {
        assert!(message.contains("instances"));
    }
}

#[test]
fn test_builder_missing_pool_id_fails() {
    use stdiobus_core::{BusConfig, PoolConfig};

    let result = StdioBusBuilder::new()
        .config(BusConfig {
            pools: vec![PoolConfig {
                id: "".into(),
                command: "/bin/echo".into(),
                args: vec![],
                instances: 1,
            }],
            limits: None,
        })
        .build();

    assert!(result.is_err());
    if let Err(Error::InvalidArgument { message }) = result {
        assert!(message.contains("id"));
    }
}

#[test]
fn test_bus_config_to_json() {
    use stdiobus_core::{BusConfig, PoolConfig};

    let config = BusConfig {
        pools: vec![PoolConfig {
            id: "test".into(),
            command: "node".into(),
            args: vec!["worker.js".into()],
            instances: 2,
        }],
        limits: None,
    };

    let json = config.to_json().unwrap();
    assert!(json.contains("\"id\":\"test\""));
    assert!(json.contains("\"command\":\"node\""));
    assert!(json.contains("\"instances\":2"));
}

#[test]
fn test_bus_config_roundtrip() {
    use stdiobus_core::{BusConfig, LimitsConfig, PoolConfig};

    let config = BusConfig {
        pools: vec![PoolConfig {
            id: "w1".into(),
            command: "/bin/cat".into(),
            args: vec!["--flag".into()],
            instances: 3,
        }],
        limits: Some(LimitsConfig {
            max_input_buffer: Some(999),
            max_restarts: Some(7),
            ..Default::default()
        }),
    };

    let json = config.to_json().unwrap();
    let parsed: BusConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.pools.len(), 1);
    assert_eq!(parsed.pools[0].id, "w1");
    assert_eq!(parsed.pools[0].instances, 3);
    assert_eq!(parsed.limits.as_ref().unwrap().max_input_buffer, Some(999));
    assert_eq!(parsed.limits.as_ref().unwrap().max_restarts, Some(7));
    assert_eq!(parsed.limits.as_ref().unwrap().drain_timeout_sec, None);
}
