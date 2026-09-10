/** Load OS monospace faces into the WASM tessellator. */

type LocalFont = {
	family: string;
	style: string;
	blob: () => Promise<Blob>;
};

function isRegular(style: string): boolean {
	return /^(regular|normal|book)$/i.test(style.trim());
}

function toBytes(data: unknown): Uint8Array | null {
	if (data instanceof Uint8Array) return data;
	if (Array.isArray(data)) return Uint8Array.from(data as number[]);
	if (typeof data === 'string') {
		try {
			const bin = atob(data);
			const out = new Uint8Array(bin.length);
			for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
			return out;
		} catch {
			return null;
		}
	}
	return null;
}

async function registerFromTauri(app: {
	register_font(name: string, data: Uint8Array): boolean;
}): Promise<boolean> {
	try {
		const { invoke, isTauri } = await import('@tauri-apps/api/core');
		if (!isTauri()) return false;
		const fonts = await invoke<Array<{ family: string; data: unknown }>>('system_mono_fonts');
		let n = 0;
		for (const font of fonts) {
			const buf = toBytes(font.data);
			if (!buf || !font.family) continue;
			if (app.register_font(font.family, buf)) n += 1;
		}
		return n > 0;
	} catch {
		return false;
	}
}

async function registerFromLocalFonts(app: {
	register_font(name: string, data: Uint8Array): boolean;
}): Promise<boolean> {
	const query = (window as Window & { queryLocalFonts?: () => Promise<LocalFont[]> })
		.queryLocalFonts;
	if (typeof query !== 'function') return false;
	try {
		const fonts = await query();
		const byFamily = new Map<string, LocalFont>();
		for (const font of fonts) {
			const key = font.family.toLowerCase();
			const prev = byFamily.get(key);
			if (!prev || (isRegular(font.style) && !isRegular(prev.style))) {
				byFamily.set(key, font);
			}
		}
		let n = 0;
		for (const font of byFamily.values()) {
			try {
				const buf = new Uint8Array(await (await font.blob()).arrayBuffer());
				if (app.register_font(font.family, buf)) n += 1;
			} catch {
				/* skip one face */
			}
		}
		return n > 0;
	} catch {
		return false;
	}
}

export async function registerSystemMonospace(app: {
	register_font(name: string, data: Uint8Array): boolean;
}): Promise<void> {
	if (await registerFromTauri(app)) return;
	await registerFromLocalFonts(app);
}
