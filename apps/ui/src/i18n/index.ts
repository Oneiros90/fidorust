import { en } from './en';
import type { Dict } from './en';
import { it } from './it';

export { en } from './en';
export type { Dict } from './en';
export { it } from './it';

export type Locale = 'it' | 'en';

export function dict(locale: Locale): typeof en {
	return locale === 'en' ? en : it;
}

/** `A`, `A e B`, `A, B e C`. English uses a serial comma. */
export function joinList(names: string[], andWord: string, serialComma: boolean): string {
	if (names.length === 0) return '';
	if (names.length === 1) return names[0];
	if (names.length === 2) return `${names[0]} ${andWord} ${names[1]}`;
	const head = names.slice(0, -1).join(', ');
	const last = names[names.length - 1];
	return serialComma ? `${head}, ${andWord} ${last}` : `${head} ${andWord} ${last}`;
}

export function commonNodeActionLabel(
	t: Dict,
	names: string[],
	count: number,
	separate: boolean
): string {
	if (names.length === 0) return separate ? t.separateNode : t.commonNode;
	if (names.length > 3) {
		const many = separate ? t.separateNodeMany : t.commonNodeMany;
		return many.replace('{n}', String(count));
	}
	const named = separate ? t.separateNodeNamed : t.commonNodeNamed;
	return named.replace('{names}', joinList(names, t.listAnd, t.listAnd === 'and'));
}
