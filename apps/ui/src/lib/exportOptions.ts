import { MM_PER_LU, PNG_PPI_PRESETS } from './constants';
import { copyRgba } from './color';
import type { Layer } from '../app/engineTypes';

export function luToCm(lu: number): number {
	return (lu * MM_PER_LU) / 10;
}

export function ptToCm(pt: number): number {
	return (pt / 72) * 2.54;
}

export function formatCm(wCm: number, hCm: number): string {
	return `${wCm.toFixed(2)} × ${hCm.toFixed(2)} cm`;
}

export type ExportFormat = 'svg' | 'png' | 'pdf' | 'emf' | 'print';

export type ExportLayerOpt = { show: boolean; color: [number, number, number, number] };

export type PdfPage = 'drawing' | 'a4' | 'letter';
export type PrintPage = 'drawing' | 'a4' | 'letter';
export type PrintScale = 'fit' | '1:1';
export type ExportRotate = 0 | 90 | 180 | 270;

export type ExportPreviewOpts = {
	format: ExportFormat;
	marginMm: number;
	bw: boolean;
	layers: ExportLayerOpt[];
	ppi: number;
	antiAlias: boolean;
	bgEnabled: boolean;
	bgColor: [number, number, number, number];
	flipH: boolean;
	flipV: boolean;
	rotate: ExportRotate;
	scale: number;
	pdfPage: PdfPage;
	pdfLandscape: boolean;
	pdfScale: number;
	printPage: PrintPage;
	printLandscape: boolean;
	printScale: PrintScale;
	printMarginMm: number;
};

export function defaultExportOpts(format: ExportFormat, layers: Layer[]): ExportPreviewOpts {
	return {
		format,
		marginMm: 2,
		bw: false,
		layers: layers.map((l) => ({ show: l.show, color: copyRgba(l.color) })),
		ppi: 1200,
		antiAlias: true,
		bgEnabled: format === 'png' || format === 'pdf' || format === 'print',
		bgColor: [255, 255, 255, 255],
		flipH: false,
		flipV: false,
		rotate: 0,
		scale: 1,
		pdfPage: 'drawing',
		pdfLandscape: false,
		pdfScale: 1,
		printPage: 'a4',
		printLandscape: false,
		printScale: 'fit',
		printMarginMm: 10
	};
}

export function wasmExportJson(opts: ExportPreviewOpts): string {
	return JSON.stringify({
		margin_lu: opts.marginMm / MM_PER_LU,
		bw: opts.bw,
		layers: opts.layers.map((l) => ({ show: l.show, color: copyRgba(l.color) }))
	});
}

export function exportBackground(opts: ExportPreviewOpts): [number, number, number, number] | null {
	return opts.bgEnabled ? copyRgba(opts.bgColor) : null;
}

export const ppiChoices = PNG_PPI_PRESETS;
