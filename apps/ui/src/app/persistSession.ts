import { saveSession, SESSION_DEBOUNCE_MS, type SessionState } from '../lib/sessionStore';
import { persistUserLibrariesBlob } from '../lib/userLibraries';
import type { AppSession } from './appSession.svelte';

export function fcdHasProjectSettings(text: string): boolean {
	return /(^|[\r\n])[ \t]*PS(?:\s|$)/i.test(text);
}

export function restoreSession(s: AppSession, session: SessionState) {
	if (!s.engine) return;
	try {
		s.engine.mutate((app) => {
			app.load_fcd(session.fcd);
			app.set_view(session.zoom, session.panX, session.panY);
			app.set_tool(
				session.tool === 'component' || session.tool === 'macro' ? 'select' : session.tool
			);
			app.set_layer(session.layer);
			if (!fcdHasProjectSettings(session.fcd)) {
				app.set_snap_enable(session.snapEnable);
				app.set_show_grid(session.showGrid);
				app.set_hide_component_origin(session.hideComponentOrigin);
			}
		});
		s.fileHandleName = session.name;
		s.savedSnapshot = session.savedFcd;
	} catch (err) {
		s.error = String(err);
		s.markClean();
	}
}

export class SessionPersist {
	#timer: ReturnType<typeof setTimeout> | null = null;
	enabled = false;

	constructor(private readonly host: AppSession) {}

	schedule() {
		if (!this.enabled) return;
		if (this.#timer) clearTimeout(this.#timer);
		this.#timer = setTimeout(() => {
			this.#timer = null;
			this.persistNow();
		}, SESSION_DEBOUNCE_MS);
	}

	flush() {
		if (this.#timer) {
			clearTimeout(this.#timer);
			this.#timer = null;
		}
		this.persistNow();
	}

	onVisibilityChange = () => {
		if (document.visibilityState === 'hidden') this.flush();
	};

	persistNow() {
		const s = this.host;
		if (!this.enabled || !s.engine) return;
		const status = s.status;
		saveSession({
			version: 1,
			fcd: s.engine.query((app) => app.save_fcd()),
			name: s.fileHandleName,
			savedFcd: s.savedSnapshot,
			zoom: status.zoom,
			panX: status.pan_x,
			panY: status.pan_y,
			tool: status.tool,
			layer: status.layer,
			snapEnable: status.snap_enable,
			showGrid: status.show_grid,
			hideComponentOrigin: status.hide_component_origin,
			theme: s.theme,
			locale: s.locale
		});
		persistUserLibrariesBlob(s.engine.query((app) => app.user_libraries_blob()));
	}
}
