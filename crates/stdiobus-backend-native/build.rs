// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026-present Raman Marozau <raman@worktif.com>
// Copyright (c) 2026-present stdiobus contributors

// Build script for stdiobus-backend-native
// Automatically selects the correct libstdio_bus.a based on target platform
//
// Supported targets:
//   - x86_64-unknown-linux-gnu
//   - aarch64-unknown-linux-gnu
//   - x86_64-apple-darwin
//   - aarch64-apple-darwin

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_path = PathBuf::from(&manifest_dir);
    let lib_base = crate_path.join("lib");
    
    // Get target triple (e.g., "x86_64-apple-darwin")
    let target = env::var("TARGET").unwrap_or_else(|_| {
        // Fallback: construct from OS + ARCH
        let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        match (os.as_str(), arch.as_str()) {
            ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
            ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
            _ => format!("{}-unknown-{}", arch, os),
        }
    });
    
    // Supported targets
    let supported = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ];
    
    // Find library path: lib/<target>/libstdio_bus.a
    let lib_dir = lib_base.join(&target);
    let lib_path = lib_dir.join("libstdio_bus.a");
    
    if !lib_path.exists() {
        // Check if any library exists (maybe old single-file layout)
        let fallback = lib_base.join("libstdio_bus.a");
        if fallback.exists() {
            // Use fallback (single library for current platform)
            println!("cargo:rustc-link-search=native={}", lib_base.display());
        } else {
            panic!(
                "\n\
                ╔══════════════════════════════════════════════════════════════╗\n\
                ║  libstdio_bus.a not found for target: {}                     \n\
                ╠══════════════════════════════════════════════════════════════╣\n\
                ║  Expected: {}                                                \n\
                ║                                                              \n\
                ║  Supported targets:                                          \n\
                ║    • x86_64-unknown-linux-gnu                                \n\
                ║    • aarch64-unknown-linux-gnu                               \n\
                ║    • x86_64-apple-darwin                                     \n\
                ║    • aarch64-apple-darwin                                    \n\
                ║                                                              \n\
                ║  Build libraries: make dist-lib                              \n\
                ╚══════════════════════════════════════════════════════════════╝\n",
                target,
                lib_path.display()
            );
        }
    } else {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }
    
    println!("cargo:rustc-link-lib=static=stdio_bus");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-changed=lib");
    
    // Platform-specific system libraries
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=pthread");
        }
        _ => {}
    }
    
    // Warn if target not in supported list
    if !supported.contains(&target.as_str()) {
        println!(
            "cargo:warning=Target '{}' is not officially supported. Build may fail.",
            target
        );
    }
}
