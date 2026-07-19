/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

// Platform
#[cfg(target_os = "windows")]
pub const PLATFORM_NAME: &str = "Windows";
#[cfg(target_os = "macos")]
pub const PLATFORM_NAME: &str = "macOS";
#[cfg(target_os = "android")]
pub const PLATFORM_NAME: &str = "Android";
#[cfg(target_os = "linux")]
pub const PLATFORM_NAME: &str = "Linux";
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
pub const PLATFORM_NAME: &str = "BSD";
#[cfg(all(
    target_family = "unix",
    not(any(
        target_os = "macos",
        target_os = "android",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))
))]
pub const PLATFORM_NAME: &str = "Unix";
#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "android",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_family = "unix"
)))]
pub const PLATFORM_NAME: &str = "Unknown Platform";

// CPU Architecture
#[cfg(target_arch = "x86_64")]
pub const ARCH_NAME: &str = "x86_64";
#[cfg(target_arch = "x86")]
pub const ARCH_NAME: &str = "x86";
#[cfg(target_arch = "aarch64")]
pub const ARCH_NAME: &str = "ARM64";
#[cfg(target_arch = "arm")]
pub const ARCH_NAME: &str = "ARM";
#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
pub const ARCH_NAME: &str = "RISC-V";
#[cfg(target_arch = "powerpc64")]
pub const ARCH_NAME: &str = "PowerPC64";
#[cfg(target_arch = "powerpc")]
pub const ARCH_NAME: &str = "PowerPC";
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "riscv64",
    target_arch = "riscv32",
    target_arch = "powerpc64",
    target_arch = "powerpc"
)))]
pub const ARCH_NAME: &str = "Unknown Arch";

// Build type
#[cfg(not(debug_assertions))]
pub const BUILD_MODE: &str = "Release";
#[cfg(debug_assertions)]
pub const BUILD_MODE: &str = "Debug";
