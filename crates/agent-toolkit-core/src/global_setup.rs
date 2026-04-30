use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GlobalSetupResult {
    pub updated_files: Vec<PathBuf>,
    pub linked_extensions: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDetection {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSetupOptions {
    pub all: bool,
    pub include_gemini: bool,
    pub detection: AgentDetection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSetupPlan {
    pub dotclaude_repo: PathBuf,
    pub actions: Vec<GlobalSetupAction>,
    pub skipped: Vec<GlobalSetupSkip>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSetupAction {
    pub agent: String,
    pub description: String,
    pub kind: GlobalSetupActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalSetupActionKind {
    ManagedRules { path: PathBuf, block_id: String },
    GeminiExtensionLink { source: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSetupSkip {
    pub agent: String,
    pub reason: String,
}

pub fn install_global_rules(
    home: &Path,
    dotclaude_repo: &Path,
) -> std::io::Result<GlobalSetupResult> {
    let plan = build_global_setup_plan(
        home,
        dotclaude_repo,
        GlobalSetupOptions {
            all: true,
            include_gemini: true,
            detection: detect_installed_agents(home),
        },
    );
    apply_global_setup_plan(&plan)
}

pub fn detect_installed_agents(home: &Path) -> AgentDetection {
    AgentDetection {
        claude: home.join(".claude").exists() || command_exists("claude"),
        codex: home.join(".codex").exists() || command_exists("codex"),
        gemini: command_exists("gemini"),
    }
}

pub fn default_global_setup_options(home: &Path) -> GlobalSetupOptions {
    GlobalSetupOptions {
        all: false,
        include_gemini: true,
        detection: detect_installed_agents(home),
    }
}

pub fn build_global_setup_plan(
    home: &Path,
    dotclaude_repo: &Path,
    options: GlobalSetupOptions,
) -> GlobalSetupPlan {
    let mut actions = Vec::new();
    let mut skipped = Vec::new();

    push_managed_rules_action(
        &mut actions,
        &mut skipped,
        options.all || options.detection.claude,
        "Claude",
        home.join(".claude/CLAUDE.md"),
        "Update ~/.claude/CLAUDE.md managed rules block",
    );
    push_managed_rules_action(
        &mut actions,
        &mut skipped,
        options.all || options.detection.codex,
        "Codex",
        home.join(".codex/AGENTS.md"),
        "Update ~/.codex/AGENTS.md managed rules block",
    );

    if !options.include_gemini {
        skipped.push(GlobalSetupSkip {
            agent: "Gemini".to_string(),
            reason: "disabled by --skip-gemini".to_string(),
        });
    } else if options.all || options.detection.gemini {
        let source = dotclaude_repo.join("plugins/dotclaude/gemini-extension");
        if source.exists() {
            actions.push(GlobalSetupAction {
                agent: "Gemini".to_string(),
                description: "Link DotClaude Gemini extension".to_string(),
                kind: GlobalSetupActionKind::GeminiExtensionLink { source },
            });
        } else {
            skipped.push(GlobalSetupSkip {
                agent: "Gemini".to_string(),
                reason: "DotClaude Gemini extension source was not found".to_string(),
            });
        }
    } else {
        skipped.push(GlobalSetupSkip {
            agent: "Gemini".to_string(),
            reason: "Gemini CLI was not detected".to_string(),
        });
    }

    GlobalSetupPlan {
        dotclaude_repo: dotclaude_repo.to_path_buf(),
        actions,
        skipped,
    }
}

pub fn apply_global_setup_plan(plan: &GlobalSetupPlan) -> std::io::Result<GlobalSetupResult> {
    let rules = fs::read_to_string(plan.dotclaude_repo.join("plugins/dotclaude/AGENTS.md"))?;
    let mut updated_files = Vec::new();
    let mut linked_extensions = Vec::new();

    for action in &plan.actions {
        match &action.kind {
            GlobalSetupActionKind::ManagedRules { path, block_id } => {
                upsert_file_block(path, block_id, &rules)?;
                updated_files.push(path.clone());
            }
            GlobalSetupActionKind::GeminiExtensionLink { source } => {
                let status = Command::new("gemini")
                    .args(["extensions", "link"])
                    .arg(source)
                    .arg("--consent")
                    .status()?;
                if !status.success() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "failed to link Gemini extension",
                    ));
                }
                linked_extensions.push(source.clone());
            }
        }
    }

    Ok(GlobalSetupResult {
        updated_files,
        linked_extensions,
    })
}

pub fn upsert_managed_block(existing: &str, id: &str, content: &str) -> String {
    let start = start_marker(id);
    let end = end_marker(id);
    let block = format!("{start}\n{}\n{end}", content.trim());

    if let Some(start_index) = existing.find(&start) {
        if let Some(relative_end_index) = existing[start_index..].find(&end) {
            let end_index = start_index + relative_end_index + end.len();
            let mut result = String::new();
            result.push_str(existing[..start_index].trim_end());
            result.push_str("\n\n");
            result.push_str(&block);
            result.push_str(existing[end_index..].trim_end_matches('\n'));
            result.push('\n');
            return result;
        }
    }

    let mut result = existing.trim_end().to_string();
    if !result.is_empty() {
        result.push_str("\n\n");
    }
    result.push_str(&block);
    result.push('\n');
    result
}

fn upsert_file_block(path: &Path, id: &str, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existing = fs::read_to_string(path).unwrap_or_default();
    fs::write(path, upsert_managed_block(&existing, id, content))
}

fn push_managed_rules_action(
    actions: &mut Vec<GlobalSetupAction>,
    skipped: &mut Vec<GlobalSetupSkip>,
    include: bool,
    agent: &str,
    path: PathBuf,
    description: &str,
) {
    if include {
        actions.push(GlobalSetupAction {
            agent: agent.to_string(),
            description: description.to_string(),
            kind: GlobalSetupActionKind::ManagedRules {
                path,
                block_id: "DOTCLAUDE".to_string(),
            },
        });
    } else {
        skipped.push(GlobalSetupSkip {
            agent: agent.to_string(),
            reason: format!("{} was not detected", agent),
        });
    }
}

fn start_marker(id: &str) -> String {
    format!("<!-- AGENT-TOOLKIT:{id}:START -->")
}

fn end_marker(id: &str) -> String {
    format!("<!-- AGENT-TOOLKIT:{id}:END -->")
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|directory| {
        let candidate = directory.join(command);
        if candidate.is_file() {
            return true;
        }

        if cfg!(windows) {
            return ["exe", "cmd", "bat"]
                .iter()
                .any(|extension| directory.join(format!("{command}.{extension}")).is_file());
        }

        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_managed_block_preserves_user_content() {
        let result =
            upsert_managed_block("# User Rules\n\nKeep this.\n", "DOTCLAUDE", "# Shared\n");

        assert!(result.contains("# User Rules"));
        assert!(result.contains("Keep this."));
        assert!(result.contains("AGENT-TOOLKIT:DOTCLAUDE:START"));
        assert!(result.contains("# Shared"));
    }

    #[test]
    fn upsert_managed_block_replaces_existing_block() {
        let first = upsert_managed_block("prefix\n", "DOTCLAUDE", "old");
        let result = upsert_managed_block(&first, "DOTCLAUDE", "new");

        assert!(result.contains("prefix"));
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
        assert_eq!(result.matches("AGENT-TOOLKIT:DOTCLAUDE:START").count(), 1);
    }

    #[test]
    fn build_global_setup_plan_only_targets_detected_agents() {
        let root = temp_dir("agent-toolkit-global-plan");
        let dotclaude = root.join("dotclaude");
        fs::create_dir_all(dotclaude.join("plugins/dotclaude/gemini-extension")).unwrap();
        let plan = build_global_setup_plan(
            &root,
            &dotclaude,
            GlobalSetupOptions {
                all: false,
                include_gemini: true,
                detection: AgentDetection {
                    claude: true,
                    codex: false,
                    gemini: true,
                },
            },
        );

        assert_eq!(plan.actions.len(), 2);
        assert!(plan.actions.iter().any(|action| action.agent == "Claude"));
        assert!(plan.actions.iter().any(|action| action.agent == "Gemini"));
        assert!(plan.skipped.iter().any(|skip| skip.agent == "Codex"));
    }

    #[test]
    fn apply_global_setup_plan_preserves_user_rules_with_managed_blocks() {
        let root = temp_dir("agent-toolkit-global-apply");
        let dotclaude = root.join("dotclaude/plugins/dotclaude");
        fs::create_dir_all(&dotclaude).unwrap();
        fs::write(dotclaude.join("AGENTS.md"), "# Shared Rules\n").unwrap();
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude")).unwrap();
        fs::write(home.join(".claude/CLAUDE.md"), "# My Existing Rules\n").unwrap();
        let plan = build_global_setup_plan(
            &home,
            &root.join("dotclaude"),
            GlobalSetupOptions {
                all: false,
                include_gemini: false,
                detection: AgentDetection {
                    claude: true,
                    codex: false,
                    gemini: false,
                },
            },
        );

        let result = apply_global_setup_plan(&plan).unwrap();
        let claude = fs::read_to_string(home.join(".claude/CLAUDE.md")).unwrap();

        assert_eq!(result.updated_files, vec![home.join(".claude/CLAUDE.md")]);
        assert!(result.linked_extensions.is_empty());
        assert!(claude.contains("# My Existing Rules"));
        assert!(claude.contains("# Shared Rules"));
        assert!(claude.contains("AGENT-TOOLKIT:DOTCLAUDE:START"));
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "{prefix}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
