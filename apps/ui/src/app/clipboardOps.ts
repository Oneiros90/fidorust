import { looksLikeFcd } from '../lib/shareCodec';
import type { AppSession } from './appSession.svelte';
import { confirmDiscard, loadText } from './fileOps';

export async function copyFcd(s: AppSession) {
	if (!s.engine) return;
	await navigator.clipboard.writeText(s.engine.query((app) => app.clipboard_fcd()));
}

export async function cutFcd(s: AppSession) {
	if (!s.engine) return;
	await copyFcd(s);
	s.engine.mutate((app) => {
		app.key('Delete', false);
	});
}

export async function pasteFcd(s: AppSession) {
	if (!s.engine) return;
	const text = await navigator.clipboard.readText();
	if (looksLikeFcd(text)) {
		s.engine.mutate((app) => {
			app.paste_selection(text);
		});
	}
}

export async function pasteNewDoc(s: AppSession) {
	const text = await navigator.clipboard.readText();
	if (!looksLikeFcd(text)) {
		s.error = s.t.clipboardNotFcd;
		return;
	}
	confirmDiscard(s, () => loadText(s, text, 'clipboard.fcd'));
}
