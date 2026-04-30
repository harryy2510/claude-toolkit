use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum IssueCode {
    MissingAgentsMd,
    MissingAgentsConfig,
    MissingAgentCheckScript,
    MissingGitHook,
    DisallowedTrackedFile,
    JavaScriptSource,
    TscUsage,
    EslintUsage,
    PrettierUsage,
    InvalidCommitMessage,
    SlopPattern,
    HardcodedSecret,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RepoIssue {
    pub code: IssueCode,
    pub message: String,
}

pub fn check_repo(root: &Path) -> Vec<RepoIssue> {
    let mut issues = Vec::new();

    if !root.join("AGENTS.md").exists() {
        issues.push(RepoIssue {
            code: IssueCode::MissingAgentsMd,
            message: "AGENTS.md is required as the canonical repo instruction file".to_string(),
        });
    }

    if !root.join(".agents/agents.json").exists() {
        issues.push(RepoIssue {
            code: IssueCode::MissingAgentsConfig,
            message: ".agents/agents.json is required for cross-agent sync".to_string(),
        });
    }
    push_missing_file_issue(
		&mut issues,
		root,
		"scripts/agent-check",
		IssueCode::MissingAgentCheckScript,
		"scripts/agent-check is required so hooks and CI can run repo-local checks without global installs",
	);
    for hook in [".husky/pre-commit", ".husky/pre-push", ".husky/commit-msg"] {
        push_missing_file_issue(
            &mut issues,
            root,
            hook,
            IssueCode::MissingGitHook,
            "repo Husky hooks must be committed and wired by bootstrap",
        );
    }

    push_disallowed_tracked_file_issues(root, &mut issues);

    let package_json = root.join("package.json");
    if let Ok(contents) = fs::read_to_string(package_json) {
        push_tooling_issue(
            &mut issues,
            &contents,
            "tsc",
            IssueCode::TscUsage,
            "use oxlint --type-aware --type-check instead of tsc",
        );
        push_tooling_issue(
            &mut issues,
            &contents,
            "eslint",
            IssueCode::EslintUsage,
            "use oxlint instead of ESLint",
        );
        push_tooling_issue(
            &mut issues,
            &contents,
            "prettier",
            IssueCode::PrettierUsage,
            "use oxfmt instead of Prettier",
        );
    }

    if contains_javascript_source(root) {
        issues.push(RepoIssue {
            code: IssueCode::JavaScriptSource,
            message: "new source code must be TypeScript, not JavaScript".to_string(),
        });
    }

    issues.extend(scan_slop(root));

    issues
}

pub fn is_conventional_commit(message: &str) -> bool {
    let first_line = message.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return false;
    }

    let Some((prefix, description)) = first_line.split_once(": ") else {
        return false;
    };
    if description.trim().is_empty() {
        return false;
    }

    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);
    let (commit_type, scope) = if let Some(open_index) = prefix.find('(') {
        if !prefix.ends_with(')') || open_index == 0 {
            return false;
        }
        (
            &prefix[..open_index],
            Some(&prefix[open_index + 1..prefix.len() - 1]),
        )
    } else {
        (prefix, None)
    };

    if !is_slug(commit_type) {
        return false;
    }

    if let Some(scope) = scope {
        if scope.is_empty() || !is_scope(scope) {
            return false;
        }
    }

    true
}

fn push_missing_file_issue(
    issues: &mut Vec<RepoIssue>,
    root: &Path,
    relative_path: &str,
    code: IssueCode,
    message: &str,
) {
    if !root.join(relative_path).exists() {
        issues.push(RepoIssue {
            code,
            message: format!("{relative_path}: {message}"),
        });
    }
}

fn push_tooling_issue(
    issues: &mut Vec<RepoIssue>,
    contents: &str,
    needle: &str,
    code: IssueCode,
    message: &str,
) {
    if contents.contains(needle) {
        issues.push(RepoIssue {
            code,
            message: message.to_string(),
        });
    }
}

fn contains_javascript_source(root: &Path) -> bool {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        for entry in entries.flatten() {
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

            if matches!(
                entry_path
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("js" | "jsx")
            ) {
                return true;
            }
        }
    }
    false
}

fn scan_slop(root: &Path) -> Vec<RepoIssue> {
    let mut issues = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        for entry in entries.flatten() {
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
            let display_path = entry_path.strip_prefix(root).unwrap_or(&entry_path);
            if !is_text_source(&entry_path) || should_skip_slop_file(display_path) {
                continue;
            }
            let Ok(contents) = fs::read_to_string(&entry_path) else {
                continue;
            };
            let display_path = display_path.display();
            if has_secret_pattern(&contents) {
                issues.push(RepoIssue {
                    code: IssueCode::HardcodedSecret,
                    message: format!("{display_path} appears to contain a hardcoded secret"),
                });
            }
            if has_slop_pattern(&contents) {
                issues.push(RepoIssue {
                    code: IssueCode::SlopPattern,
                    message: format!(
                        "{display_path} contains debug, placeholder, or empty error handling code"
                    ),
                });
            }
        }
    }
    issues
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
            | ".claude"
            | ".codex"
            | ".gemini"
            | ".cursor"
            | ".antigravity"
            | ".windsurf"
            | ".opencode"
            | ".junie"
    )
}

fn is_text_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "rs" | "go" | "py" | "sql" | "sh")
    )
}

fn should_skip_slop_file(relative_path: &Path) -> bool {
    let path = relative_path.to_string_lossy();
    path.contains("/skills/deslop/")
        || path.ends_with("/deslop.sh")
        || path == "plugins/dotclaude/scripts/deslop.sh"
}

fn has_secret_pattern(contents: &str) -> bool {
    [
        concat!("sk", "_live_"),
        concat!("AK", "IA"),
        concat!("gh", "p_"),
        concat!("xo", "xb-"),
        concat!("SUPABASE", "_SERVICE_ROLE_KEY="),
    ]
    .iter()
    .any(|pattern| contents.contains(pattern))
}

fn has_slop_pattern(contents: &str) -> bool {
    [
        concat!("console", ".debug("),
        concat!("debugger", ";"),
        concat!("throw new Error(\"", "TODO"),
        concat!("throw new Error('", "TODO"),
        concat!("lorem", " ipsum"),
    ]
    .iter()
    .any(|pattern| contents.contains(pattern))
}

fn push_disallowed_tracked_file_issues(root: &Path, issues: &mut Vec<RepoIssue>) {
    for file in tracked_files(root) {
        if !root.join(&file).exists() {
            continue;
        }
        if is_disallowed_tracked_path(&file) {
            issues.push(RepoIssue {
                code: IssueCode::DisallowedTrackedFile,
                message: format!("{file} must stay local/generated and should not be tracked"),
            });
        }
    }
}

fn tracked_files(root: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter_map(|path| std::str::from_utf8(path).ok())
        .filter(|path| !path.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn is_disallowed_tracked_path(path: &str) -> bool {
    path == "CLAUDE.md"
        || path == "opencode.json"
        || path == ".agents/local.json"
        || path.starts_with(".agents/generated/")
        || path.starts_with(".agents/intel/")
        || path.starts_with(".codex/")
        || path.starts_with(".claude/")
        || path.starts_with(".cursor/")
        || path.starts_with(".gemini/")
        || path.starts_with(".antigravity/")
        || path.starts_with(".windsurf/")
        || path.starts_with(".opencode/")
        || path.starts_with(".junie/")
        || path.starts_with("docs/superpowers/")
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn is_scope(value: &str) -> bool {
    value.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '-' | '_' | '.' | '/')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-check-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn conventional_commit_accepts_common_valid_messages() {
        assert!(is_conventional_commit("feat: add repo intelligence"));
        assert!(is_conventional_commit("fix(cli): preserve user files"));
        assert!(is_conventional_commit("chore!: drop legacy setup"));
    }

    #[test]
    fn conventional_commit_rejects_freeform_messages() {
        assert!(!is_conventional_commit("updated stuff"));
        assert!(!is_conventional_commit("Fix thing"));
        assert!(!is_conventional_commit(""));
    }

    #[test]
    fn check_repo_reports_missing_agent_files() {
        let root = temp_dir();
        let issues = check_repo(&root);

        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::MissingAgentsMd));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::MissingAgentsConfig));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::MissingAgentCheckScript));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::MissingGitHook));
    }

    #[test]
    fn check_repo_reports_disallowed_tooling_for_new_standard() {
        let root = temp_dir();
        fs::create_dir_all(root.join(".agents")).unwrap();
        fs::write(root.join("AGENTS.md"), "# Rules\n").unwrap();
        fs::write(root.join(".agents/agents.json"), "{}\n").unwrap();
        fs::write(
			root.join("package.json"),
			r#"{"scripts":{"check":"tsc --noEmit","lint":"eslint .","format":"prettier . --check"}}"#,
		)
		.unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/index.js"),
            concat!("console", ".log('bad')\n"),
        )
        .unwrap();

        let issues = check_repo(&root);

        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::JavaScriptSource));
        assert!(issues.iter().any(|issue| issue.code == IssueCode::TscUsage));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::EslintUsage));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::PrettierUsage));
    }

    #[test]
    fn check_repo_reports_deslop_patterns() {
        let root = temp_dir();
        fs::create_dir_all(root.join(".agents")).unwrap();
        fs::write(root.join("AGENTS.md"), "# Rules\n").unwrap();
        fs::write(root.join(".agents/agents.json"), "{}\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/index.ts"),
            concat!(
                "console",
                ".debug('debug')\nconst key = 'sk",
                "_live_secret'\n"
            ),
        )
        .unwrap();

        let issues = check_repo(&root);

        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::SlopPattern));
        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::HardcodedSecret));
    }

    #[test]
    fn check_repo_reports_disallowed_tracked_generated_files() {
        let root = temp_dir();
        write_minimal_repo_files(&root);
        fs::create_dir_all(root.join(".git")).unwrap();
        let _ = Command::new("git")
            .arg("init")
            .current_dir(&root)
            .output()
            .unwrap();
        fs::write(root.join("CLAUDE.md"), "# Generated\n").unwrap();
        fs::write(root.join(".agents/local.json"), "{}\n").unwrap();
        let _ = Command::new("git")
            .args([
                "add",
                "AGENTS.md",
                ".agents/agents.json",
                "CLAUDE.md",
                ".agents/local.json",
            ])
            .current_dir(&root)
            .output()
            .unwrap();

        let issues = check_repo(&root);

        assert!(issues
            .iter()
            .any(|issue| issue.code == IssueCode::DisallowedTrackedFile));
    }

    fn write_minimal_repo_files(root: &Path) {
        fs::create_dir_all(root.join(".agents")).unwrap();
        fs::create_dir_all(root.join(".husky")).unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(root.join("AGENTS.md"), "# Rules\n").unwrap();
        fs::write(root.join(".agents/agents.json"), "{}\n").unwrap();
        fs::write(root.join("scripts/agent-check"), "#!/bin/sh\n").unwrap();
        fs::write(root.join(".husky/pre-commit"), "#!/bin/sh\n").unwrap();
        fs::write(root.join(".husky/pre-push"), "#!/bin/sh\n").unwrap();
        fs::write(root.join(".husky/commit-msg"), "#!/bin/sh\n").unwrap();
    }
}
