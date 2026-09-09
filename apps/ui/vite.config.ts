import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig, type Plugin, type ViteDevServer } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const uiRoot = path.dirname(fileURLToPath(import.meta.url));
const cratesDir = path.resolve(uiRoot, '../../crates');

function wasmPackDev(): Plugin {
	const args = [
		'build',
		path.resolve(cratesDir, 'fidocad-wasm'),
		'--target',
		'web',
		'--out-dir',
		path.resolve(uiRoot, 'src/wasm'),
		'--out-name',
		'fidocad_wasm',
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

	return {
		name: 'wasm-pack-dev',
		apply: 'serve',
		async configureServer(devServer) {
			server = devServer;
			await run();
			devServer.watcher.add(cratesDir);
			devServer.watcher.on('change', (file) => {
				const rel = path.relative(cratesDir, file);
				if (rel.startsWith('..') || path.isAbsolute(rel)) return;
				if (rel.split(path.sep).includes('target')) return;
				if (file.endsWith('.rs') || file.endsWith('Cargo.toml')) void rebuild();
			});
		}
	};
}

export default defineConfig({
	plugins: [svelte(), wasmPackDev()],
	clearScreen: false,
	envPrefix: ['VITE_', 'TAURI_ENV_*'],
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**', '**/target/**']
		}
	},
	build: {
		target: 'esnext',
		outDir: 'dist'
	}
});
