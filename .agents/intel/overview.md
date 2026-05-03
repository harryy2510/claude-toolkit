# Overview

## Stack Signals

- No framework signals detected from package metadata or file layout.

## Scale

- Source-like files: 26
- UI components: 0
- Routes: 1
- API endpoints/modules: 1
- SQL objects: 0
- Tests: 2

## Source Areas

- `crates/agent-toolkit-core`: 8
- `.`: 6
- `.github/workflows`: 2
- `crates/agent-toolkit-cli`: 2
- `bin`: 1
- `scripts/build-native.ts`: 1
- `scripts/bump-version.ts`: 1
- `src/cli.ts`: 1
- `src/native.ts`: 1
- `src/platform.ts`: 1
- `test/bump-version.test.ts`: 1
- `test/native.test.ts`: 1

## High-Impact Files

- `src/platform.ts` — imported by 3 local files
- `src/native.ts` — imported by 2 local files
- `src/cli.ts` — imported by 1 local files

## Largest Non-Generated Files

- `crates/agent-toolkit-core/src/intel.rs` — 5323 lines
- `crates/agent-toolkit-core/src/check.rs` — 758 lines
- `crates/agent-toolkit-core/src/global_setup.rs` — 639 lines
- `crates/agent-toolkit-core/src/hooks.rs` — 489 lines
- `crates/agent-toolkit-cli/src/main.rs` — 467 lines
- `README.md` — 439 lines
- `.github/workflows/release.yml` — 173 lines
- `src/native.ts` — 108 lines
- `test/native.test.ts` — 91 lines
- `crates/agent-toolkit-core/src/fleet.rs` — 90 lines
- `scripts/bump-version.ts` — 76 lines
- `crates/agent-toolkit-core/src/migrate.rs` — 59 lines
- `test/bump-version.test.ts` — 57 lines
- `AGENTS.md` — 45 lines
- `package.json` — 36 lines
- `scripts/build-native.ts` — 30 lines
- `.github/workflows/agent-check.yml` — 29 lines
- `tsconfig.json` — 29 lines
- `crates/agent-toolkit-core/Cargo.toml` — 17 lines
- `crates/agent-toolkit-cli/Cargo.toml` — 13 lines

## Exported Symbols Sample

- `crates/agent-toolkit-core/src/check.rs` — IssueCode, RepoIssue, check_repo, is_conventional_commit
- `crates/agent-toolkit-core/src/fleet.rs` — discover_git_repos
- `crates/agent-toolkit-core/src/global_setup.rs` — AgentDetection, GlobalSetupAction, GlobalSetupActionKind, GlobalSetupExtensionSkip, GlobalSetupOptions, GlobalSetupPlan, GlobalSetupResult, GlobalSetupSkip, apply_global_setup_plan, build_global_setup_plan, default_global_setup_options, detect_installed_agents, install_global_rules, upsert_managed_block
- `crates/agent-toolkit-core/src/hooks.rs` — BootstrapChange, BootstrapChangeKind, bootstrap_repo, commit_msg_hook, verb
- `crates/agent-toolkit-core/src/intel.rs` — RepoIntel, build_repo_intel, write_repo_intel
- `crates/agent-toolkit-core/src/lib.rs` — check, fleet, global_setup, hooks, intel, migrate
- `crates/agent-toolkit-core/src/migrate.rs` — RepoMigrationResult, migrate_repo
- `src/cli.ts` — runAgentToolkit
- `src/native.ts` — NativeCommandResolution, NativeRunOptions, findNativeBinary, nativeBinaryCandidates, nativePlatformKey, resolveNativeCommand, runNative
- `src/platform.ts` — binaryName, platformKey
