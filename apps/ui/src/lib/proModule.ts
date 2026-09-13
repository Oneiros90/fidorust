import { appVersion } from './updates';
import { decodeModule, type ProPayload } from './proCrypto';
import { PRO_SIGNING_PUBLIC_KEY_HEX, hexToBytes, unwrapModuleKey } from './proKeys';

export const DEFAULT_PRO_MODULE_BASE = 'https://oneiros90.github.io/fidorust';

export function proModuleUrl(version = appVersion): string {
	const raw = import.meta.env.VITE_PRO_MODULE_BASE as string | undefined;
	const base = (raw && raw.length > 0 ? raw : DEFAULT_PRO_MODULE_BASE).replace(/\/$/, '');
	return `${base}/pro/${version}/fidorust-pro.bin`;
}

function bytesFromBase64(b64: string): Uint8Array {
	const bin = atob(b64);
	const out = new Uint8Array(bin.length);
	for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
	return out;
}

async function downloadBytes(url: string): Promise<Uint8Array> {
	try {
		const { invoke, isTauri } = await import('@tauri-apps/api/core');
		if (isTauri()) {
			const b64 = await invoke<string>('fetch_pro_module', { url });
			return bytesFromBase64(b64);
		}
	} catch {
		/* web, or tauri command missing */
	}
	const res = await fetch(url);
	if (!res.ok) throw new Error(`HTTP ${res.status}`);
	return new Uint8Array(await res.arrayBuffer());
}

export async function fetchAndDecryptModule(license: string): Promise<ProPayload> {
	const aes = unwrapModuleKey(license);
	if (!aes) throw new Error('no license');
	const blob = await downloadBytes(proModuleUrl());
	return decodeModule(blob, hexToBytes(PRO_SIGNING_PUBLIC_KEY_HEX), aes, appVersion);
}
