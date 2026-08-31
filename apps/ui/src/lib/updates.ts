import { version as packageVersion } from '../../package.json';

const GITHUB_REPO = 'Oneiros90/fidorust';
const LATEST_RELEASE_API = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;
const RELEASE_PATH = `/${GITHUB_REPO}/releases/`;

export const appVersion = packageVersion;

export type LatestRelease = {
	version: string;
	tag: string;
	url: string;
};

export function displayVersion(version: string): string {
	return version.startsWith('v') ? version : `v${version}`;
}

export function parseSemver(version: string): [number, number, number] | null {
	const m = version.trim().replace(/^v/i, '').match(/^(\d+)\.(\d+)\.(\d+)/);
	if (!m) return null;
	return [Number(m[1]), Number(m[2]), Number(m[3])];
}

export function isNewerVersion(remote: string, current: string): boolean {
	const a = parseSemver(remote);
	const b = parseSemver(current);
	if (!a || !b) return false;
	for (let i = 0; i < 3; i++) {
		if (a[i]! > b[i]!) return true;
		if (a[i]! < b[i]!) return false;
	}
	return false;
}

export function isReleaseUrl(url: string): boolean {
	try {
		const parsed = new URL(url);
		return parsed.protocol === 'https:' && parsed.hostname === 'github.com' && parsed.pathname.startsWith(RELEASE_PATH);
	} catch {
		return false;
	}
}

export async function isDesktopApp(): Promise<boolean> {
	try {
		const { isTauri } = await import('@tauri-apps/api/core');
		return isTauri();
	} catch {
		return false;
	}
}

export async function fetchLatestRelease(): Promise<LatestRelease | null> {
	try {
		const res = await fetch(LATEST_RELEASE_API, {
			headers: { Accept: 'application/vnd.github+json' }
		});
		if (!res.ok) return null;
		const data: { tag_name?: unknown; html_url?: unknown } = await res.json();
		if (typeof data.tag_name !== 'string' || typeof data.html_url !== 'string') return null;
		if (!isReleaseUrl(data.html_url)) return null;
		return {
			version: data.tag_name.replace(/^v/i, ''),
			tag: displayVersion(data.tag_name),
			url: data.html_url
		};
	} catch {
		return null;
	}
}

export async function checkDesktopUpdate(current: string): Promise<LatestRelease | null> {
	const latest = await fetchLatestRelease();
	if (!latest || !isNewerVersion(latest.version, current)) return null;
	return latest;
}

export async function openReleasePage(url: string): Promise<void> {
	if (!isReleaseUrl(url)) return;
	try {
		const { openUrl } = await import('@tauri-apps/plugin-opener');
		await openUrl(url);
	} catch {
		window.open(url, '_blank', 'noopener,noreferrer');
	}
}
