import alimentatore from './sample.fcd?raw';

export const EXAMPLES = [
	{ label: 'Alimentatore per CB', file: 'sample.fcd', fcd: alimentatore }
] as const;

export type Example = (typeof EXAMPLES)[number];
