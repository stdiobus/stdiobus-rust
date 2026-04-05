// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

#![cfg_attr(docsrs, feature(doc_cfg))]

//! Native FFI backend for stdio_bus
//!
//! This backend links directly to libstdio_bus.a and provides
//! the highest performance option for Unix systems.

use async_trait::async_trait;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use stdiobus_core::{Backend, BusMessage, BusState, BusStats, Error, Result};
use stdiobus_ffi::*;
use tokio::sync::{mpsc, Mutex};

/// Thread-safe wrapper for bus pointer
struct BusPtr(AtomicUsize);

impl BusPtr {
    fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    fn set(&self, ptr: *mut stdio_bus_t) {
        self.0.store(ptr as usize, Ordering::SeqCst);
    }

    fn get(&self) -> Option<*mut stdio_bus_t> {
        let ptr = self.0.load(Ordering::SeqCst);
        if ptr == 0 {
            None
        } else {
            Some(ptr as *mut stdio_bus_t)
        }
    }

    fn take(&self) -> Option<*mut stdio_bus_t> {
        let ptr = self.0.swap(0, Ordering::SeqCst);
        if ptr == 0 {
            None
        } else {
            Some(ptr as *mut stdio_bus_t)
        }
    }
}

unsafe impl Send for BusPtr {}
unsafe impl Sync for BusPtr {}

/// Wrapper for raw callback context pointer to allow storage in async Mutex.
/// Safety: The pointer is only accessed under Mutex guard and follows
/// strict lifecycle rules (see CallbackContext docs).
struct CtxPtr(*mut CallbackContext);
unsafe impl Send for CtxPtr {}
unsafe impl Sync for CtxPtr {}

fn state_to_u8(s: BusState) -> u8 {
    match s {
        BusState::Created => 0,
        BusState::Starting => 1,
        BusState::Running => 2,
        BusState::Stopping => 3,
        BusState::Stopped => 4,
    }
}

fn u8_to_state(v: u8) -> BusState {
    match v {
        0 => BusState::Created,
        1 => BusState::Starting,
        2 => BusState::Running,
        3 => BusState::Stopping,
        4 => BusState::Stopped,
        _ => BusState::Created,
    }
}

/// Context passed to C callbacks via user_data.
///
/// Safety: This context is shared with C callbacks via raw pointer.
/// The `alive` flag MUST be set to `false` before `stdio_bus_stop` is called,
/// and the context MUST NOT be freed until after `stdio_bus_destroy` completes.
struct CallbackContext {
    /// Set to false during shutdown to prevent callbacks from accessing Rust state
    alive: AtomicBool,
    message_tx: mpsc::Sender<BusMessage>,
    stats: Arc<Stats>,
}

/// Native backend using FFI to libstdio_bus
pub struct NativeBackend {
    bus: Arc<BusPtr>,
    config_path: String,
    state: Arc<AtomicU8>,
    message_tx: mpsc::Sender<BusMessage>,
    message_rx: Mutex<Option<mpsc::Receiver<BusMessage>>>,
    stats: Arc<Stats>,
    running: Arc<AtomicBool>,
    /// Owned callback context — freed only after C library is fully destroyed
    callback_ctx: Mutex<Option<CtxPtr>>,
}

struct Stats {
    messages_in: AtomicU64,
    messages_out: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    worker_restarts: AtomicU64,
    routing_errors: AtomicU64,
}

impl NativeBackend {
    pub fn new(config_path: &str) -> Result<Self> {
        let (tx, rx) = mpsc::channel(1000);

        Ok(Self {
            bus: Arc::new(BusPtr::new()),
            config_path: config_path.to_string(),
            state: Arc::new(AtomicU8::new(0)),
            message_tx: tx,
            message_rx: Mutex::new(Some(rx)),
            stats: Arc::new(Stats {
                messages_in: AtomicU64::new(0),
                messages_out: AtomicU64::new(0),
                bytes_in: AtomicU64::new(0),
                bytes_out: AtomicU64::new(0),
                worker_restarts: AtomicU64::new(0),
                routing_errors: AtomicU64::new(0),
            }),
            running: Arc::new(AtomicBool::new(false)),
            callback_ctx: Mutex::new(None),
        })
    }

    fn get_state(&self) -> BusState {
        u8_to_state(self.state.load(Ordering::SeqCst))
    }

    fn set_state(&self, state: BusState) {
        self.state.store(state_to_u8(state), Ordering::SeqCst);
    }
}

impl Drop for NativeBackend {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        // Phase 1: Signal callbacks to stop accessing Rust state
        if let Ok(guard) = self.callback_ctx.try_lock() {
            if let Some(ref wrapper) = *guard {
                unsafe { (*wrapper.0).alive.store(false, Ordering::SeqCst) };
            }
        }

        // Phase 2: Stop and destroy the C bus (no more callbacks after this)
        if let Some(bus) = self.bus.take() {
            unsafe {
                stdio_bus_stop(bus, 1);
                stdio_bus_destroy(bus);
            }
        }

        // Phase 3: Now safe to free the callback context
        if let Ok(mut guard) = self.callback_ctx.try_lock() {
            if let Some(wrapper) = guard.take() {
                unsafe { drop(Box::from_raw(wrapper.0)) };
            }
        }
    }
}


#[async_trait]
impl Backend for NativeBackend {
    async fn start(&self) -> Result<()> {
        let current_state = self.get_state();
        if !current_state.can_start() {
            return Err(Error::InvalidState {
                expected: "CREATED or STOPPED".to_string(),
                actual: current_state.to_string(),
            });
        }

        self.set_state(BusState::Starting);

        // Create callback context with alive flag for safe teardown
        let ctx = Box::new(CallbackContext {
            alive: AtomicBool::new(true),
            message_tx: self.message_tx.clone(),
            stats: self.stats.clone(),
        });
        let ctx_ptr = Box::into_raw(ctx);
        let ctx_usize = ctx_ptr as usize;

        // Store the pointer so we can free it on stop/drop
        *self.callback_ctx.lock().await = Some(CtxPtr(ctx_ptr));

        // Clone config path for the blocking task
        let config_path = self.config_path.clone();
        
        let bus = tokio::task::spawn_blocking(move || {
            let config_cstr = CString::new(config_path).map_err(|_| Error::InvalidArgument {
                message: "Invalid config path".to_string(),
            })?;

            let listener = stdio_bus_listener_config_t {
                mode: stdio_bus_listen_mode_t::STDIO_BUS_LISTEN_NONE,
                tcp_host: ptr::null(),
                tcp_port: 0,
                unix_path: ptr::null(),
            };

            let options = stdio_bus_options_t {
                config_path: config_cstr.as_ptr(),
                config_json: ptr::null(),
                listener,
                on_message: Some(on_message_callback),
                on_error: Some(on_error_callback),
                on_log: Some(on_log_callback),
                on_worker: None,
                on_client_connect: None,
                on_client_disconnect: None,
                user_data: ctx_usize as *mut c_void,
                log_level: 1,
            };

            let bus = unsafe { stdio_bus_create(&options) };
            if bus.is_null() {
                return Err(Error::InternalError {
                    message: "Failed to create bus".to_string(),
                });
            }

            let result = unsafe { stdio_bus_start(bus) };
            if result != STDIO_BUS_OK {
                unsafe { stdio_bus_destroy(bus) };
                return Err(Error::InternalError {
                    message: format!("Failed to start bus: error code {}", result),
                });
            }

            Ok(bus as usize)
        })
        .await
        .map_err(|e| Error::InternalError {
            message: format!("Task join error: {}", e),
        })??;

        self.bus.set(bus as *mut stdio_bus_t);
        self.set_state(BusState::Running);
        self.running.store(true, Ordering::SeqCst);

        // Start polling task
        let bus_ptr = self.bus.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                if let Some(bus) = bus_ptr.get() {
                    let bus_usize = bus as usize;
                    let _ = tokio::task::spawn_blocking(move || {
                        unsafe { stdio_bus_step(bus_usize as *mut stdio_bus_t, 10) };
                    })
                    .await;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        });

        Ok(())
    }

    async fn stop(&self, timeout_secs: u32) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.set_state(BusState::Stopping);

        // Phase 1: Signal callbacks to stop accessing Rust state
        {
            let guard = self.callback_ctx.lock().await;
            if let Some(ref wrapper) = *guard {
                unsafe { (*wrapper.0).alive.store(false, Ordering::SeqCst) };
            }
        }

        // Phase 2: Stop and destroy the C bus (no more callbacks after this)
        if let Some(bus) = self.bus.take() {
            let bus_usize = bus as usize;
            let timeout = timeout_secs as c_int;
            
            tokio::task::spawn_blocking(move || {
                unsafe {
                    stdio_bus_stop(bus_usize as *mut stdio_bus_t, timeout);
                    stdio_bus_destroy(bus_usize as *mut stdio_bus_t);
                }
            })
            .await
            .map_err(|e| Error::InternalError {
                message: format!("Task join error: {}", e),
            })?;
        }

        // Phase 3: Now safe to free the callback context
        {
            let mut guard = self.callback_ctx.lock().await;
            if let Some(wrapper) = guard.take() {
                unsafe { drop(Box::from_raw(wrapper.0)) };
            }
        }

        self.set_state(BusState::Stopped);
        Ok(())
    }

    async fn send(&self, message: &str) -> Result<()> {
        let bus = self.bus.get().ok_or_else(|| Error::InvalidState {
            expected: "RUNNING".to_string(),
            actual: "not initialized".to_string(),
        })?;

        let bus_usize = bus as usize;
        let msg = message.to_string();
        let msg_len = msg.len();

        let result = tokio::task::spawn_blocking(move || {
            unsafe {
                stdio_bus_ingest(
                    bus_usize as *mut stdio_bus_t,
                    msg.as_ptr() as *const c_char,
                    msg_len,
                )
            }
        })
        .await
        .map_err(|e| Error::InternalError {
            message: format!("Task join error: {}", e),
        })?;

        if result != STDIO_BUS_OK {
            return Err(Error::TransportError {
                message: format!("Failed to send message: error code {}", result),
            });
        }

        self.stats.messages_in.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_in.fetch_add(msg_len as u64, Ordering::Relaxed);

        Ok(())
    }

    fn state(&self) -> BusState {
        self.get_state()
    }

    fn stats(&self) -> BusStats {
        BusStats {
            messages_in: self.stats.messages_in.load(Ordering::Relaxed),
            messages_out: self.stats.messages_out.load(Ordering::Relaxed),
            bytes_in: self.stats.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.stats.bytes_out.load(Ordering::Relaxed),
            worker_restarts: self.stats.worker_restarts.load(Ordering::Relaxed),
            routing_errors: self.stats.routing_errors.load(Ordering::Relaxed),
            ..Default::default()
        }
    }

    fn worker_count(&self) -> i32 {
        self.bus
            .get()
            .map(|bus| unsafe { stdio_bus_worker_count(bus) })
            .unwrap_or(-1)
    }

    fn client_count(&self) -> i32 {
        self.bus
            .get()
            .map(|bus| unsafe { stdio_bus_client_count(bus) })
            .unwrap_or(0)
    }

    fn subscribe(&self) -> Option<mpsc::Receiver<BusMessage>> {
        self.message_rx.try_lock().ok().and_then(|mut rx| rx.take())
    }

    fn backend_type(&self) -> &'static str {
        "native"
    }
}


extern "C" fn on_message_callback(
    _bus: *mut stdio_bus_t,
    msg: *const c_char,
    len: usize,
    user_data: *mut c_void,
) {
    // Guard: catch any panic to prevent unwinding across FFI boundary
    let _ = std::panic::catch_unwind(|| {
        if user_data.is_null() {
            return;
        }

        let ctx = unsafe { &*(user_data as *const CallbackContext) };

        // Check alive flag — if shutting down, do not touch Rust state
        if !ctx.alive.load(Ordering::SeqCst) {
            return;
        }
        
        let slice = unsafe { std::slice::from_raw_parts(msg as *const u8, len) };
        if let Ok(json) = std::str::from_utf8(slice) {
            ctx.stats.messages_out.fetch_add(1, Ordering::Relaxed);
            ctx.stats.bytes_out.fetch_add(len as u64, Ordering::Relaxed);
            
            let message = BusMessage { json: json.to_string() };
            if let Err(e) = ctx.message_tx.try_send(message) {
                tracing::warn!("Message channel full: {}", e);
            }
        }
    });
}

extern "C" fn on_error_callback(
    _bus: *mut stdio_bus_t,
    code: c_int,
    msg: *const c_char,
    user_data: *mut c_void,
) {
    let _ = std::panic::catch_unwind(|| {
        if !user_data.is_null() {
            let ctx = unsafe { &*(user_data as *const CallbackContext) };
            if !ctx.alive.load(Ordering::SeqCst) {
                return;
            }
        }
        let msg = unsafe { CStr::from_ptr(msg) };
        tracing::error!("Bus error {}: {:?}", code, msg);
    });
}

extern "C" fn on_log_callback(
    _bus: *mut stdio_bus_t,
    level: c_int,
    msg: *const c_char,
    user_data: *mut c_void,
) {
    let _ = std::panic::catch_unwind(|| {
        if !user_data.is_null() {
            let ctx = unsafe { &*(user_data as *const CallbackContext) };
            if !ctx.alive.load(Ordering::SeqCst) {
                return;
            }
        }
        let msg = unsafe { CStr::from_ptr(msg) };
        match level {
            0 => tracing::debug!("{:?}", msg),
            1 => tracing::info!("{:?}", msg),
            2 => tracing::warn!("{:?}", msg),
            _ => tracing::error!("{:?}", msg),
        }
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_backend_new() {
        let result = NativeBackend::new("./test-config.json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_native_backend_initial_state() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        assert_eq!(backend.state(), BusState::Created);
    }

    #[test]
    fn test_native_backend_stats_initial() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        let stats = backend.stats();
        
        assert_eq!(stats.messages_in, 0);
        assert_eq!(stats.messages_out, 0);
        assert_eq!(stats.bytes_in, 0);
        assert_eq!(stats.bytes_out, 0);
    }

    #[test]
    fn test_native_backend_type() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        assert_eq!(backend.backend_type(), "native");
    }

    #[test]
    fn test_native_backend_worker_count_not_started() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        assert_eq!(backend.worker_count(), -1);
    }

    #[test]
    fn test_native_backend_client_count_not_started() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        assert_eq!(backend.client_count(), 0);
    }

    #[test]
    fn test_native_backend_subscribe() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        
        // First subscribe should succeed
        let rx = backend.subscribe();
        assert!(rx.is_some());
        
        // Second subscribe should fail
        let rx2 = backend.subscribe();
        assert!(rx2.is_none());
    }

    #[test]
    fn test_state_conversion() {
        assert_eq!(u8_to_state(0), BusState::Created);
        assert_eq!(u8_to_state(1), BusState::Starting);
        assert_eq!(u8_to_state(2), BusState::Running);
        assert_eq!(u8_to_state(3), BusState::Stopping);
        assert_eq!(u8_to_state(4), BusState::Stopped);
        assert_eq!(u8_to_state(255), BusState::Created);
        
        assert_eq!(state_to_u8(BusState::Created), 0);
        assert_eq!(state_to_u8(BusState::Starting), 1);
        assert_eq!(state_to_u8(BusState::Running), 2);
        assert_eq!(state_to_u8(BusState::Stopping), 3);
        assert_eq!(state_to_u8(BusState::Stopped), 4);
    }

    #[test]
    fn test_bus_ptr_operations() {
        let ptr = BusPtr::new();
        assert!(ptr.get().is_none());
        
        let fake_ptr = 0x12345678 as *mut stdio_bus_t;
        ptr.set(fake_ptr);
        
        assert!(ptr.get().is_some());
        assert_eq!(ptr.get().unwrap() as usize, 0x12345678);
        
        let taken = ptr.take();
        assert!(taken.is_some());
        assert!(ptr.get().is_none());
    }

    #[tokio::test]
    async fn test_native_backend_start_invalid_state() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        
        backend.state.store(state_to_u8(BusState::Running), Ordering::SeqCst);
        
        let result = backend.start().await;
        assert!(result.is_err());
        
        if let Err(Error::InvalidState { expected, actual }) = result {
            assert!(expected.contains("CREATED"));
            assert!(actual.contains("RUNNING"));
        }
    }

    #[tokio::test]
    async fn test_native_backend_send_not_started() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        
        let result = backend.send(r#"{"test": true}"#).await;
        assert!(result.is_err());
        
        if let Err(Error::InvalidState { .. }) = result {
            // Expected
        } else {
            panic!("Expected InvalidState error");
        }
    }

    #[tokio::test]
    async fn test_native_backend_stop_not_started() {
        let backend = NativeBackend::new("./test-config.json").unwrap();
        
        let result = backend.stop(1).await;
        assert!(result.is_ok());
        assert_eq!(backend.state(), BusState::Stopped);
    }
}
