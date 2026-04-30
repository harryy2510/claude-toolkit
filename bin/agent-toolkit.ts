#!/usr/bin/env bun

import { runAgentToolkit } from '../src/cli.ts'

const exitCode = await runAgentToolkit(Bun.argv.slice(2))
process.exit(exitCode)
