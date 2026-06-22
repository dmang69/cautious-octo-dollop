use std::ffi::OsString;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn install_service(service_name: &str, display_name: &str, exe_path: &Path) -> Result<()> {
    let exe = exe_path
        .canonicalize()
        .with_context(|| format!("resolve executable path {}", exe_path.display()))?;
    let bin_path = format!("\"{}\"", exe.display());

    let status = std::process::Command::new("sc.exe")
        .args([
            "create",
            service_name,
            &format!("binPath= {bin_path}"),
            "start= auto",
            &format!("DisplayName= {display_name}"),
        ])
        .output()
        .context("sc create")?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let stdout = String::from_utf8_lossy(&status.stdout);
        if !stderr.contains("1073") && !stdout.contains("1073") {
            bail!("sc create failed: {stdout}{stderr}");
        }
    }

    let _ = std::process::Command::new("sc.exe")
        .args(["description", service_name, display_name])
        .status();

    Ok(())
}

pub fn uninstall_service(service_name: &str) -> Result<()> {
    let _ = std::process::Command::new("sc.exe")
        .args(["stop", service_name])
        .status();

    std::thread::sleep(Duration::from_millis(500));

    let status = std::process::Command::new("sc.exe")
        .args(["delete", service_name])
        .output()
        .context("sc delete")?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let stdout = String::from_utf8_lossy(&status.stdout);
        if !stderr.contains("1060") && !stdout.contains("1060") {
            bail!("sc delete failed: {stdout}{stderr}");
        }
    }

    Ok(())
}

pub fn run_as_service(service_name: &'static str, main_fn: fn() -> Result<()>) -> Result<()> {
    match service_dispatcher::start(service_name, move |arguments| {
        service_main(service_name, main_fn, arguments)
    }) {
        Ok(()) => Ok(()),
        Err(windows_service::Error::Winapi(err))
            if err.raw_os_error() == Some(1063) =>
        {
            main_fn()
        }
        Err(err) => Err(anyhow::anyhow!("service dispatcher failed: {err}")),
    }
}

fn service_main(
    service_name: &'static str,
    main_fn: fn() -> Result<()>,
    _arguments: Vec<OsString>,
) {
    if let Err(err) = run_service_loop(service_name, main_fn) {
        eprintln!("{err:#}");
    }
}

fn run_service_loop(service_name: &'static str, main_fn: fn() -> Result<()>) -> Result<()> {
    STOP_REQUESTED.store(false, Ordering::SeqCst);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_for_handler = Arc::clone(&stop_flag);

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                STOP_REQUESTED.store(true, Ordering::SeqCst);
                stop_for_handler.store(true, Ordering::SeqCst);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle =
        service_control_handler::register(service_name, event_handler).map_err(|err| {
            anyhow::anyhow!("register service control handler: {err}")
        })?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|err| anyhow::anyhow!("set service status (running): {err}"))?;

    let result = main_fn();

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: if result.is_ok() {
                ServiceExitCode::Win32(0)
            } else {
                ServiceExitCode::ServiceSpecific(1)
            },
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|err| anyhow::anyhow!("set service status (stopped): {err}"))?;

    result
}

pub fn stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::SeqCst)
}