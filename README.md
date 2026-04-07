```
 ██████╗██╗      █████╗ ██╗   ██╗██████╗ ███████╗
██╔════╝██║     ██╔══██╗██║   ██║██╔══██╗██╔════╝
██║     ██║     ███████║██║   ██║██║  ██║█████╗
██║     ██║     ██╔══██║██║   ██║██║  ██║██╔══╝
╚██████╗███████╗██║  ██║╚██████╔╝██████╔╝███████╗
 ╚═════╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝
████████╗ ██████╗  ██████╗ ██╗     ██╗  ██╗██╗████████╗
╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██║ ██╔╝██║╚══██╔══╝
   ██║   ██║   ██║██║   ██║██║     █████╔╝ ██║   ██║
   ██║   ██║   ██║██║   ██║██║     ██╔═██╗ ██║   ██║
   ██║   ╚██████╔╝╚██████╔╝███████╗██║  ██╗██║   ██║
   ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝
```

**Curated Claude Code plugins by Hariom Sharma**

[![Claude Code](https://img.shields.io/badge/Claude_Code-Plugin_Marketplace-7C3AED?style=flat-square&logo=anthropic&logoColor=white)](https://claude.ai) [![Plugins](https://img.shields.io/badge/Plugins-2-22C55E?style=flat-square)](https://github.com/harryy2510/claude-toolkit) [![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

---

## What Is This?

Claude Toolkit is a **plugin marketplace** for Claude Code. It is a curated registry that points to plugin repositories hosted on GitHub. Each plugin ships skills, agents, and commands that extend Claude Code with battle-tested conventions and automation.

```
+------------------------------------------------------+
|                  claude-toolkit                       |
|                  (marketplace)                        |
|                                                       |
|   +-------------------+   +----------------------+   |
|   |    dotclaude       |   |    vibe-pilot        |   |
|   |    16 skills       |   |    4 skills          |   |
|   |    19 agents       |   |    3 commands         |   |
|   |    6 commands      |   |    kanban autopilot  |   |
|   +-------------------+   +----------------------+   |
+------------------------------------------------------+
```

---

## Quick Install

```bash
# 1. Add the marketplace
claude plugin marketplace add harryy2510/claude-toolkit

# 2. Install the plugins you want
claude plugin install dotclaude@claude-toolkit
claude plugin install vibe-pilot@claude-toolkit
```

---

## Plugin Catalog

| Plugin | Description | What You Get | Install |
|--------|-------------|--------------|---------|
| **dotclaude** | Code conventions, agents, and tooling for all projects | 16 skills, 19 agents, 6 commands | `claude plugin install dotclaude@claude-toolkit` |
| **vibe-pilot** | AI kanban autopilot -- classify, triage, and status reports | 4 skills, 3 commands | `claude plugin install vibe-pilot@claude-toolkit` |

---

## dotclaude

> Code conventions, 19 specialist agents, 16 skills, and tooling for all projects.

```
+------------------------------------------------------------------+
|  dotclaude                                                        |
|  "Load a skill, not your whole brain."                           |
+------------------------------------------------------------------+
|                                                                    |
|  SKILLS (16)              AGENTS (19)           COMMANDS (6)      |
|  ----------------         ----------------      ----------------  |
|  ui                       engineer              /setup            |
|  forms-rhf-zod            tester                /update           |
|  react-query-mutative     product               /uninstall        |
|  zustand-x-ui-state       orchestrator          /repo-map         |
|  tanstack-start-cf        reviewer              /skill-lint       |
|  supabase-auth-data       debugger              /deslop           |
|  cloudflare               security              |                 |
|  project-setup            performance           |                 |
|  scaffold                 ...and 11 more        |                 |
|  react-best-practices                           |                 |
|  conventions                                    |                 |
|  shadcn                                         |                 |
|  supabase-postgres                              |                 |
|  vite                                           |                 |
|  repo-map                                       |                 |
|  deslop                                         |                 |
|                                                                    |
+------------------------------------------------------------------+
```

### Skills at a Glance

| Category | Skills | What They Cover |
|----------|--------|-----------------|
| **UI & Components** | `ui`, `shadcn`, `forms-rhf-zod` | Tailwind v4, shadcn/base-ui, CVA, dark mode, react-hook-form + zod |
| **State & Data** | `zustand-x-ui-state`, `react-query-mutative` | zustand-x v6, React Query, optimistic updates, key factories |
| **Frameworks** | `tanstack-start-cloudflare`, `vite`, `cloudflare` | Routing, server functions, SSR, Workers, Wrangler config |
| **Backend** | `supabase-auth-data`, `supabase-postgres-best-practices` | 3 Supabase clients, auth flow, RLS, migrations, query optimization |
| **Quality** | `react-best-practices`, `conventions`, `deslop` | 57 perf rules, ESLint/Prettier enforcement, AI slop detection |
| **Tooling** | `project-setup`, `scaffold`, `repo-map` | DX tooling, fullstack scaffolding, codebase symbol indexing |

### Agents at a Glance

19 specialist agents covering engineering, testing, product management, orchestration, code review, debugging, security analysis, performance optimization, and more. Each agent has a focused role and knows when to defer to another.

### Install

```bash
claude plugin install dotclaude@claude-toolkit
```

Full documentation: [github.com/harryy2510/dotclaude](https://github.com/harryy2510/dotclaude)

---

## vibe-pilot

> Zero-touch kanban autopilot. Discovers repos, classifies tasks, triages complex work, picks what to build, and launches AI workspaces.

```
+------------------------------------------------------------------+
|  vibe-pilot                                                       |
|  "Set it and forget it."                                         |
+------------------------------------------------------------------+
|                                                                    |
|  SKILLS (4)               COMMANDS (3)                            |
|  ----------------         ----------------                        |
|  classify                 /classify                               |
|  triage                   /triage                                 |
|  implement                /status-report                          |
|  status-report                                                    |
|                                                                    |
+------------------------------------------------------------------+
|                                                                    |
|  HOW IT WORKS                                                     |
|  ---------------------------------------------------------------- |
|                                                                    |
|    Discover repos  -->  Classify tasks  -->  Triage complex work  |
|         |                                          |               |
|         v                                          v               |
|    Pick next task  <--  Launch workspace  <--  Break into steps   |
|         |                                          |               |
|         v                                          v               |
|    Implement  -------->  Commit & update  ---->  Next cycle       |
|                                                                    |
+------------------------------------------------------------------+
```

### Skills

| Skill | What It Does |
|-------|--------------|
| **classify** | Scans repos and classifies kanban tasks by complexity, domain, and priority |
| **triage** | Breaks complex tasks into actionable subtasks with dependencies |
| **implement** | Picks the next task, launches an AI workspace, and builds it |
| **status-report** | Generates progress reports across all tracked projects |

### Install

```bash
claude plugin install vibe-pilot@claude-toolkit
```

Full documentation: [github.com/harryy2510/vibe-pilot](https://github.com/harryy2510/vibe-pilot)

---

## Author

**Hariom Sharma** -- [github.com/harryy2510](https://github.com/harryy2510)

---

## License

MIT
