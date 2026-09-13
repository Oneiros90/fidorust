/**
 * Module verification keys compiled into the public app.
 *
 * The Ed25519 secret never lives here. The AES-256 mock key is temporary: any
 * non-empty license unwraps it. Replace `unwrapModuleKey` when real licenses
 * wrap per-release keys.
 */

export const PRO_SIGNING_PUBLIC_KEY_HEX =
	'67ca627b7d0dba81ed1c89cf0fee0182ebbaf808891aa06c4075b73f365df878';

export const PRO_MOCK_MODULE_KEY_HEX =
	'a48d628971c6139e50d850a8cbb518cb31c2b61d0c0fbc1fdf47eb69b67f7c3d';

export function unwrapModuleKey(_license: string | null): Uint8Array | null {
	if (!_license) return null;
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
