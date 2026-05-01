use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapChangeKind {
    Created,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapChange {
    pub kind: BootstrapChangeKind,
    pub path: String,
}

impl BootstrapChange {
    pub fn verb(&self) -> &'static str {
        match self.kind {
            BootstrapChangeKind::Created => "created",
            BootstrapChangeKind::Updated => "updated",
        }
    }

    fn created(path: &str) -> Self {
        Self {
            kind: BootstrapChangeKind::Created,
            path: path.to_string(),
        }
    }

    fn updated(path: &str) -> Self {
        Self {
            kind: BootstrapChangeKind::Updated,
            path: path.to_string(),
        }
    }
}

pub fn bootstrap_repo(root: &Path) -> std::io::Result<Vec<BootstrapChange>> {
    let mut changes = Vec::new();
    create_file_if_missing(
		root,
		"AGENTS.md",
		"# Repository Agent Instructions\n\nUse this file as the canonical source for coding agent guidance in this repo.\n",
		&mut changes,
	)?;
    create_file_if_missing(root, ".agents/agents.json", agents_json(), &mut changes)?;
    create_file_if_missing(root, ".agents/README.md", agents_readme(), &mut changes)?;
    create_file_if_missing(
        root,
        "scripts/agent-check",
        agent_check_script(),
        &mut changes,
    )?;
    ensure_hook(
        root,
        ".husky/pre-commit",
        pre_commit_hook(),
        "scripts/agent-check --staged",
        pre_commit_agent_check_block(),
        &mut changes,
    )?;
    ensure_hook(
        root,
        ".husky/pre-push",
        pre_push_hook(),
        "scripts/agent-check",
        pre_push_agent_check_block(),
        &mut changes,
    )?;
    ensure_hook(
        root,
        ".husky/commit-msg",
        commit_msg_hook(),
        "Conventional Commit",
        commit_msg_agent_check_block(),
        &mut changes,
    )?;
    ensure_gitignore_entries(root, &mut changes)?;
    make_executable(root.join("scripts/agent-check").as_path())?;
    make_executable(root.join(".husky/pre-commit").as_path())?;
    make_executable(root.join(".husky/pre-push").as_path())?;
    make_executable(root.join(".husky/commit-msg").as_path())?;
    configure_git_hooks_path(root);
    Ok(changes)
}

pub fn commit_msg_hook() -> &'static str {
    "#!/bin/sh\nset -eu\n\nfirst_line=$(sed -n '1p' \"$1\")\n\nif printf '%s\\n' \"$first_line\" | grep -Eq '^[a-z0-9-]+(\\([a-z0-9._/-]+\\))?!?: .+'; then\n\texit 0\nfi\n\necho \"Commit message must use Conventional Commit format, for example: feat: add repo intelligence\" >&2\nexit 1\n"
}

fn pre_commit_hook() -> &'static str {
    "#!/bin/sh\nset -eu\n\nscripts/agent-check --staged\n"
}

fn pre_push_hook() -> &'static str {
    "#!/bin/sh\nset -eu\n\nscripts/agent-check\n"
}

fn pre_commit_agent_check_block() -> &'static str {
    "# agent-toolkit:start\nscripts/agent-check --staged\n# agent-toolkit:end\n"
}

fn pre_push_agent_check_block() -> &'static str {
    "# agent-toolkit:start\nscripts/agent-check\n# agent-toolkit:end\n"
}

fn commit_msg_agent_check_block() -> &'static str {
    "# agent-toolkit:start\nfirst_line=$(sed -n '1p' \"$1\")\n\nif ! printf '%s\\n' \"$first_line\" | grep -Eq '^[a-z0-9-]+(\\([a-z0-9._/-]+\\))?!?: .+'; then\n\techo \"Commit message must use Conventional Commit format, for example: feat: add repo intelligence\" >&2\n\texit 1\nfi\n# agent-toolkit:end\n"
}

fn agent_check_script() -> &'static str {
    "#!/bin/sh\nset -eu\n\nif [ -z \"${AGENT_TOOLKIT_BIN:-}\" ] && ! command -v bunx >/dev/null 2>&1; then\n\techo \"agent-toolkit requires bunx. Install Bun, then rerun this command.\" >&2\n\texit 1\nfi\n\nif [ -n \"${AGENT_TOOLKIT_BIN:-}\" ]; then\n\t\"$AGENT_TOOLKIT_BIN\" repo check \"$@\"\nelse\n\tbunx @harryy/agent-toolkit repo check \"$@\"\nfi\n\nif [ \"${AGENT_TOOLKIT_SYNC_CHECK:-}\" = \"1\" ] && command -v agents >/dev/null 2>&1; then\n\tagents sync --path . --check\nfi\n"
}

fn agents_json() -> &'static str {
    "{\n\t\"schemaVersion\": 3,\n\t\"instructions\": {\n\t\t\"path\": \"AGENTS.md\"\n\t},\n\t\"integrations\": {\n\t\t\"enabled\": [],\n\t\t\"options\": {\n\t\t\t\"cursorAutoApprove\": true,\n\t\t\t\"antigravityGlobalSync\": false\n\t\t}\n\t},\n\t\"syncMode\": \"source-only\",\n\t\"mcp\": {\n\t\t\"servers\": {}\n\t},\n\t\"workspace\": {\n\t\t\"vscode\": {\n\t\t\t\"hideGenerated\": true,\n\t\t\t\"hiddenPaths\": [\n\t\t\t\t\"**/.codex\",\n\t\t\t\t\"**/.claude\",\n\t\t\t\t\"**/.gemini\",\n\t\t\t\t\"**/.cursor\",\n\t\t\t\t\"**/.antigravity\",\n\t\t\t\t\"**/.windsurf\",\n\t\t\t\t\"**/.opencode\",\n\t\t\t\t\"**/.junie\",\n\t\t\t\t\"**/opencode.json\",\n\t\t\t\t\"**/.agents/generated\",\n\t\t\t\t\"**/.agents/intel\"\n\t\t\t]\n\t\t}\n\t},\n\t\"lastSync\": null,\n\t\"lastSyncSourceHash\": null\n}\n"
}

fn agents_readme() -> &'static str {
    "# .agents\n\nProject-local source files for agent setup.\n\n- `agents.json`: cross-agent sync config\n- `intel/`: generated local repo intelligence, ignored by git\n- `local.json`: machine-specific overrides, ignored by git\n"
}

fn create_file_if_missing(
    root: &Path,
    relative_path: &str,
    contents: &str,
    changes: &mut Vec<BootstrapChange>,
) -> std::io::Result<()> {
    let path = root.join(relative_path);
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;
    changes.push(BootstrapChange::created(relative_path));
    Ok(())
}

fn ensure_hook(
    root: &Path,
    relative_path: &str,
    create_contents: &str,
    required_needle: &str,
    append_block: &str,
    changes: &mut Vec<BootstrapChange>,
) -> std::io::Result<()> {
    let path = root.join(relative_path);
    if !path.exists() {
        create_file_if_missing(root, relative_path, create_contents, changes)?;
        return Ok(());
    }

    let existing = fs::read_to_string(&path)?;
    if existing.contains(required_needle) {
        return Ok(());
    }

    let mut updated = existing.trim_end().to_string();
    if !updated.is_empty() {
        updated.push_str("\n\n");
    }
    updated.push_str(append_block);
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(path, updated)?;
    changes.push(BootstrapChange::updated(relative_path));
    Ok(())
}

fn ensure_gitignore_entries(
    root: &Path,
    changes: &mut Vec<BootstrapChange>,
) -> std::io::Result<()> {
    let path = root.join(".gitignore");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut updated = existing.trim_end().to_string();
    let mut changed = false;

    for entry in [".agents/intel/", ".agents/local.json", ".agents/generated/"] {
        if has_gitignore_entry(&existing, entry) {
            continue;
        }
        if !updated.is_empty() {
            updated.push('\n');
        }
        updated.push_str(entry);
        changed = true;
    }

    if changed {
        updated.push('\n');
        fs::write(path, updated)?;
        changes.push(BootstrapChange::updated(".gitignore"));
    }

    Ok(())
}

fn has_gitignore_entry(contents: &str, entry: &str) -> bool {
    contents
        .lines()
        .map(str::trim)
        .any(|line| line == entry || line == format!("/{entry}"))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if path.exists() {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn configure_git_hooks_path(root: &Path) {
    if !root.join(".git").exists() {
        return;
    }
    let _ = std::process::Command::new("git")
        .args(["config", "core.hooksPath", ".husky"])
        .current_dir(root)
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-hooks-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn commit_msg_hook_enforces_conventional_commits() {
        let hook = commit_msg_hook();

        assert!(hook.contains("grep -Eq"));
        assert!(hook.contains("Conventional Commit"));
    }

    #[test]
    fn bootstrap_repo_creates_agent_files_and_hooks() {
        let root = temp_dir();
        let changes = bootstrap_repo(&root).unwrap();

        assert!(has_change(
            &changes,
            BootstrapChangeKind::Created,
            "AGENTS.md"
        ));
        assert!(has_change(
            &changes,
            BootstrapChangeKind::Created,
            ".agents/agents.json"
        ));
        assert!(has_change(
            &changes,
            BootstrapChangeKind::Created,
            ".husky/pre-commit"
        ));
        assert!(has_change(
            &changes,
            BootstrapChangeKind::Created,
            ".husky/pre-push"
        ));
        assert!(has_change(
            &changes,
            BootstrapChangeKind::Created,
            ".husky/commit-msg"
        ));
        assert!(has_change(
            &changes,
            BootstrapChangeKind::Updated,
            ".gitignore"
        ));
        assert!(root.join(".husky/commit-msg").exists());
        let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gitignore.contains(".agents/intel/"));
        assert!(gitignore.contains(".agents/local.json"));
        assert!(gitignore.contains(".agents/generated/"));
    }

    #[test]
    fn bootstrap_repo_preserves_gitignore_and_deduplicates_agent_entries() {
        let root = temp_dir();
        fs::write(root.join(".gitignore"), "node_modules/\n.agents/intel/\n").unwrap();

        let first_changes = bootstrap_repo(&root).unwrap();
        let second_changes = bootstrap_repo(&root).unwrap();

        let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gitignore.contains("node_modules/"));
        assert_eq!(gitignore.matches(".agents/intel/").count(), 1);
        assert_eq!(gitignore.matches(".agents/local.json").count(), 1);
        assert_eq!(gitignore.matches(".agents/generated/").count(), 1);
        assert!(has_change(
            &first_changes,
            BootstrapChangeKind::Updated,
            ".gitignore"
        ));
        assert!(!has_change(
            &second_changes,
            BootstrapChangeKind::Updated,
            ".gitignore"
        ));
    }

    #[test]
    fn bootstrap_repo_integrates_existing_hooks_without_overwriting() {
        let root = temp_dir();
        fs::create_dir_all(root.join(".husky")).unwrap();
        fs::write(
            root.join(".husky/pre-commit"),
            "#!/bin/sh\nset -eu\n\nbun lint-staged --allow-empty\n",
        )
        .unwrap();

        let first_changes = bootstrap_repo(&root).unwrap();
        let second_changes = bootstrap_repo(&root).unwrap();

        let hook = fs::read_to_string(root.join(".husky/pre-commit")).unwrap();
        assert!(hook.contains("bun lint-staged --allow-empty"));
        assert_eq!(hook.matches("scripts/agent-check --staged").count(), 1);
        assert_eq!(hook.matches("# agent-toolkit:start").count(), 1);
        assert!(has_change(
            &first_changes,
            BootstrapChangeKind::Updated,
            ".husky/pre-commit"
        ));
        assert!(!has_change(
            &second_changes,
            BootstrapChangeKind::Updated,
            ".husky/pre-commit"
        ));
    }

    fn has_change(changes: &[BootstrapChange], kind: BootstrapChangeKind, path: &str) -> bool {
        changes
            .iter()
            .any(|change| change.kind == kind && change.path == path)
    }
}
