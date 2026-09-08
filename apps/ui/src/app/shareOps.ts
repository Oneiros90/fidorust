import { encodeProject, shareUrl } from '../lib/shareCodec';
import type { AppSession } from './appSession.svelte';
import type { SaveLibraryPolicy } from './fileOps';

export async function openShareLink(s: AppSession) {
	if (!s.engine) return;
	if (s.engine.query((app) => app.uses_user_library_components())) {
		s.dialogs.open({ kind: 'saveLocalComponents', purpose: 'shareLink' });
		return;
	}
	await openShareLinkWithPolicy(s, 'keep');
}

export async function openShareLinkWithPolicy(s: AppSession, policy: SaveLibraryPolicy) {
	if (!s.engine) return;
	s.dialogs.open({ kind: 'shareLink', url: '' });
	try {
		const url = shareUrl(
			await encodeProject(s.engine.query((app) => app.save_fcd_with_policy(policy)))
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
	if (s.engine.query((app) => app.uses_user_library_components())) {
		s.dialogs.open({ kind: 'saveLocalComponents', purpose: 'shareFcd' });
		return;
	}
	openShareFcdWithPolicy(s, 'keep');
}

export function openShareFcdWithPolicy(s: AppSession, policy: SaveLibraryPolicy) {
	if (!s.engine) return;
	s.dialogs.open({
		kind: 'shareFcd',
		text: s.engine.query((app) => app.save_fcd_with_policy(policy))
	});
}

export function closeShare(s: AppSession) {
	const kind = s.dialogs.dialog?.kind;
	if (kind === 'shareLink' || kind === 'shareFcd') s.dialogs.close();
}
