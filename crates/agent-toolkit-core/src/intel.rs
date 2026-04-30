use std::fs;
use std::path::{Path, PathBuf};

pub struct RepoIntel {
    pub summary_markdown: String,
    pub file_count: usize,
}

pub fn write_repo_intel(root: &Path) -> std::io::Result<RepoIntel> {
    let intel = build_repo_intel(root)?;
    let intel_dir = root.join(".agents/intel");
    fs::create_dir_all(&intel_dir)?;
    fs::write(intel_dir.join("summary.md"), &intel.summary_markdown)?;
    fs::write(
        intel_dir.join("repo.json"),
        format!(
            "{{\n\t\"schemaVersion\": 1,\n\t\"fileCount\": {}\n}}\n",
            intel.file_count
        ),
    )?;
    Ok(intel)
}

pub fn build_repo_intel(root: &Path) -> std::io::Result<RepoIntel> {
    let files = collect_source_files(root)?;
    let has_codesight = root.join(".codesight/CODESIGHT.md").exists()
        || root.join(".codesight/wiki/index.md").exists();
    let mut summary = String::from("# Repository Intelligence\n\n");

    if has_codesight {
        summary.push_str("## Preferred Context\n\n");
        summary.push_str(
            "- CodeSight was found. Prefer its repo map before broad source exploration.\n",
        );
        if root.join(".codesight/wiki/index.md").exists() {
            summary.push_str("- Read `.codesight/wiki/index.md` first.\n");
        }
        if root.join(".codesight/CODESIGHT.md").exists() {
            summary.push_str("- Read `.codesight/CODESIGHT.md` next.\n");
        }
        summary.push_str("- Use focused `.codesight/*` files only as needed.\n\n");
    } else {
        summary.push_str("## Preferred Context\n\n");
        summary.push_str(
            "- CodeSight was not found. Use this generated repo intelligence summary first.\n\n",
        );
    }

    summary.push_str("## Source Files\n\n");
    summary.push_str(&format!("- Count: {}\n", files.len()));
    for file in files.iter().take(50) {
        summary.push_str(&format!("- `{}`\n", normalize_path(file)));
    }

    Ok(RepoIntel {
        summary_markdown: summary,
        file_count: files.len(),
    })
}

fn collect_source_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if entry_path.is_dir() {
                if should_skip_dir(&name) {
                    continue;
                }
                stack.push(entry_path);
                continue;
            }

            if is_source_file(&entry_path) {
                files.push(
                    entry_path
                        .strip_prefix(root)
                        .unwrap_or(&entry_path)
                        .to_path_buf(),
                );
            }
        }
    }

    files.sort();
    Ok(files)
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
            | ".agents"
            | ".codesight"
    )
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "rs" | "go" | "py" | "sql" | "md")
    )
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-intel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn build_repo_intel_prefers_codesight_when_present() {
        let root = temp_dir();
        let codesight = root.join(".codesight/wiki");
        fs::create_dir_all(&codesight).unwrap();
        fs::write(codesight.join("index.md"), "# CodeSight Index\n").unwrap();
        fs::write(root.join(".codesight/CODESIGHT.md"), "# Project Context\n").unwrap();

        let intel = build_repo_intel(&root).unwrap();

        assert!(intel.summary_markdown.contains("CodeSight"));
        assert!(intel.summary_markdown.contains(".codesight/wiki/index.md"));
        assert!(intel.summary_markdown.contains(".codesight/CODESIGHT.md"));
    }

    #[test]
    fn build_repo_intel_counts_source_files_and_hotspots() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src/components")).unwrap();
        fs::write(
            root.join("src/components/button.tsx"),
            "export const Button = () => null\n",
        )
        .unwrap();
        fs::write(
            root.join("src/index.ts"),
            "export * from './components/button'\n",
        )
        .unwrap();

        let intel = build_repo_intel(&root).unwrap();

        assert_eq!(intel.file_count, 2);
        assert!(intel.summary_markdown.contains("src/components/button.tsx"));
        assert!(intel.summary_markdown.contains("src/index.ts"));
    }

    #[test]
    fn write_repo_intel_creates_local_cache_files() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/index.ts"), "export const value = 1\n").unwrap();

        write_repo_intel(&root).unwrap();

        assert!(root.join(".agents/intel/summary.md").exists());
        assert!(root.join(".agents/intel/repo.json").exists());
    }
}
