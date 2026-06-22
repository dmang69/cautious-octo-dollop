use intentkernel_crypto::hash::context_hash;
use intentkernel_crypto::token::{FileScope, NetworkScope, ResourceScope};

/// Resource type identifiers aligned with RFC-INTENT-001 / UCCS capability classes.
pub const RESOURCE_FILE: u32 = 1;
pub const RESOURCE_NETWORK: u32 = 2;
pub const RESOURCE_RAW: u32 = 3;

/// File access mode from token scope (`access` field).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccess {
    Read = 1,
    Write = 2,
    ReadWrite = 3,
}

impl FileAccess {
    pub fn permits(self, requested: FileAccess) -> bool {
        match (self, requested) {
            (FileAccess::ReadWrite, _) => true,
            (a, b) => a == b,
        }
    }
}

pub fn resource_type_for_scope(scope: &ResourceScope) -> u32 {
    match scope {
        ResourceScope::File(_) => RESOURCE_FILE,
        ResourceScope::Network(_) => RESOURCE_NETWORK,
        ResourceScope::Raw(_) => RESOURCE_RAW,
    }
}

/// Normalize paths for comparison (collapse `.` segments, trim trailing `/`).
pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut parts: Vec<&str> = Vec::new();
    for component in trimmed.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    if trimmed.starts_with('/') {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    }
}

pub fn scope_matches_file(scope: &FileScope, path: &str, access: FileAccess) -> bool {
    let requested = normalize_path(path);
    let allowed = normalize_path(&scope.path);
    if requested != allowed {
        return false;
    }
    let scope_access = match scope.access {
        1 => FileAccess::Read,
        2 => FileAccess::Write,
        3 => FileAccess::ReadWrite,
        _ => FileAccess::Read,
    };
    scope_access.permits(access)
}

pub fn parse_ip_bytes(addr: &str) -> Option<Vec<u8>> {
    let host = addr.split(':').next()?.trim();
    if host.contains('.') {
        let octets: Option<Vec<u8>> = host
            .split('.')
            .map(|p| p.parse::<u8>().ok())
            .collect();
        return octets;
    }
    if host.contains(':') {
        return parse_ipv6(host);
    }
    None
}

fn parse_ipv6(host: &str) -> Option<Vec<u8>> {
    let mut parts: Vec<&str> = host.split(':').collect();
    if let Some(idx) = parts.iter().position(|p| p.is_empty()) {
        let missing = 8usize.saturating_sub(parts.len() - 1);
        let mut expanded = Vec::with_capacity(8);
        for (i, p) in parts.iter().enumerate() {
            if i == idx {
                for _ in 0..missing {
                    expanded.push("0");
                }
            } else if !p.is_empty() {
                expanded.push(*p);
            }
        }
        parts = expanded;
    }
    if parts.len() != 8 {
        return None;
    }
    let mut out = Vec::with_capacity(16);
    for p in parts {
        let v = u16::from_str_radix(p, 16).ok()?;
        out.extend_from_slice(&v.to_be_bytes());
    }
    Some(out)
}

pub fn scope_matches_network(scope: &NetworkScope, dst: &[u8], port: u16, proto: u32) -> bool {
    if scope.dst_port != port {
        return false;
    }
    if scope.proto != 0 && scope.proto != proto {
        return false;
    }
    scope.dst_ip == dst
}

pub fn scope_matches_raw(scope: &[u8], action: &[u8]) -> bool {
    if scope.is_empty() {
        return false;
    }
    if scope.len() == 48 {
        return scope == context_hash(action);
    }
    scope == action
}

pub fn scope_matches_resource(
    scope: &ResourceScope,
    path: Option<&str>,
    file_access: FileAccess,
    network_dst: Option<&[u8]>,
    network_port: Option<u16>,
    network_proto: u32,
    raw_action: Option<&[u8]>,
) -> bool {
    match scope {
        ResourceScope::File(file) => {
            path.is_some_and(|p| scope_matches_file(file, p, file_access))
        }
        ResourceScope::Network(net) => network_dst
            .zip(network_port)
            .is_some_and(|(dst, port)| scope_matches_network(net, dst, port, network_proto)),
        ResourceScope::Raw(bytes) => raw_action.is_some_and(|a| scope_matches_raw(bytes, a)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_normalization() {
        assert_eq!(normalize_path("/foo/bar/"), "/foo/bar");
        assert_eq!(normalize_path("/foo//bar"), "/foo/bar");
    }

    #[test]
    fn file_scope_exact_match() {
        let scope = FileScope {
            path: "/data/secret.txt".into(),
            access: 2,
            inode: None,
        };
        assert!(scope_matches_file(&scope, "/data/secret.txt", FileAccess::Write));
        assert!(!scope_matches_file(&scope, "/data/secret.txt", FileAccess::Read));
        assert!(!scope_matches_file(&scope, "/data/other.txt", FileAccess::Write));
    }

    #[test]
    fn network_scope_match() {
        let scope = NetworkScope {
            proto: 1,
            dst_ip: vec![10, 0, 0, 1],
            dst_port: 443,
            bytes: 1024,
        };
        assert!(scope_matches_network(&scope, &[10, 0, 0, 1], 443, 1));
        assert!(!scope_matches_network(&scope, &[10, 0, 0, 2], 443, 1));
    }
}