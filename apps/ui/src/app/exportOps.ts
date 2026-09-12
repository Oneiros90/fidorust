import { svgToPdfBlob } from '../lib/svgPdf';
import { svgToEmf } from '../lib/emf';
import { canvasToPngBlob, rasterizeSvg } from '../lib/svgRaster';
import { MM_PER_LU } from '../lib/constants';
import { parseSvgViewBox, withSvgPixelSize, withSvgSizeAttrs } from '../lib/svgGeom';
import { embedExportFonts } from '../lib/exportFonts';
import { rgbaCss } from '../lib/color';
import { applyExportLayout } from '../lib/exportLayout';
import { exportBackground, wasmExportJson, type ExportPreviewOpts } from '../lib/exportOptions';
import type { AppSession } from './appSession.svelte';
import { download } from './fileOps';

export function queryExportSvg(s: AppSession, opts: ExportPreviewOpts): string {
	if (!s.engine) return '';
	return s.engine.query((app) =>
		applyExportLayout(
			embedExportFonts(app.export_svg(wasmExportJson(opts)), (name) => {
				const bytes = app.font_file_bytes(name);
				return bytes?.length ? new Uint8Array(bytes) : null;
			}),
			opts
		)
	);
}

function baseName(s: AppSession): string {
	return s.fileHandleName.replace(/\.fcd$/i, '') || 'drawing';
}

export async function confirmExport(s: AppSession, opts: ExportPreviewOpts, svg: string) {
	const name = baseName(s);
	try {
		if (opts.format === 'svg') {
			const box = parseSvgViewBox(svg);
			const scaled = withSvgPixelSize(svg, box.w * opts.scale, box.h * opts.scale);
			download(name + '.svg', scaled, 'image/svg+xml');
		} else if (opts.format === 'png') {
			const canvas = await rasterizeSvg(svg, {
				ppi: opts.ppi,
				antiAlias: opts.antiAlias
			});
			const blob = await canvasToPngBlob(canvas);
			download(name + '.png', blob, blob.type || 'image/png');
		} else if (opts.format === 'pdf') {
			download(
				name + '.pdf',
				svgToPdfBlob(svg, {
					page: opts.pdfPage,
					landscape: opts.pdfLandscape,
					scale: opts.pdfScale,
					background: exportBackground(opts)
				}),
				'application/pdf'
			);
		} else if (opts.format === 'emf') {
			const bytes = svgToEmf(svg, opts.scale, exportBackground(opts));
			const copy = new Uint8Array(bytes.byteLength);
			copy.set(bytes);
			download(name + '.emf', new Blob([copy.buffer], { type: 'image/x-emf' }), 'image/x-emf');
		} else {
			printSvg(svg, opts);
		}
		s.dialogs.close();
	} catch (err) {
		s.error = String(err);
	}
}

export function printSvg(svg: string, opts: ExportPreviewOpts) {
	const box = parseSvgViewBox(svg);
	const wMm = box.w * MM_PER_LU;
	const hMm = box.h * MM_PER_LU;
	const page =
		opts.printPage === 'letter'
			? `letter ${opts.printLandscape ? 'landscape' : 'portrait'}`
			: opts.printPage === 'a4'
				? `A4 ${opts.printLandscape ? 'landscape' : 'portrait'}`
				: `${wMm}mm ${hMm}mm`;
	const sized = opts.printScale === '1:1' ? withSvgSizeAttrs(svg, `${wMm}mm`, `${hMm}mm`) : svg;
	const svgCss =
		opts.printScale === 'fit'
			? 'svg{width:100%;height:auto;max-height:100%;display:block;}'
			: 'svg{display:block;}';
	const bg = exportBackground(opts);
	const pageBg = bg ? rgbaCss(bg) : 'transparent';
	const w = window.open('');
	if (!w) return;
	w.document.open();
	w.document.write(`<!DOCTYPE html><html><head><title>Print</title><style>
@page{size:${page};margin:${opts.printMarginMm}mm;}
html,body{margin:0;background:${pageBg};}
${svgCss}
</style></head><body>${sized}</body></html>`);
	w.document.close();
	w.focus();
	w.print();
}
