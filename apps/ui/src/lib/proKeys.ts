/**
 * Module verification keys compiled into the public app.
 *
 * The Ed25519 secret never lives here. AES-256 is still the mock module key.
 * Activation requires SHA-256(passphrase) === VITE_PRO_LICENSE_HASH (baked at
 * build). Empty hash fails closed. Replace `unwrapModuleKey` when real licenses
 * wrap per-release keys.
 */

import { sha256 } from '@noble/hashes/sha2.js';

export const PRO_SIGNING_PUBLIC_KEY_HEX =
	'67ca627b7d0dba81ed1c89cf0fee0182ebbaf808891aa06c4075b73f365df878';

export const PRO_MOCK_MODULE_KEY_HEX =
	'a48d628971c6139e50d850a8cbb518cb31c2b61d0c0fbc1fdf47eb69b67f7c3d';

function expectedLicenseHash(): string {
	const raw =
		(import.meta.env.VITE_PRO_LICENSE_HASH as string | undefined)?.trim().toLowerCase() ?? '';
	return /^[0-9a-f]{64}$/.test(raw) ? raw : '';
}

export function normalizeLicenseKey(stored: string | null): string | null {
	if (!stored) return null;
	const s = stored.startsWith('key:') ? stored.slice(4) : stored;
	const t = s.trim();
	return t.length > 0 ? t : null;
}

function hashesEqual(a: string, b: string): boolean {
	if (a.length !== b.length) return false;
	let d = 0;
	for (let i = 0; i < a.length; i++) d |= a.charCodeAt(i) ^ b.charCodeAt(i);
	return d === 0;
}

export function licenseKeyMatches(key: string | null): boolean {
	const phrase = key?.trim() ?? '';
	const expected = expectedLicenseHash();
	if (!phrase || !expected) return false;
	const got = bytesToHex(sha256(new TextEncoder().encode(phrase)));
	return hashesEqual(got, expected);
}

export function unwrapModuleKey(license: string | null): Uint8Array | null {
	if (!licenseKeyMatches(normalizeLicenseKey(license))) return null;
	return hexToBytes(PRO_MOCK_MODULE_KEY_HEX);
}

export function hexToBytes(hex: string): Uint8Array {
	const clean = hex.trim().replace(/^0x/i, '').replace(/\s+/g, '');
	if (clean.length % 2 !== 0 || !/^[0-9a-f]*$/i.test(clean)) {
		throw new Error('invalid hex');
	}
	const out = new Uint8Array(clean.length / 2);
	for (let i = 0; i < out.length; i++) {
		out[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
	}
	return out;
}

export function bytesToHex(bytes: Uint8Array): string {
	return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}
