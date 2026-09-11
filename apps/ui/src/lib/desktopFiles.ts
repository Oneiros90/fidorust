import type { AppSession } from '../app/appSession.svelte';
import {
	applyOpenedFiles,
	fileKindFromName,
	type OpenedBytes,
	type OpenedFileKind
} from '../app/fileOps';

type RawOpenedFile = {
	name: string;
	kind: string;
	data: number[] | Uint8Array;
};

function toOpenedFiles(raw: RawOpenedFile[]): OpenedBytes[] {
	const opened: OpenedBytes[] = [];
	for (const file of raw) {
		const kind: OpenedFileKind | null =
			file.kind === 'fcd' || file.kind === 'fcl' ? file.kind : fileKindFromName(file.name);
		if (!kind) continue;
		opened.push({
			name: file.name,
			kind,
			data: file.data instanceof Uint8Array ? file.data : new Uint8Array(file.data)
		});
	}
	return opened;
}

export async function startDesktopFileBridge(s: AppSession): Promise<boolean> {
	try {
		const { invoke, isTauri } = await import('@tauri-apps/api/core');
		if (!isTauri()) return false;
		const { listen } = await import('@tauri-apps/api/event');
		const { getCurrentWindow } = await import('@tauri-apps/api/window');

		const consumePending = async (replaceSession: boolean) => {
			const raw = await invoke<RawOpenedFile[]>('take_pending_opens');
			return applyOpenedFiles(s, toOpenedFiles(raw), { replaceSession });
		};

		await listen('open-files-ready', () => {
			void consumePending(false);
		});

		const fromLaunch = await consumePending(true);

		await getCurrentWindow().onDragDropEvent((event) => {
			if (event.payload.type !== 'drop') return;
			const { paths } = event.payload;
			void (async () => {
				const raw = await invoke<RawOpenedFile[]>('read_open_files', { paths });
				applyOpenedFiles(s, toOpenedFiles(raw));
			})();
		});

		return fromLaunch;
	} catch {
		return false;
	}
}

export async function syncDesktopTitle(title: string) {
	try {
		const { isTauri } = await import('@tauri-apps/api/core');
		if (!isTauri()) return;
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		await getCurrentWindow().setTitle(title);
	} catch {
		// web build, or the window API is unavailable
	}
}
