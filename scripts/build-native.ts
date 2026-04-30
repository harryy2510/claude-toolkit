#!/usr/bin/env bun

import { copyFileSync, mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { binaryName, platformKey } from '../src/platform.ts'

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const key = process.env.AGENT_TOOLKIT_TARGET ?? platformKey()
const executable = binaryName(process.platform)
const source = join(packageRoot, 'target', 'release', executable)
const destinationDirectory = join(packageRoot, 'bin', 'native', key)
const destination = join(destinationDirectory, executable)

const build = Bun.spawnSync({
	cmd: ['cargo', 'build', '--release', '-p', 'agent-toolkit'],
	cwd: packageRoot,
	stdin: 'inherit',
	stdout: 'inherit',
	stderr: 'inherit',
})

if (build.exitCode !== 0) {
	process.exit(build.exitCode)
}

mkdirSync(destinationDirectory, { recursive: true })
copyFileSync(source, destination)
process.stdout.write(`wrote ${destination}\n`)
