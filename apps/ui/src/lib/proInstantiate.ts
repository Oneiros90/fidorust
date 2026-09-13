import type { App as WasmApp } from '../wasm/fidorust_wasm.js';
import type { ProPack } from './proHost';

const liveUrls: string[] = [];

function blobUrl(bytes: Uint8Array, type: string): string {
	const copy = new Uint8Array(bytes.byteLength);
	copy.set(bytes);
	const url = URL.createObjectURL(new Blob([copy], { type }));
	liveUrls.push(url);
	return url;
}

export function revokeProBlobUrls() {
	for (const url of liveUrls.splice(0)) URL.revokeObjectURL(url);
}

type Glue = {
	default: (module?: unknown) => Promise<unknown>;
	App: new () => WasmApp;
};

export async function instantiateProApp(
	jsSource: Uint8Array,
	wasmBytes: Uint8Array
): Promise<WasmApp> {
	const js = new TextDecoder().decode(jsSource);
	const url = blobUrl(new TextEncoder().encode(js), 'text/javascript');
	const glue = (await import(/* @vite-ignore */ url)) as Glue;
	const copy = new Uint8Array(wasmBytes.byteLength);
	copy.set(wasmBytes);
	await glue.default({ module_or_path: copy });
	return new glue.App();
}

export async function instantiateOverlay(jsSource: Uint8Array): Promise<ProPack> {
	const js = new TextDecoder().decode(jsSource);
	const url = blobUrl(new TextEncoder().encode(js), 'text/javascript');
	const mod = (await import(/* @vite-ignore */ url)) as { register: () => Promise<ProPack> };
	return mod.register();
}
