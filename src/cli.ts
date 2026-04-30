import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { runNative } from './native.ts'

export async function runAgentToolkit(args: Array<string>): Promise<number> {
	const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
	return runNative(args, { packageRoot })
}
