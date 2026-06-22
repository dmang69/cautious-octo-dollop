//! Ensures the intentos reference path does not depend on legacy IKRL daemon crates.

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use cargo_metadata::{MetadataCommand, PackageId};

const LEGACY_CRATE_NAMES: &[&str] = &[
    "capd",
    "intentd",
    "leasebroker",
    "eventscope",
    "eventscope-ebpf",
    "eventscope-lsm",
    "intentkernel-core",
    "intentkernel-crypto",
    "intentkernel-util",
    "intentkernel-sh",
    "ai-runtime",
    "intent-verifier",
    "ikrl-core",
    "ikrl-daemon",
];

fn intentos_metadata() -> cargo_metadata::Metadata {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    MetadataCommand::new()
        .manifest_path(manifest)
        .exec()
        .expect("read intentos workspace metadata")
}

fn resolve_deps(metadata: &cargo_metadata::Metadata, root: &PackageId) -> Vec<String> {
    let resolve = metadata
        .resolve
        .as_ref()
        .expect("metadata resolve graph");
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.clone());

    while let Some(id) = queue.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(pkg) = metadata.packages.iter().find(|p| &p.id == &id) {
            names.push(pkg.name.clone());
        }
        if let Some(node) = resolve.nodes.iter().find(|n| &n.id == &id) {
            for dep in &node.dependencies {
                queue.push_back(dep.clone());
            }
        }
    }
    names
}

#[test]
fn intentos_path_has_no_legacy_ikrl_dependencies() {
    let metadata = intentos_metadata();
    let intentos_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "intentos")
        .expect("intentos package");
    let deps = resolve_deps(&metadata, &intentos_pkg.id);
    for legacy in LEGACY_CRATE_NAMES {
        assert!(
            !deps.iter().any(|d| d == legacy),
            "intentos dependency graph must not include legacy crate `{legacy}`; found: {deps:?}"
        );
    }
    assert!(deps.iter().any(|d| d == "intentos-kernel"));
    assert!(deps.iter().any(|d| d == "intentos-shell"));
    assert!(deps.iter().any(|d| d == "intentos-utilities"));
}