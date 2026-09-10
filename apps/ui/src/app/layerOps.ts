import type { Dict } from '../i18n';
import type { Layer } from './engineTypes';

const FALLBACK_ALIASES: readonly (readonly string[])[] = [
	['Schematic', 'Schema', 'Circuit', 'Circuito'],
	['PCB copper side', 'PCB lato rame', 'Bottom copper'],
	['PCB component side', 'PCB lato componenti', 'Top copper'],
	['Silkscreen', 'Serigrafie']
];

const FALLBACK_KEYS = [
	'layerSchematic',
	'layerPcbCopper',
	'layerPcbComponent',
	'layerSilkscreen'
] as const;

function isStandardGenericName(name: string, index: number): boolean {
	return name === '' || name === `Layer ${index + 1}`;
}

export function displayLayerName(name: string, index: number, t: Dict): string {
	const aliases = FALLBACK_ALIASES[index];
	if (aliases?.some((alias) => alias === name)) {
		return t[FALLBACK_KEYS[index]];
	}
	if (index < FALLBACK_ALIASES.length && !name) {
		return t[FALLBACK_KEYS[index]];
	}
	if (index >= FALLBACK_ALIASES.length && isStandardGenericName(name, index)) {
		return t.layerGeneric.replace('{n}', String(index + 1));
	}
	return name;
}

export function commitLayerName(typed: string, stored: string, index: number, t: Dict): string {
	const next = typed.trim();
	if (next === displayLayerName(stored, index, t)) {
		return stored;
	}
	return next;
}

export function otherLayers(layers: Layer[], index: number, t: Dict) {
	return layers
		.map((l, i) => ({ name: displayLayerName(l.name, i, t), i }))
		.filter((x) => x.i !== index);
}
