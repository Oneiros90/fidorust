export type RecentEntry = {
	name: string;
	fcd: string;
	at: number;
};

const KEY = 'fidorust.recent';
const MAX = 10;

export function loadRecents(): RecentEntry[] {
	try {
		const raw = localStorage.getItem(KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw) as unknown;
		if (!Array.isArray(parsed)) return [];
		return parsed.filter(isRecentEntry).slice(0, MAX);
	} catch {
		return [];
	}
}

export function pushRecent(list: RecentEntry[], name: string, fcd: string): RecentEntry[] {
	const next = [{ name, fcd, at: Date.now() }, ...list.filter((e) => e.name !== name)].slice(
		0,
		MAX
	);
	try {
		localStorage.setItem(KEY, JSON.stringify(next));
	} catch {
		/* quota / private mode */
	}
	return next;
}

function isRecentEntry(v: unknown): v is RecentEntry {
	if (!v || typeof v !== 'object') return false;
	const e = v as RecentEntry;
	return typeof e.name === 'string' && typeof e.fcd === 'string' && typeof e.at === 'number';
}
