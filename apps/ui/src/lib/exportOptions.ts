import { MM_PER_LU, PNG_PPI_PRESETS } from './constants';
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

export type ExportLayerOpt = { show: boolean; invert: boolean };

export type PdfPage = 'drawing' | 'a4' | 'letter';
export type PrintPage = 'drawing' | 'a4' | 'letter';
export type PrintScale = 'fit' | '1:1';

export type ExportPreviewOpts = {
	format: ExportFormat;
	marginMm: number;
	bw: boolean;
	layers: ExportLayerOpt[];
	ppi: number;
	antiAlias: boolean;
	whiteBg: boolean;
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
		layers: layers.map((l) => ({ show: l.show, invert: false })),
		ppi: 300,
		antiAlias: true,
		whiteBg: format === 'png' || format === 'pdf' || format === 'print',
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
		layers: opts.layers
	});
}

export const ppiChoices = PNG_PPI_PRESETS;
