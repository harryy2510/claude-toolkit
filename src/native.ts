import { existsSync } from 'node:fs'
import { join } from 'node:path'

import { binaryName, platformKey } from './platform.ts'

export type NativeRunOptions = {
	packageRoot: string
	env?: Record<string, string | undefined>
	platform?: NodeJS.Platform
	arch?: string
}

export type NativeCommandResolution = {
	command: Array<string> | null
	error: string | null
}

export function nativePlatformKey(options: Pick<NativeRunOptions, 'arch' | 'platform'> = {}): string {
	return platformKey(options.platform, options.arch)
}

export function nativeBinaryCandidates(options: NativeRunOptions): Array<string> {
	const env = options.env ?? process.env
	const candidates: Array<string> = []

	if (env.AGENT_TOOLKIT_NATIVE) {
		candidates.push(env.AGENT_TOOLKIT_NATIVE)
	}

	const executableName = binaryName(options.platform)
	candidates.push(
		join(
			options.packageRoot,
			'bin',
			'native',
			nativePlatformKey(options),
			executableName,
		),
	)

	if (isSourceCheckout(options.packageRoot)) {
		candidates.push(join(options.packageRoot, 'target', 'release', executableName))
		candidates.push(join(options.packageRoot, 'target', 'debug', executableName))
	}

	return candidates
}

export function findNativeBinary(options: NativeRunOptions): string | null {
	return nativeBinaryCandidates(options).find((candidate) => existsSync(candidate)) ?? null
}

export function resolveNativeCommand(
	args: Array<string>,
	options: NativeRunOptions,
): NativeCommandResolution {
	const binary = findNativeBinary(options)
	if (binary) {
		return {
			command: [binary, ...args],
			error: null,
		}
	}

	if (isSourceCheckout(options.packageRoot)) {
		return {
			command: ['cargo', 'run', '-p', 'agent-toolkit', '--quiet', '--', ...args],
			error: null,
		}
	}

	return {
		command: null,
		error: [
			`No bundled agent-toolkit native binary found for ${nativePlatformKey(options)}.`,
			'Install a supported release or build from source in a git checkout.',
			'End users should not need Rust installed.',
		].join(' '),
	}
}

export async function runNative(args: Array<string>, options: NativeRunOptions): Promise<number> {
	const resolution = resolveNativeCommand(args, options)
	if (!resolution.command) {
		console.error(resolution.error)
		return 1
	}

	const result = Bun.spawnSync({
		cmd: resolution.command,
		cwd: options.packageRoot,
		stdin: 'inherit',
		stdout: 'inherit',
		stderr: 'inherit',
	})

	return result.exitCode
}

function isSourceCheckout(packageRoot: string): boolean {
	return existsSync(join(packageRoot, '.git')) && existsSync(join(packageRoot, 'Cargo.toml'))
}
