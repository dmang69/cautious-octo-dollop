//! aya-based BPF loader for `bpf/eventscope.bpf.c`.
//!
//! Compiled only with `--features bpf`. On WSL2 without CAP_BPF, use mock tests
//! and `scripts/load-eventscope-bpf.sh` which documents prerequisites.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::bridge::BridgeError;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("BPF object not found at {0} — run scripts/load-eventscope-bpf.sh build")]
    ObjectMissing(PathBuf),
    #[error("aya loader failed: {0}")]
    Aya(String),
    #[error("bridge error: {0}")]
    Bridge(#[from] BridgeError),
}

/// Resolve BPF object path: `EVENTSCOPE_BPF_OBJ` or workspace default.
pub fn default_bpf_object_path() -> PathBuf {
    if let Ok(path) = std::env::var("EVENTSCOPE_BPF_OBJ") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/bpf/eventscope.bpf.o")
}

#[cfg(feature = "bpf")]
mod aya_impl {
    use std::collections::HashMap as StdHashMap;

    use super::*;
    use aya::maps::{HashMap as BpfHashMap, MapData};
    use aya::programs::{Lsm, ProgramError};
    use aya::{Btf, Ebpf, EbpfLoader, Pod};

    use crate::bridge::{replace_global_bridge, BridgeError, KernelBridge};
    use crate::policy::{
        HandleMapEntry, HANDLE_MAP_NAME, PROG_FILE_OPEN, PROG_SOCKET_CONNECT,
    };

    unsafe impl Pod for HandleMapEntry {}

    pub struct BpfKernelBridge {
        #[allow(dead_code)]
        bpf: Ebpf,
        handle_map: BpfHashMap<MapData, u32, HandleMapEntry>,
        cache: StdHashMap<u32, HandleMapEntry>,
    }

    fn load_lsm(bpf: &mut Ebpf, name: &str, hook: &str, btf: &Btf) -> Result<(), LoaderError> {
        let program: &mut Lsm = bpf
            .program_mut(name)
            .ok_or_else(|| LoaderError::Aya(format!("program {name} missing")))?
            .try_into()
            .map_err(|e: ProgramError| LoaderError::Aya(e.to_string()))?;
        program
            .load(hook, btf)
            .map_err(|e| LoaderError::Aya(e.to_string()))?;
        program
            .attach()
            .map(|_| ())
            .map_err(|e| LoaderError::Aya(e.to_string()))
    }

    impl BpfKernelBridge {
        pub fn load(object: &Path) -> Result<Self, LoaderError> {
            let mut bpf = EbpfLoader::new()
                .load_file(object)
                .map_err(|e| LoaderError::Aya(e.to_string()))?;
            let btf = Btf::from_sys_fs().map_err(|e| LoaderError::Aya(e.to_string()))?;

            load_lsm(&mut bpf, PROG_FILE_OPEN, "file_open", &btf)?;
            load_lsm(&mut bpf, PROG_SOCKET_CONNECT, "socket_connect", &btf)?;

            let handle_map = BpfHashMap::try_from(
                bpf.take_map(HANDLE_MAP_NAME)
                    .ok_or_else(|| LoaderError::Aya(format!("map {HANDLE_MAP_NAME} missing")))?,
            )
            .map_err(|e| LoaderError::Aya(e.to_string()))?;

            Ok(Self {
                bpf,
                handle_map,
                cache: StdHashMap::new(),
            })
        }
    }

    impl KernelBridge for BpfKernelBridge {
        fn is_loaded(&self) -> bool {
            true
        }

        fn publish(&mut self, entry: HandleMapEntry) -> Result<(), BridgeError> {
            if entry.pid == 0 || entry.handle == 0 {
                return Err(BridgeError::InvalidEntry);
            }
            self.handle_map
                .insert(entry.pid, entry, 0)
                .map_err(|e| BridgeError::Bpf(e.to_string()))?;
            self.cache.insert(entry.pid, entry);
            Ok(())
        }

        fn revoke(&mut self, pid: u32) -> Result<(), BridgeError> {
            let entry = HandleMapEntry::revoked(pid);
            self.handle_map
                .insert(pid, entry, 0)
                .map_err(|e| BridgeError::Bpf(e.to_string()))?;
            self.cache.insert(pid, entry);
            Ok(())
        }

        fn snapshot(&self) -> StdHashMap<u32, HandleMapEntry> {
            self.cache.clone()
        }
    }

    pub fn load_and_install(object: &Path) -> Result<(), LoaderError> {
        let bridge = BpfKernelBridge::load(object)?;
        replace_global_bridge(Box::new(bridge));
        Ok(())
    }
}

#[cfg(feature = "bpf")]
pub use aya_impl::load_and_install;

#[cfg(not(feature = "bpf"))]
pub fn load_and_install(object: &Path) -> Result<(), LoaderError> {
    let _ = object;
    Err(LoaderError::Aya(
        "crate built without `bpf` feature — rebuild with `cargo build -p eventscope-ebpf --features bpf`"
            .into(),
    ))
}

/// Attempt BPF load; returns structured error when object or privileges are missing.
pub fn try_load_bpf() -> Result<(), LoaderError> {
    let object = default_bpf_object_path();
    if !object.exists() {
        return Err(LoaderError::ObjectMissing(object));
    }
    load_and_install(&object)
}

/// Loader status for diagnostics (WSL2 / CI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderStatus {
    Ready,
    ObjectMissing(PathBuf),
    FeatureDisabled,
    LoadFailed(String),
}

pub fn probe_loader_status() -> LoaderStatus {
    let object = default_bpf_object_path();
    if !object.exists() {
        return LoaderStatus::ObjectMissing(object);
    }
    #[cfg(not(feature = "bpf"))]
    {
        return LoaderStatus::FeatureDisabled;
    }
    #[cfg(feature = "bpf")]
    {
        match try_load_bpf() {
            Ok(()) => LoaderStatus::Ready,
            Err(LoaderError::Aya(msg)) => LoaderStatus::LoadFailed(msg),
            Err(LoaderError::ObjectMissing(p)) => LoaderStatus::ObjectMissing(p),
            Err(LoaderError::Bridge(e)) => LoaderStatus::LoadFailed(e.to_string()),
        }
    }
}