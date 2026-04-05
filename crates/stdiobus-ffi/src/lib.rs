// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

#![cfg_attr(docsrs, feature(doc_cfg))]

//! FFI bindings to libstdio_bus C library
//!
//! This crate provides raw FFI bindings to the stdio_bus embedding API
//! as defined in `include/stdio_bus_embed.h`.
//!
//! For a safe Rust API, use `stdiobus-client` instead.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::os::raw::{c_char, c_int, c_void};

/// API version
pub const STDIO_BUS_EMBED_API_VERSION: c_int = 2;

/// Return codes (from stdio_bus.h)
pub const STDIO_BUS_OK: c_int = 0;
pub const STDIO_BUS_ERR: c_int = -1;
pub const STDIO_BUS_EAGAIN: c_int = -2;
pub const STDIO_BUS_EOF: c_int = -3;
pub const STDIO_BUS_EFULL: c_int = -4;
pub const STDIO_BUS_ENOTFOUND: c_int = -5;
pub const STDIO_BUS_EINVAL: c_int = -6;

/// Error codes (from stdio_bus_embed.h)
pub const STDIO_BUS_ERR_CONFIG: c_int = -10;
pub const STDIO_BUS_ERR_WORKER: c_int = -11;
pub const STDIO_BUS_ERR_ROUTING: c_int = -12;
pub const STDIO_BUS_ERR_BUFFER: c_int = -13;
pub const STDIO_BUS_ERR_INVALID: c_int = -14;
pub const STDIO_BUS_ERR_STATE: c_int = -15;

/// Bus state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum stdio_bus_state_t {
    STDIO_BUS_STATE_CREATED = 0,
    STDIO_BUS_STATE_STARTING = 1,
    STDIO_BUS_STATE_RUNNING = 2,
    STDIO_BUS_STATE_STOPPING = 3,
    STDIO_BUS_STATE_STOPPED = 4,
}

/// Listen mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum stdio_bus_listen_mode_t {
    STDIO_BUS_LISTEN_NONE = 0,
    STDIO_BUS_LISTEN_TCP = 1,
    STDIO_BUS_LISTEN_UNIX = 2,
}

/// Opaque bus handle
#[repr(C)]
pub struct stdio_bus_t {
    _private: [u8; 0],
}

/// Message callback
pub type stdio_bus_message_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        msg: *const c_char,
        len: usize,
        user_data: *mut c_void,
    ),
>;

/// Error callback
pub type stdio_bus_error_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        code: c_int,
        message: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Log callback
pub type stdio_bus_log_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        level: c_int,
        message: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Worker event callback
pub type stdio_bus_worker_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        worker_id: c_int,
        event: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Client connect callback
pub type stdio_bus_client_connect_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        client_id: c_int,
        peer_info: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Client disconnect callback
pub type stdio_bus_client_disconnect_cb = Option<
    extern "C" fn(
        bus: *mut stdio_bus_t,
        client_id: c_int,
        reason: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Listener configuration
#[repr(C)]
pub struct stdio_bus_listener_config_t {
    pub mode: stdio_bus_listen_mode_t,
    pub tcp_host: *const c_char,
    pub tcp_port: u16,
    pub unix_path: *const c_char,
}

/// Options for creating a stdio_bus instance
#[repr(C)]
pub struct stdio_bus_options_t {
    pub config_path: *const c_char,
    pub config_json: *const c_char,
    pub listener: stdio_bus_listener_config_t,
    pub on_message: stdio_bus_message_cb,
    pub on_error: stdio_bus_error_cb,
    pub on_log: stdio_bus_log_cb,
    pub on_worker: stdio_bus_worker_cb,
    pub on_client_connect: stdio_bus_client_connect_cb,
    pub on_client_disconnect: stdio_bus_client_disconnect_cb,
    pub user_data: *mut c_void,
    pub log_level: c_int,
}

/// Statistics
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct stdio_bus_stats_t {
    pub messages_in: u64,
    pub messages_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub worker_restarts: u64,
    pub routing_errors: u64,
    pub client_connects: u64,
    pub client_disconnects: u64,
}

extern "C" {
    /// Create a new stdio_bus instance
    pub fn stdio_bus_create(options: *const stdio_bus_options_t) -> *mut stdio_bus_t;

    /// Start the bus (spawn workers)
    pub fn stdio_bus_start(bus: *mut stdio_bus_t) -> c_int;

    /// Process pending I/O (non-blocking)
    pub fn stdio_bus_step(bus: *mut stdio_bus_t, timeout_ms: c_int) -> c_int;

    /// Initiate graceful shutdown
    pub fn stdio_bus_stop(bus: *mut stdio_bus_t, timeout_sec: c_int) -> c_int;

    /// Destroy instance and free resources
    pub fn stdio_bus_destroy(bus: *mut stdio_bus_t);

    /// Send a message into the bus
    pub fn stdio_bus_ingest(bus: *mut stdio_bus_t, msg: *const c_char, len: usize) -> c_int;

    /// Get current bus state
    pub fn stdio_bus_get_state(bus: *const stdio_bus_t) -> stdio_bus_state_t;

    /// Get number of active workers
    pub fn stdio_bus_worker_count(bus: *const stdio_bus_t) -> c_int;

    /// Get number of active sessions
    pub fn stdio_bus_session_count(bus: *const stdio_bus_t) -> c_int;

    /// Get number of pending requests
    pub fn stdio_bus_pending_count(bus: *const stdio_bus_t) -> c_int;

    /// Get number of connected clients
    pub fn stdio_bus_client_count(bus: *const stdio_bus_t) -> c_int;

    /// Get the underlying event loop fd
    pub fn stdio_bus_get_poll_fd(bus: *const stdio_bus_t) -> c_int;

    /// Get statistics
    pub fn stdio_bus_get_stats(bus: *const stdio_bus_t, stats: *mut stdio_bus_stats_t);
}


#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Constants Tests
    // ============================================================================

    #[test]
    fn test_api_version() {
        assert_eq!(STDIO_BUS_EMBED_API_VERSION, 2);
    }

    #[test]
    fn test_return_codes() {
        assert_eq!(STDIO_BUS_OK, 0);
        assert_eq!(STDIO_BUS_ERR, -1);
        assert_eq!(STDIO_BUS_EAGAIN, -2);
        assert_eq!(STDIO_BUS_EOF, -3);
        assert_eq!(STDIO_BUS_EFULL, -4);
        assert_eq!(STDIO_BUS_ENOTFOUND, -5);
        assert_eq!(STDIO_BUS_EINVAL, -6);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(STDIO_BUS_ERR_CONFIG, -10);
        assert_eq!(STDIO_BUS_ERR_WORKER, -11);
        assert_eq!(STDIO_BUS_ERR_ROUTING, -12);
        assert_eq!(STDIO_BUS_ERR_BUFFER, -13);
        assert_eq!(STDIO_BUS_ERR_INVALID, -14);
        assert_eq!(STDIO_BUS_ERR_STATE, -15);
    }

    // ============================================================================
    // Enum Tests
    // ============================================================================

    #[test]
    fn test_bus_state_values() {
        assert_eq!(stdio_bus_state_t::STDIO_BUS_STATE_CREATED as i32, 0);
        assert_eq!(stdio_bus_state_t::STDIO_BUS_STATE_STARTING as i32, 1);
        assert_eq!(stdio_bus_state_t::STDIO_BUS_STATE_RUNNING as i32, 2);
        assert_eq!(stdio_bus_state_t::STDIO_BUS_STATE_STOPPING as i32, 3);
        assert_eq!(stdio_bus_state_t::STDIO_BUS_STATE_STOPPED as i32, 4);
    }

    #[test]
    fn test_listen_mode_values() {
        assert_eq!(stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_NONE as i32, 0);
        assert_eq!(stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_TCP as i32, 1);
        assert_eq!(stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_UNIX as i32, 2);
    }

    #[test]
    fn test_bus_state_equality() {
        let state1 = stdio_bus_state_t::STDIO_BUS_STATE_RUNNING;
        let state2 = stdio_bus_state_t::STDIO_BUS_STATE_RUNNING;
        let state3 = stdio_bus_state_t::STDIO_BUS_STATE_STOPPED;
        
        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }

    #[test]
    fn test_bus_state_copy() {
        let state = stdio_bus_state_t::STDIO_BUS_STATE_RUNNING;
        let copied = state;
        assert_eq!(state, copied);
    }

    #[test]
    fn test_bus_state_debug() {
        let state = stdio_bus_state_t::STDIO_BUS_STATE_RUNNING;
        let debug = format!("{:?}", state);
        assert!(debug.contains("RUNNING"));
    }

    // ============================================================================
    // Struct Tests
    // ============================================================================

    #[test]
    fn test_stats_default() {
        let stats = stdio_bus_stats_t::default();
        
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
    fn test_stats_clone() {
        let stats = stdio_bus_stats_t {
            messages_in: 100,
            messages_out: 50,
            bytes_in: 1000,
            bytes_out: 500,
            worker_restarts: 2,
            routing_errors: 1,
            client_connects: 10,
            client_disconnects: 5,
        };
        
        let cloned = stats.clone();
        assert_eq!(cloned.messages_in, 100);
        assert_eq!(cloned.messages_out, 50);
    }

    #[test]
    fn test_stats_debug() {
        let stats = stdio_bus_stats_t::default();
        let debug = format!("{:?}", stats);
        assert!(debug.contains("messages_in"));
    }

    #[test]
    fn test_listener_config_size() {
        // Should be able to create the struct
        let config = stdio_bus_listener_config_t {
            mode: stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_NONE,
            tcp_host: std::ptr::null(),
            tcp_port: 0,
            unix_path: std::ptr::null(),
        };
        
        assert_eq!(config.mode, stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_NONE);
        assert_eq!(config.tcp_port, 0);
    }

    #[test]
    fn test_options_struct() {
        use std::ptr;
        
        let listener = stdio_bus_listener_config_t {
            mode: stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_NONE,
            tcp_host: ptr::null(),
            tcp_port: 0,
            unix_path: ptr::null(),
        };
        
        let options = stdio_bus_options_t {
            config_path: ptr::null(),
            config_json: ptr::null(),
            listener,
            on_message: None,
            on_error: None,
            on_log: None,
            on_worker: None,
            on_client_connect: None,
            on_client_disconnect: None,
            user_data: ptr::null_mut(),
            log_level: 1,
        };
        
        assert_eq!(options.log_level, 1);
        assert!(options.on_message.is_none());
    }

    // ============================================================================
    // Callback Type Tests
    // ============================================================================

    #[test]
    fn test_callback_types_are_option() {
        // Verify callback types can be None
        let msg_cb: stdio_bus_message_cb = None;
        let err_cb: stdio_bus_error_cb = None;
        let log_cb: stdio_bus_log_cb = None;
        let worker_cb: stdio_bus_worker_cb = None;
        let connect_cb: stdio_bus_client_connect_cb = None;
        let disconnect_cb: stdio_bus_client_disconnect_cb = None;
        
        assert!(msg_cb.is_none());
        assert!(err_cb.is_none());
        assert!(log_cb.is_none());
        assert!(worker_cb.is_none());
        assert!(connect_cb.is_none());
        assert!(disconnect_cb.is_none());
    }

    // Note: We can't test actual FFI calls without linking to libstdio_bus
    // Those tests would be integration tests
}
