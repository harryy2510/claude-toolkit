use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_git_repos(roots: &[PathBuf]) -> std::io::Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    for root in roots {
        walk(root, &mut repos)?;
    }
    repos.sort();
    repos.dedup();
    Ok(repos)
}

fn walk(path: &Path, repos: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.join(".git").is_dir() {
        repos.push(path.to_path_buf());
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if should_skip_dir(&name) {
            continue;
        }
        walk(&entry_path, repos)?;
    }
    Ok(())
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".turbo"
            | ".cache"
            | ".claude"
            | ".codex"
            | ".gemini"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-fleet-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn make_git_repo(path: &std::path::Path) {
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    }

    #[test]
    fn discover_git_repos_skips_ignored_directories() {
        let root = temp_dir();
        let app = root.join("app");
        let nested = root.join("packages/site");
        let ignored = root.join("node_modules/dep");
        make_git_repo(&app);
        make_git_repo(&nested);
        make_git_repo(&ignored);

        let repos = discover_git_repos(&[root]).unwrap();

        assert_eq!(repos, vec![app, nested]);
    }
}
