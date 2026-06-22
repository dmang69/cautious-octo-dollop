use std::path::Path;

use anyhow::{bail, Result};

#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod stub;

/// Register a Windows service that launches `exe_path` with no extra arguments.
pub fn install_service(service_name: &str, display_name: &str, exe_path: &Path) -> Result<()> {
    install_impl(service_name, display_name, exe_path)
}

/// Remove a previously registered Windows service.
pub fn uninstall_service(service_name: &str) -> Result<()> {
    uninstall_impl(service_name)
}

/// Run `main_fn` under the Windows Service Control Manager, or execute it directly elsewhere.
pub fn run_as_service(service_name: &'static str, main_fn: fn() -> Result<()>) -> Result<()> {
    run_impl(service_name, main_fn)
}

/// Returns true when the Windows SCM has requested a service stop.
pub fn stop_requested() -> bool {
    stop_requested_impl()
}

#[cfg(windows)]
fn stop_requested_impl() -> bool {
    windows::stop_requested()
}

#[cfg(not(windows))]
fn stop_requested_impl() -> bool {
    stub::stop_requested()
}

#[cfg(windows)]
fn install_impl(service_name: &str, display_name: &str, exe_path: &Path) -> Result<()> {
    windows::install_service(service_name, display_name, exe_path)
}

#[cfg(windows)]
fn uninstall_impl(service_name: &str) -> Result<()> {
    windows::uninstall_service(service_name)
}

#[cfg(windows)]
fn run_impl(service_name: &'static str, main_fn: fn() -> Result<()>) -> Result<()> {
    windows::run_as_service(service_name, main_fn)
}

#[cfg(not(windows))]
fn install_impl(_service_name: &str, _display_name: &str, _exe_path: &Path) -> Result<()> {
    bail!("service install is only supported on Windows")
}

#[cfg(not(windows))]
fn uninstall_impl(_service_name: &str) -> Result<()> {
    bail!("service uninstall is only supported on Windows")
}

#[cfg(not(windows))]
fn run_impl(_service_name: &'static str, main_fn: fn() -> Result<()>) -> Result<()> {
    main_fn()
}