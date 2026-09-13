import { gcm } from '@noble/ciphers/aes.js';
import * as ed from '@noble/ed25519';
import { unzipSync } from 'fflate';

const MAGIC = new TextEncoder().encode('FIDOPRO1');
const ALG_AES_GCM = 1;
const NONCE_LEN = 12;
const TAG_LEN = 16;
const SIG_LEN = 64;

export const PAYLOAD_FILES = [
	'fidorust_wasm.js',
	'fidorust_wasm_bg.wasm',
	'overlay.js',
	'manifest.json'
] as const;

export type ProPayload = {
	version: string;
	files: Record<(typeof PAYLOAD_FILES)[number], Uint8Array>;
};

export async function decodeModule(
	blob: Uint8Array,
	publicKey: Uint8Array,
	aesKey: Uint8Array,
	expectedVersion: string
): Promise<ProPayload> {
	const min = MAGIC.length + 2 + NONCE_LEN + TAG_LEN + SIG_LEN;
	if (blob.length < min) throw new Error('module truncated');

	const unsigned = blob.subarray(0, blob.length - SIG_LEN);
	const signature = blob.subarray(blob.length - SIG_LEN);
	const ok = await ed.verifyAsync(signature, unsigned, publicKey);
	if (!ok) throw new Error('invalid module signature');

	for (let i = 0; i < MAGIC.length; i++) {
		if (unsigned[i] !== MAGIC[i]) throw new Error('bad magic');
	}
	const alg = unsigned[MAGIC.length];
	if (alg !== ALG_AES_GCM) throw new Error(`unsupported alg ${alg}`);
	const versionLen = unsigned[MAGIC.length + 1] ?? 0;
	const versionStart = MAGIC.length + 2;
	const nonceStart = versionStart + versionLen;
	if (nonceStart + NONCE_LEN > unsigned.length) throw new Error('module truncated');
	const version = new TextDecoder().decode(unsigned.subarray(versionStart, nonceStart));
	if (version !== expectedVersion) {
		throw new Error(`module version ${version} != app ${expectedVersion}`);
	}
	const nonce = unsigned.subarray(nonceStart, nonceStart + NONCE_LEN);
	const ciphertext = unsigned.subarray(nonceStart + NONCE_LEN);
	if (ciphertext.length < TAG_LEN) throw new Error('ciphertext truncated');
	const zipBytes = gcm(aesKey, nonce).decrypt(ciphertext);
	const unzipped = unzipSync(zipBytes);
	const files = {} as ProPayload['files'];
	for (const name of PAYLOAD_FILES) {
		const data = unzipped[name];
		if (!data) throw new Error(`zip missing ${name}`);
		files[name] = data;
	}
	return { version, files };
}
