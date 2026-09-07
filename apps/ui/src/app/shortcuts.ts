import type { AppSession } from './appSession.svelte';

export type Shortcut = {
	key: string;
	meta?: boolean;
	alt?: boolean;
	when?: (app: AppSession) => boolean;
	run: (app: AppSession, e: KeyboardEvent) => void;
};

export const APP_SHORTCUTS: Shortcut[] = [
	{
		key: 'o',
		meta: true,
		run: (app, e) => {
			e.preventDefault();
			app.openFile();
		}
	},
	{
		key: 's',
		meta: true,
		run: (app, e) => {
			e.preventDefault();
			app.saveFile();
		}
	},
	{
		key: 'x',
		meta: true,
		run: (app, e) => {
			e.preventDefault();
			void app.cutFcd();
		}
	},
	{
		key: 'c',
		meta: true,
		run: (app, e) => {
			e.preventDefault();
			void app.copyFcd();
		}
	},
	{
		key: 'v',
		meta: true,
		run: (app, e) => {
			e.preventDefault();
			void app.pasteFcd();
		}
	},
	{
		key: 'Enter',
		alt: true,
		when: (app) => app.status.selected > 0,
		run: (app, e) => {
			e.preventDefault();
			app.openProperties();
		}
	}
];

export function shortcutMatches(e: KeyboardEvent, s: Shortcut, app: AppSession): boolean {
	const key = s.key === 'Enter' ? e.key === 'Enter' : e.key.toLowerCase() === s.key.toLowerCase();
	if (!key) return false;
	if (s.meta && !(e.metaKey || e.ctrlKey)) return false;
	if (s.alt && !e.altKey) return false;
	if (s.when && !s.when(app)) return false;
	return true;
}

export function runShortcuts(app: AppSession, e: KeyboardEvent, table: Shortcut[]) {
	for (const s of table) {
		if (shortcutMatches(e, s, app)) s.run(app, e);
	}
}
