import type { Locale } from '../i18n';
import type { Theme } from '../app/types';

export const SESSION_KEY = 'fidorust.session';
export const SESSION_DEBOUNCE_MS = 400;

export type PaneView = {
	sheetIndex: number;
	zoom: number;
	panX: number;
	panY: number;
};

export type SessionState = {
	version: 1;
	fcd: string;
	name: string;
	savedFcd: string;
	zoom: number;
	panX: number;
	panY: number;
	tool: string;
	layer: number;
	snapEnable: boolean;
	showGrid: boolean;
	hideComponentOrigin: boolean;
	theme: Theme;
	locale: Locale;
	split: boolean;
	splitRatio: number;
	panes: [PaneView, PaneView];
};

export function loadSession(): SessionState | null {
	try {
		const raw = localStorage.getItem(SESSION_KEY);
		if (!raw) return null;
		const parsed = JSON.parse(raw) as unknown;
		return isSessionState(parsed) ? parsed : null;
	} catch {
		return null;
	}
}

export function saveSession(state: SessionState) {
	try {
		localStorage.setItem(SESSION_KEY, JSON.stringify(state));
	} catch {
		/* quota / private mode */
	}
}

function paneView(v: unknown, fallback: PaneView): PaneView {
	if (!v || typeof v !== 'object') return fallback;
	const p = v as Record<string, unknown>;
	if (
		typeof p.sheetIndex !== 'number' ||
		typeof p.zoom !== 'number' ||
		typeof p.panX !== 'number' ||
		typeof p.panY !== 'number'
	) {
		return fallback;
	}
	return {
		sheetIndex: p.sheetIndex,
		zoom: p.zoom,
		panX: p.panX,
		panY: p.panY
	};
}

function isSessionState(v: unknown): v is SessionState {
	if (!v || typeof v !== 'object') return false;
	const s = v as Record<string, unknown>;
	const hide =
		typeof s.hideComponentOrigin === 'boolean'
			? s.hideComponentOrigin
			: typeof s.hideMacroOrigin === 'boolean'
				? s.hideMacroOrigin
				: null;
	if (hide === null) return false;
	if (
		s.version !== 1 ||
		typeof s.fcd !== 'string' ||
		typeof s.name !== 'string' ||
		typeof s.savedFcd !== 'string' ||
		typeof s.zoom !== 'number' ||
		typeof s.panX !== 'number' ||
		typeof s.panY !== 'number' ||
		typeof s.tool !== 'string' ||
		typeof s.layer !== 'number' ||
		typeof s.snapEnable !== 'boolean' ||
		typeof s.showGrid !== 'boolean' ||
		(s.theme !== 'light' && s.theme !== 'white' && s.theme !== 'dark') ||
		(s.locale !== 'it' && s.locale !== 'en')
	) {
		return false;
	}
	(s as SessionState).hideComponentOrigin = hide;
	const fallback: PaneView = {
		sheetIndex: 0,
		zoom: s.zoom as number,
		panX: s.panX as number,
		panY: s.panY as number
	};
	(s as SessionState).split = s.split === true;
	(s as SessionState).splitRatio =
		typeof s.splitRatio === 'number' && Number.isFinite(s.splitRatio)
			? Math.min(0.8, Math.max(0.2, s.splitRatio))
			: 0.5;
	const panes = Array.isArray(s.panes) ? s.panes : [];
	(s as SessionState).panes = [paneView(panes[0], fallback), paneView(panes[1], fallback)];
	return true;
}
