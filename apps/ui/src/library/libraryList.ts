import type { LibraryComponent, LibraryEntry } from '../app/engineTypes';

const collator = new Intl.Collator(undefined, { sensitivity: 'base', numeric: true });

export type FlatComponent = {
	stem: string;
	writable: boolean;
	key: string;
	name: string;
	origin: string;
};

export function sortComponents<T extends Pick<LibraryComponent, 'key' | 'name'>>(
	items: readonly T[]
): T[] {
	return [...items].sort(
		(a, b) => collator.compare(a.name, b.name) || collator.compare(a.key, b.key)
	);
}

export function flattenLibrary(lib: LibraryEntry): LibraryComponent[] {
	return sortComponents(lib.categories.flatMap((c) => c.components));
}

export function searchComponents(
	libs: LibraryEntry[],
	rawQuery: string,
	titleFor: (lib: LibraryEntry) => string
): FlatComponent[] {
	const query = rawQuery.trim().toLowerCase();
	if (!query) return [];
	const hits: FlatComponent[] = [];
	for (const lib of libs) {
		const title = titleFor(lib);
		for (const cat of lib.categories) {
			const origin =
				!cat.name || cat.name === lib.title || cat.name === title
					? title
					: `${title} / ${cat.name}`;
			for (const item of cat.components) {
				const hay = [item.key, item.name, title, cat.name, origin];
				if (hay.some((s) => s.toLowerCase().includes(query))) {
					hits.push({
						stem: lib.stem,
						writable: lib.writable,
						key: item.key,
						name: item.name,
						origin
					});
				}
			}
		}
	}
	hits.sort((a, b) => collator.compare(a.name, b.name) || collator.compare(a.key, b.key));
	return hits;
}
