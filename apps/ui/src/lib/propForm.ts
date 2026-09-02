/** Property form field from WASM `selection_props_form_json()`. */

export type PropFieldId =
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
	| { kind: 'padStyle' };

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

export function parsePropForm(raw: string): PropFormField[] {
	if (!raw || raw === '[]') return [];
	try {
		const arr = JSON.parse(raw) as PropFormField[];
		return Array.isArray(arr) ? arr : [];
	} catch {
		return [];
	}
}

/** Local edit state: null = indeterminate / unchanged on apply. */
export type PropEditState = Record<
	PropFieldId,
	| { mode: 'unset' }
	| { mode: 'bool'; value: boolean }
	| { mode: 'int'; value: number }
	| { mode: 'string'; value: string }
	| { mode: 'layer'; value: number }
	| { mode: 'padStyle'; value: string }
>;

export function initEditState(fields: PropFormField[]): PropEditState {
	const s = {} as PropEditState;
	for (const f of fields) {
		if (f.value.state === 'unset') {
			s[f.id] = { mode: 'unset' };
		} else if (f.value.state === 'bool') {
			s[f.id] = { mode: 'bool', value: f.value.value };
		} else if (f.value.state === 'int') {
			s[f.id] = { mode: 'int', value: f.value.value };
		} else if (f.value.state === 'string') {
			s[f.id] = { mode: 'string', value: f.value.value };
		} else if (f.value.state === 'layer') {
			s[f.id] = { mode: 'layer', value: f.value.value };
		} else if (f.value.state === 'padStyle') {
			s[f.id] = { mode: 'padStyle', value: f.value.value };
		}
	}
	return s;
}

export function editStateToPatch(state: PropEditState): PropPatch {
	const patch: PropPatch = {};
	for (const [id, v] of Object.entries(state) as [PropFieldId, PropEditState[PropFieldId]][]) {
		if (v.mode === 'unset') continue;
		if (v.mode === 'bool') (patch as Record<string, unknown>)[id] = v.value;
		else if (v.mode === 'int') (patch as Record<string, unknown>)[id] = v.value;
		else if (v.mode === 'string') (patch as Record<string, unknown>)[id] = v.value;
		else if (v.mode === 'layer') patch.layer = v.value;
		else if (v.mode === 'padStyle') patch.padStyle = v.value;
	}
	return patch;
}
