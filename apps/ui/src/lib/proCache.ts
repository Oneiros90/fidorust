import { appVersion } from './updates';
import type { ProPayload } from './proCrypto';

const DB_NAME = 'fidorust-pro';
const STORE = 'module';
const KEY = 'payload';

type Cached = {
	appVersion: string;
	files: Record<string, ArrayBuffer>;
};

function openDb(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const req = indexedDB.open(DB_NAME, 1);
		req.onupgradeneeded = () => {
			const db = req.result;
			if (!db.objectStoreNames.contains(STORE)) db.createObjectStore(STORE);
		};
		req.onsuccess = () => resolve(req.result);
		req.onerror = () => reject(req.error ?? new Error('idb open failed'));
	});
}

export async function readCachedPayload(): Promise<ProPayload | null> {
	try {
		const db = await openDb();
		const cached = await new Promise<Cached | undefined>((resolve, reject) => {
			const tx = db.transaction(STORE, 'readonly');
			const req = tx.objectStore(STORE).get(KEY);
			req.onsuccess = () => resolve(req.result as Cached | undefined);
			req.onerror = () => reject(req.error ?? new Error('idb read failed'));
		});
		db.close();
		if (!cached || cached.appVersion !== appVersion) return null;
		const files = cached.files;
		const needed = ['fidorust_wasm.js', 'fidorust_wasm_bg.wasm', 'overlay.js', 'manifest.json'];
		if (!needed.every((name) => files[name])) return null;
		return {
			version: cached.appVersion,
			files: {
				'fidorust_wasm.js': new Uint8Array(files['fidorust_wasm.js']!),
				'fidorust_wasm_bg.wasm': new Uint8Array(files['fidorust_wasm_bg.wasm']!),
				'overlay.js': new Uint8Array(files['overlay.js']!),
				'manifest.json': new Uint8Array(files['manifest.json']!)
			}
		};
	} catch {
		return null;
	}
}

export async function writeCachedPayload(payload: ProPayload): Promise<void> {
	const files: Record<string, ArrayBuffer> = {};
	for (const [name, data] of Object.entries(payload.files)) {
		const buf = new ArrayBuffer(data.byteLength);
		new Uint8Array(buf).set(data);
		files[name] = buf;
	}
	const db = await openDb();
	await new Promise<void>((resolve, reject) => {
		const tx = db.transaction(STORE, 'readwrite');
		tx.oncomplete = () => resolve();
		tx.onerror = () => reject(tx.error ?? new Error('idb write failed'));
		tx.objectStore(STORE).put({ appVersion: payload.version, files } satisfies Cached, KEY);
	});
	db.close();
}

export async function clearCachedPayload(): Promise<void> {
	try {
		const db = await openDb();
		await new Promise<void>((resolve, reject) => {
			const tx = db.transaction(STORE, 'readwrite');
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error ?? new Error('idb clear failed'));
			tx.objectStore(STORE).delete(KEY);
		});
		db.close();
	} catch {
		/* private mode */
	}
}
