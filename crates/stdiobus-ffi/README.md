# stdiobus-ffi

[![Crates.io](https://img.shields.io/crates/v/stdiobus-ffi.svg)](https://crates.io/crates/stdiobus-ffi)
[![License](https://img.shields.io/crates/l/stdiobus-ffi)](LICENSE)

FFI bindings to libstdio_bus C library.

This crate provides raw, unsafe FFI bindings to the stdio_bus C library. Most users should use `stdiobus-client` with the `native` feature instead.

## Prerequisites

Requires `libstdio_bus.a` built from the main stdio_bus repository:

```bash
git clone https://github.com/stdiobus/stdiobus-rust
cd stdiobus-rust
make lib
```

## Installation

```toml
[dependencies]
stdiobus-ffi = "1.0"
```

Set environment variables for linking:

```bash
export STDIO_BUS_LIB_DIR=/path/to/stdiobus/build
```

## Usage

```rust
use stdiobus_ffi::*;

unsafe {
    let mut opts = stdio_bus_options_t::default();
    opts.config_path = c"config.json".as_ptr();
    
    let bus = stdio_bus_create(&opts);
    if !bus.is_null() {
        stdio_bus_start(bus);
        // ...
        stdio_bus_destroy(bus);
    }
}
```

## Safety

All functions in this crate are unsafe. Use `stdiobus-client` for a safe API.

## License

Apache-2.0
