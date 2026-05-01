use std::fs;
use std::io::Write;
use std::path::Path;

pub fn bootstrap_repo(root: &Path) -> std::io::Result<Vec<String>> {
    let mut created = Vec::new();
    create_file_if_missing(
		root,
		"AGENTS.md",
		"# Repository Agent Instructions\n\nUse this file as the canonical source for coding agent guidance in this repo.\n",
		&mut created,
	)?;
    create_file_if_missing(root, ".agents/agents.json", agents_json(), &mut created)?;
    create_file_if_missing(root, ".agents/README.md", agents_readme(), &mut created)?;
    create_file_if_missing(
        root,
        "scripts/agent-check",
        agent_check_script(),
        &mut created,
    )?;
    create_file_if_missing(root, ".husky/pre-commit", pre_commit_hook(), &mut created)?;
    create_file_if_missing(root, ".husky/pre-push", pre_push_hook(), &mut created)?;
    create_file_if_missing(root, ".husky/commit-msg", commit_msg_hook(), &mut created)?;
    ensure_gitignore_entries(root)?;
    make_executable(root.join("scripts/agent-check").as_path())?;
    make_executable(root.join(".husky/pre-commit").as_path())?;
    make_executable(root.join(".husky/pre-push").as_path())?;
    make_executable(root.join(".husky/commit-msg").as_path())?;
    configure_git_hooks_path(root);
    Ok(created)
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
    created: &mut Vec<String>,
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
    created.push(relative_path.to_string());
    Ok(())
}

fn ensure_gitignore_entries(root: &Path) -> std::io::Result<()> {
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
        let created = bootstrap_repo(&root).unwrap();

        assert!(created.contains(&"AGENTS.md".to_string()));
        assert!(created.contains(&".agents/agents.json".to_string()));
        assert!(created.contains(&".husky/pre-commit".to_string()));
        assert!(created.contains(&".husky/pre-push".to_string()));
        assert!(created.contains(&".husky/commit-msg".to_string()));
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

        bootstrap_repo(&root).unwrap();
        bootstrap_repo(&root).unwrap();

        let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gitignore.contains("node_modules/"));
        assert_eq!(gitignore.matches(".agents/intel/").count(), 1);
        assert_eq!(gitignore.matches(".agents/local.json").count(), 1);
        assert_eq!(gitignore.matches(".agents/generated/").count(), 1);
    }
}
