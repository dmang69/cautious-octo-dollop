use std::collections::HashMap;

use intentos_kernel::KernelError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VfsError {
    #[error("kernel: {0}")]
    Kernel(#[from] KernelError),
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("path denied by capability scope: {0}")]
    ScopeDenied(String),
}

#[derive(Default)]
pub struct VirtualFs {
    files: HashMap<String, String>,
}

impl VirtualFs {
    pub fn new() -> Self {
        let mut fs = Self::default();
        fs.files.insert("/welcome.txt".into(), "IntentOS in-memory VFS".into());
        fs.files.insert("/notes.txt".into(), "event-scoped authority demo".into());
        fs
    }

    /// Seed a file for demo/test setup (bypasses capability gate).
    pub fn seed(&mut self, path: &str, content: &str) {
        self.files.insert(path.to_string(), content.to_string());
    }

    pub fn list(&self) -> Vec<String> {
        let mut paths: Vec<_> = self.files.keys().cloned().collect();
        paths.sort();
        paths
    }

    pub fn read_gated(
        &self,
        path: &str,
        handle: u64,
        kernel: &intentos_kernel::Kernel,
    ) -> Result<String, VfsError> {
        let bound = kernel
            .binding_resource(handle)
            .ok_or_else(|| VfsError::Kernel(KernelError::Denied("no binding".into())))?;
        if bound != path {
            return Err(VfsError::ScopeDenied(format!("token scope is {bound}, not {path}")));
        }
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| VfsError::NotFound(path.into()))
    }

    pub fn write_gated(
        &mut self,
        path: &str,
        content: &str,
        handle: u64,
        kernel: &intentos_kernel::Kernel,
    ) -> Result<(), VfsError> {
        let bound = kernel
            .binding_resource(handle)
            .ok_or_else(|| VfsError::Kernel(KernelError::Denied("no binding".into())))?;
        if bound != path {
            return Err(VfsError::ScopeDenied(format!("token scope is {bound}, not {path}")));
        }
        self.files.insert(path.to_string(), content.to_string());
        Ok(())
    }
}