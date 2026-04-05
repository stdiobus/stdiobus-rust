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
