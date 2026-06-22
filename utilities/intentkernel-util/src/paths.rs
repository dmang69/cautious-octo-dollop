use std::path::{Path, PathBuf};

pub fn resolve_root(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    if let Ok(dir) = std::env::var("INTENTKERNEL_ROOT") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("INTENTKERNEL_DEV_ROOT") {
        return PathBuf::from(dir);
    }
    for candidate in dev_root_candidates() {
        if looks_like_dev_tree(&candidate) {
            return candidate;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn looks_like_dev_tree(root: &Path) -> bool {
    root.join("core/ai-runtime/proto/intentkernel.proto").is_file()
        || root.join("kernel/intentkernel-core/Cargo.toml").is_file()
}

pub fn dev_root_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(windows)]
    {
        candidates.push(PathBuf::from(
            r"C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
        ));
        if let Ok(dir) = std::env::var("USERPROFILE") {
            let profile = PathBuf::from(dir);
            candidates.push(profile.join("Documents/GitHub/cautious-octo-dollop"));
            candidates.push(profile.join("CLionProjects/cautious-octo-dollop"));
            candidates.push(profile.join("cautious-octo-dollop"));
        }
        candidates.push(PathBuf::from(
            r"C:\Users\Dizzle\CLionProjects\cautious-octo-dollop",
        ));
        candidates.push(PathBuf::from(r"C:\Users\Dizzle\cautious-octo-dollop"));
        candidates.push(PathBuf::from(r"D:\intentkernel"));
        candidates.push(PathBuf::from(r"D:\cautious-octo-dollop"));
    }

    if let Ok(dir) = std::env::var("HOME") {
        candidates.push(PathBuf::from(dir).join("cautious-octo-dollop"));
    }

    candidates
}