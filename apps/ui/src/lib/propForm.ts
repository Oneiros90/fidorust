/** Property form field from WASM `selection_props_form_json()`. */

import type { Dict } from '../i18n';

export type PropFieldId =
	| 'useComponentLayers'
	| 'filled'
	| 'layer'
	| 'thickness'
	| 'sizeX'
	| 'sizeY'
	| 'intDiam'
	| 'padStyle'
	| 'text'
	| 'fontFace'
	| 'fontHeight'
	| 'fontWidth'
	| 'rotationAngle'
	| 'bold'
	| 'italic'
	| 'mirrored'
	| 'underlined';

export type PropFieldKind =
	| { kind: 'bool' }
	| { kind: 'int'; min: number; max: number }
	| { kind: 'string' }
	| { kind: 'layer' }
	| { kind: 'padStyle' }
	| { kind: 'choice'; options: string[] };

export type PropFieldValue =
	| { state: 'unset' }
	| { state: 'bool'; value: boolean }
	| { state: 'int'; value: number }
	| { state: 'string'; value: string }
	| { state: 'layer'; value: number }
	| { state: 'padStyle'; value: string };

export type PropFormField = {
	id: PropFieldId;
	kind: PropFieldKind;
	value: PropFieldValue;
	readOnly?: boolean;
};

export type PropPatch = Partial<{
	useComponentLayers: boolean;
	filled: boolean;
	layer: number;
	thickness: number;
	sizeX: number;
	sizeY: number;
	intDiam: number;
	padStyle: string;
	text: string;
	fontFace: string;
	fontHeight: number;
	fontWidth: number;
	rotationAngle: number;
	bold: boolean;
	italic: boolean;
	mirrored: boolean;
	underlined: boolean;
}>;

export const fieldLabels: Record<PropFieldId, keyof Dict> = {
	useComponentLayers: 'propUseComponentLayers',
	filled: 'propFilled',
	layer: 'layer',
	thickness: 'propThickness',
	sizeX: 'propSizeX',
	sizeY: 'propSizeY',
	intDiam: 'propIntDiam',
	padStyle: 'propPadStyle',
	text: 'propText',
	fontFace: 'propFontFace',
	fontHeight: 'propFontHeight',
	fontWidth: 'propFontWidth',
	rotationAngle: 'propRotationAngle',
	bold: 'propBold',
	italic: 'propItalic',
	mirrored: 'propMirrored',
	underlined: 'propUnderlined'
};

const BOOL_IDS = ['useComponentLayers', 'filled', 'bold', 'italic', 'mirrored', 'underlined'] as const;
const INT_IDS = [
	'thickness',
	'sizeX',
	'sizeY',
	'intDiam',
	'fontHeight',
	'fontWidth',
	'rotationAngle'
] as const;
const STRING_IDS = ['text', 'fontFace'] as const;

function isBoolId(id: PropFieldId): id is (typeof BOOL_IDS)[number] {
	return (BOOL_IDS as readonly string[]).includes(id);
}
function isIntId(id: PropFieldId): id is (typeof INT_IDS)[number] {
	return (INT_IDS as readonly string[]).includes(id);
}
function isStringId(id: PropFieldId): id is (typeof STRING_IDS)[number] {
	return (STRING_IDS as readonly string[]).includes(id);
}

export function parsePropForm(raw: string): PropFormField[] {
	if (!raw || raw === '[]') return [];
	try {
		const arr = JSON.parse(raw) as PropFormField[];
		return Array.isArray(arr) ? arr : [];
	} catch {
		return [];
	}
}

export function editStateToPatch(state: Partial<Record<PropFieldId, PropFieldValue>>): PropPatch {
	const patch: PropPatch = {};
	for (const id of Object.keys(state) as PropFieldId[]) {
		const v = state[id];
		if (!v || v.state === 'unset') continue;
		if (v.state === 'bool' && isBoolId(id)) patch[id] = v.value;
		else if (v.state === 'int' && isIntId(id)) patch[id] = v.value;
		else if (v.state === 'string' && isStringId(id)) patch[id] = v.value;
		else if (v.state === 'layer') {
			const flag = state.useComponentLayers;
			if (flag?.state === 'bool' && flag.value) continue;
			patch.layer = v.value;
		}
		else if (v.state === 'padStyle') patch.padStyle = v.value;
	}
	return patch;
}
