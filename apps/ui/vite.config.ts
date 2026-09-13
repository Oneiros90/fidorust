import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig, type Plugin, type ViteDevServer } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const uiRoot = path.dirname(fileURLToPath(import.meta.url));
const cratesDir = path.resolve(uiRoot, '../../crates');
const proRoot = path.resolve(uiRoot, '../../../fidorust-pro');
const proEnabled = process.env.FIDORUST_PRO === '1';
const wasmCrate = proEnabled
	? path.resolve(proRoot, 'crates/fidorust-pro-wasm')
	: path.resolve(cratesDir, 'fidorust-wasm');
const watchDirs = proEnabled ? [cratesDir, path.resolve(proRoot, 'crates')] : [cratesDir];

function wasmPackDev(): Plugin {
	const args = [
		'build',
		wasmCrate,
		'--target',
		'web',
		'--out-dir',
		path.resolve(uiRoot, 'src/wasm'),
		'--out-name',
		'fidorust_wasm',
		'--dev'
	];

	const run = () =>
		new Promise<void>((resolve, reject) => {
			const child = spawn('wasm-pack', args, { cwd: uiRoot, stdio: 'inherit', shell: true });
			child.on('exit', (code) => {
				if (code === 0) resolve();
				else reject(new Error(`wasm-pack exited ${code}`));
			});
		});

	let server: ViteDevServer | undefined;
	let busy = false;
	let queued = false;

	const rebuild = async () => {
		if (busy) {
			queued = true;
			return;
		}
		busy = true;
		try {
			await run();
			server?.ws.send({ type: 'full-reload' });
		} catch (err) {
			console.error(err);
		} finally {
			busy = false;
			if (queued) {
				queued = false;
				await rebuild();
			}
		}
	};

	const inWatchTree = (file: string) =>
		watchDirs.some((dir) => {
			const rel = path.relative(dir, file);
			return rel && !rel.startsWith('..') && !path.isAbsolute(rel);
		});

	return {
		name: 'wasm-pack-dev',
		apply: 'serve',
		async configureServer(devServer) {
			server = devServer;
			await run();
			for (const dir of watchDirs) {
				devServer.watcher.add(dir);
			}
			devServer.watcher.on('change', (file) => {
				if (!inWatchTree(file)) return;
				if (file.split(path.sep).includes('target')) return;
				if (file.endsWith('.rs') || file.endsWith('Cargo.toml')) void rebuild();
			});
		}
	};
}

function fidorustPro(): Plugin {
	const registerPath = path.resolve(proRoot, 'ui/src/register.ts').replaceAll('\\', '/');
	return {
		name: 'fidorust-pro',
		resolveId(id) {
			if (id === 'virtual:fidorust-pro') return '\0virtual:fidorust-pro';
		},
		load(id) {
			if (id !== '\0virtual:fidorust-pro') return;
			if (!proEnabled) {
				return 'export async function register() {\n\treturn null;\n}\n';
			}
			return `export { register } from ${JSON.stringify(registerPath)};\n`;
		}
	};
}

export default defineConfig({
	plugins: [svelte(), fidorustPro(), wasmPackDev()],
	resolve: proEnabled ? { alias: { '@fidorust-ui': path.resolve(uiRoot, 'src') } } : undefined,
	clearScreen: false,
	envPrefix: ['VITE_', 'TAURI_ENV_*'],
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		fs: {
			allow: [path.resolve(uiRoot, '../..'), ...(proEnabled ? [proRoot] : [])]
		},
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**', '**/target/**'],
			...(proEnabled ? { followSymlinks: true } : {})
		}
	},
	build: {
		target: 'esnext',
		outDir: 'dist'
	}
});
