import type { Dict } from '../i18n';
import type { PrimitiveDefaults } from './engineTypes';

export type ProjectSettingsSection =
	| 'general'
	| 'line'
	| 'rect'
	| 'ellipse'
	| 'poly'
	| 'bezier'
	| 'text'
	| 'pcb-track'
	| 'pcb-pad';

export const TOOL_DEFAULT_SECTIONS: readonly {
	id: ProjectSettingsSection;
	toolId: string;
	label: keyof Dict;
}[] = [
	{ id: 'line', toolId: 'line', label: 'line' },
	{ id: 'rect', toolId: 'rect', label: 'rect' },
	{ id: 'ellipse', toolId: 'ellipse', label: 'ellipse' },
	{ id: 'poly', toolId: 'poly', label: 'poly' },
	{ id: 'bezier', toolId: 'bezier', label: 'bezier' },
	{ id: 'text', toolId: 'text', label: 'text' },
	{ id: 'pcb-track', toolId: 'pcb-track', label: 'pcbTrack' },
	{ id: 'pcb-pad', toolId: 'pcb-pad', label: 'pcbPad' }
];

const TOOL_SECTION_IDS = new Set(TOOL_DEFAULT_SECTIONS.map((s) => s.toolId));

export function toolHasDefaults(toolId: string): boolean {
	return TOOL_SECTION_IDS.has(toolId);
}

export function sectionForTool(toolId: string): ProjectSettingsSection {
	return TOOL_DEFAULT_SECTIONS.find((s) => s.toolId === toolId)?.id ?? 'general';
}

export function defaultPrimitiveDefaults(): PrimitiveDefaults {
	return {
		line_stroke_hundredths: 25,
		bezier_stroke_hundredths: 25,
		rect: { stroke_hundredths: 25, filled: false },
		ellipse: { stroke_hundredths: 25, filled: false },
		poly: { stroke_hundredths: 25, filled: false },
		text: { sy: 4, sx: 3, style: 0, font: 'Courier Prime' },
		track_width: 4,
		pad: { dx: 18, dy: 18, hole: 8, style: 'Oval' }
	};
}
