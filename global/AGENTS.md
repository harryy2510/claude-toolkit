<!-- DOTAGENT:START -->
# Global Agent Guidelines

These rules apply to every agent session and every project unless a more specific repository instruction overrides them.

## Default Execution Mode

Default to speed mode unless the user explicitly asks for deep review, exhaustive testing, E2E validation, release readiness, or careful verification.

- Use repository instructions and existing patterns first.
- Use skills only when they materially reduce risk or the user explicitly requests them.
- For straightforward code changes, do not run skill ceremonies first.
- Use parallel subagents only for clearly independent work when supported and allowed.
- Do not use Browser Use or Computer Use unless explicitly requested or required for the task.
- Do not run full test suites unless explicitly requested, preparing a commit/PR/release, or touching broad shared behavior.
- Run the smallest targeted check that covers changed behavior.
- Skip checks only for docs-only or trivial edits, and state that checks were skipped.
- Keep tracker/doc updates to concise bullets.
- Timebox investigation to about 5 minutes before making a concrete edit plan.
- Timebox blockers to about 10 minutes, then record the blocker and move on or ask for direction.

## Scope And Precedence

- Read repository instructions first: `AGENTS.md`, `CLAUDE.md`, `README.md`, and equivalent local guidance.
- If multiple instruction files exist, follow the most specific one first, then broader guidance.
- Local project rules override these global defaults.
- Tool-specific global files should point to this shared policy where possible.

## Startup And Routing

- For speed mode, skip agent-routing unless the user asks for role routing or the task is broad enough to need subagents.
- If agent-routing is used and the host supports native subagents, invoke the matching specialist only for self-contained work that can run independently. If native subagents are unavailable or disallowed, use the matching role profile as lightweight guidance.
- Do not spawn `agents-orchestrator` as a child subagent when it needs to call other agents; keep orchestration in the main thread and invoke specialists directly.
- Treat the `skills:` list in an agent profile as optional context unless the user asks for that role, the task is high-risk, or the skill materially reduces risk.
- Use skills only when they materially reduce risk or the user explicitly requests them.
- For speed-mode implementation, do not run skill ceremonies before straightforward code changes.
- Use repository instructions and existing patterns first.

## Hard Rules

### Git

- Never run `git push` unless the user explicitly asks for it in the current message.
- Permission to push expires after that one push.
- Never push to `main` or `master` unless the user says the exact words and confirms after you warn them.
- Never skip git hooks with `--no-verify`, `--no-gpg-sign`, or similar flags.
- Any commit you create must use Conventional Commit format, for example `feat: add repo intelligence` or `fix(cli): preserve user files`.
- Create git worktrees inside the same repository under `.worktrees/`.
- Keep `.worktrees/` gitignored.

### Environment Files

- Never edit, create, overwrite, or delete any file whose name starts with `.env`.
- This includes `.env`, `.env.local`, `.env.development`, `.env.production`, `.env.keys`, `.env.vault`, `.env.example`, encrypted env files, and every other variant.
- If the user needs env changes, tell them exactly what to change themselves.

### Toolchain

- Use Bun for JavaScript and TypeScript package management and execution.
- Use TypeScript for new JavaScript-platform code. Do not create `.js` or `.jsx` source files.
- Do not use `tsc` as the project check command. Use `oxlint --type-aware --type-check`.
- Use `oxlint` instead of ESLint.
- Use `oxfmt` instead of Prettier.
- Use Husky for git hooks. Do not introduce `.githooks` or ad hoc hook folders.
- GitHub Actions workflows must use Node.js 24 with `actions/setup-node@v6` when a Node runtime is needed.
- Do not add npm, yarn, pnpm, npx, ESLint, Prettier, or `tsc --noEmit` workflows.

### Package Scripts

- Do not add, modify, or remove `package.json` scripts unless the user explicitly asks or the current task is package/tooling setup.
- Use existing scripts when they are present.
- If a needed script does not exist and the user did not ask for tooling changes, ask before adding one.

### Supabase

- Never run direct Supabase CLI commands in repos that require package scripts for Supabase.
- Use existing package scripts, such as `bun db-push-and-types`, when the repository provides them.
- Never hand-write types for Supabase tables, views, functions, or RPCs when generated database types are available.
- SQL functions must set `search_path = ''` or explicit schemas and schema-qualify references such as `public.users` and `auth.uid()`.

### Generated Files

- Do not edit generated files unless explicitly requested.
- Generated database types, Worker configuration types, route trees, and agent sync outputs should be regenerated through the owning tool.

## Common Skill Routing

- New project or tooling setup: `toolchain`, `project-setup`, `scaffold`
- Codebase exploration or reviews: `repo-intelligence`, `deslop`
- Agent delegation or role selection: `agent-routing`
- Debugging, regressions, flaky tests, incidents: `debugging`, `testing`
- React UI: `ui`, `shadcn`, `react-best-practices`
- Forms: `forms-rhf-zod`
- Data fetching: `react-query-mutative`
- Client state: `zustand-x-ui-state`
- Routes and server functions: `tanstack-start-cloudflare`
- Supabase auth, data, migrations, RLS: `supabase-auth-data`, `supabase-postgres-best-practices`
- Cloudflare Workers and Wrangler: `cloudflare`
- Vite: `vite`
- Testing: `testing`

## Completion

- For speed mode, run only targeted checks unless the user asks for full verification, commit, PR, or release readiness.
- Before PR/commit, run the relevant package checks. Full repo checks are optional unless requested.
- Run `/skill-lint` or `plugins/dotagent/scripts/skill-lint.sh` when agent, skill, command, or plugin instruction files change.
- Run `bunx @harryy/agent-toolkit repo check` in agentized repos once the toolkit is available.
- Run `agents sync --check` when `AGENTS.md` or `.agents/` changed.
- If checks are not run, state why.
<!-- DOTAGENT:END -->
