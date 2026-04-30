# .agents

This repository uses `@agents-dev/cli` as the shared source of truth for agent instructions.

Commit these files:

- `../AGENTS.md`
- `agents.json`

Do not commit local or generated outputs:

- `local.json`
- `generated/`
- `intel/`
- root `CLAUDE.md`
- tool folders such as `.codex/`, `.claude/`, `.cursor/`, `.gemini/`, `.windsurf/`, `.opencode/`

Useful commands:

```bash
agent-toolkit repo migrate
agent-toolkit repo check
agent-toolkit repo sync --check
agents status --path .
agents sync --path .
agents sync --path . --check
agents watch --path .
```

Note: Windsurf writes user-level MCP config when synced. Antigravity is enabled for generated snapshots, but global sync is disabled by default.
