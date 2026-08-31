function bytesToBase64(bytes: Uint8Array): string {
	let binary = '';
	const chunk = 0x8000;
	for (let i = 0; i < bytes.length; i += chunk) {
		binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
	}
	return btoa(binary);
}

function base64ToBytes(b64: string): Uint8Array {
	const binary = atob(b64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

async function streamToBytes(stream: ReadableStream<Uint8Array>): Promise<Uint8Array> {
	return new Uint8Array(await new Response(stream).arrayBuffer());
}

export async function encodeProject(fcd: string): Promise<string> {
	const stream = new Blob([fcd]).stream().pipeThrough(new CompressionStream('deflate-raw'));
	return bytesToBase64(await streamToBytes(stream));
}

export async function decodeProject(data: string): Promise<string> {
	const raw = base64ToBytes(data);
	const bytes = new Uint8Array(raw.byteLength);
	bytes.set(raw);
	const stream = new Blob([bytes.buffer]).stream().pipeThrough(
		new DecompressionStream('deflate-raw')
	);
	return new TextDecoder().decode(await streamToBytes(stream));
}

export function shareUrl(encoded: string): string {
	const path = window.location.pathname || '/';
	return `${window.location.origin}${path}?project=${encodeURIComponent(encoded)}`;
}

export function looksLikeFcd(text: string): boolean {
	return text.includes('FIDOCAD') || text.includes('LI ') || text.includes('MC ');
}
