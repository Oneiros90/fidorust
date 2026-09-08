import { encodeProject, shareUrl } from '../lib/shareCodec';
import type { AppSession } from './appSession.svelte';

export async function openShareLink(s: AppSession) {
	if (!s.engine) return;
	s.dialogs.open({ kind: 'shareLink', url: '' });
	try {
		const url = shareUrl(
			await encodeProject(s.engine.query((app) => app.save_fcd()))
		);
		if (s.dialogs.dialog?.kind === 'shareLink') {
			s.dialogs.dialog = { kind: 'shareLink', url };
		}
	} catch (err) {
		s.dialogs.close();
		s.error = String(err);
	}
}

export function openShareFcd(s: AppSession) {
	if (!s.engine) return;
	s.dialogs.open({
		kind: 'shareFcd',
		text: s.engine.query((app) => app.save_fcd())
	});
}

export function closeShare(s: AppSession) {
	const kind = s.dialogs.dialog?.kind;
	if (kind === 'shareLink' || kind === 'shareFcd') s.dialogs.close();
}
