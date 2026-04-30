use std::path::Path;

use crate::check::{check_repo, RepoIssue};
use crate::hooks::bootstrap_repo;
use crate::intel::{write_repo_intel, RepoIntel};

pub struct RepoMigrationResult {
    pub created: Vec<String>,
    pub intel: RepoIntel,
    pub issues: Vec<RepoIssue>,
}

pub fn migrate_repo(root: &Path) -> std::io::Result<RepoMigrationResult> {
    let created = bootstrap_repo(root)?;
    let intel = write_repo_intel(root)?;
    let issues = check_repo(root);

    Ok(RepoMigrationResult {
        created,
        intel,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-migrate-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn migrate_repo_bootstraps_intel_and_checks_repo() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/index.ts"), "export const value = 1\n").unwrap();

        let result = migrate_repo(&root).unwrap();

        assert!(result.created.contains(&"AGENTS.md".to_string()));
        assert!(root.join(".agents/intel/summary.md").exists());
        assert_eq!(result.intel.file_count, 2);
        assert!(result.issues.is_empty());
    }
}
