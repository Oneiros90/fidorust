<script lang="ts">
	import { parseSvgElement } from '../lib/attachSvg';
	import {
		exportBackground,
		formatCm,
		luToCm,
		ptToCm,
		type ExportPreviewOpts
	} from '../lib/exportOptions';
	import { pngPixelSize, rasterizeSvg } from '../lib/svgRaster';
	import { pdfLayout, svgToPdfBlob } from '../lib/svgPdf';
	import { A4_PT, LETTER_PT } from '../lib/constants';
	import { parseSvgViewBox } from '../lib/svgGeom';
	import {
		applyPreviewWheel,
		fitToView,
		syncPreviewView,
		type PreviewCamera
	} from '../lib/exportPreviewView';
	import { getAppSession } from '../app/appContext';

	let {
		svg,
		opts,
		title
	}: {
		svg: string;
		opts: ExportPreviewOpts;
		title: string;
	} = $props();

	const app = getAppSession();
	let pngUrl = $state('');
	let pdfUrl = $state('');
	let viewerEl = $state<HTMLDivElement | undefined>();
	let dragging = false;
	let lastX = 0;
	let lastY = 0;
	let cam = $state<PreviewCamera>({
		zoom: 1,
		panX: 0,
		panY: 0,
		fittedFormat: null,
		artW: 0,
		artH: 0
	});

	const sheet = $derived.by(() => {
		if (!svg) return null;
		try {
			const box = parseSvgViewBox(svg);
			const wCm = luToCm(box.w);
			const hCm = luToCm(box.h);
			return { wCm, hCm, label: formatCm(wCm, hCm) };
		} catch {
			return null;
		}
	});
	const pngInfo = $derived.by(() => {
		if (opts.format !== 'png' || !svg) return null;
		try {
			return pngPixelSize(svg, opts.ppi);
		} catch {
			return null;
		}
	});
	const pdfPageLabel = $derived.by(() => {
		if (opts.format !== 'pdf' || !svg) return null;
		try {
			const layout = pdfLayout(svg, {
				page: opts.pdfPage,
				landscape: opts.pdfLandscape,
				scale: opts.pdfScale
			});
			return formatCm(ptToCm(layout.pageW), ptToCm(layout.pageH));
		} catch {
			return null;
		}
	});
	const printPaperLabel = $derived.by(() => {
		if (opts.format !== 'print' || opts.printPage === 'drawing') return null;
		const paper = opts.printPage === 'letter' ? LETTER_PT : A4_PT;
		const w = opts.printLandscape ? paper.h : paper.w;
		const h = opts.printLandscape ? paper.w : paper.h;
		return formatCm(ptToCm(w), ptToCm(h));
	});
	const scaledSheetLabel = $derived(
		sheet && (opts.format === 'svg' || opts.format === 'emf') && opts.scale !== 1
			? formatCm(sheet.wCm * opts.scale, sheet.hCm * opts.scale)
			: null
	);

	$effect(() => {
		const format = opts.format;
		const current = svg;
		if (format !== 'png' || !current) {
			pngUrl = '';
			return;
		}
		let cancelled = false;
		void rasterizeSvg(current, {
			ppi: opts.ppi,
			antiAlias: opts.antiAlias
		})
			.then((canvas) => {
				if (!cancelled) pngUrl = canvas.toDataURL('image/png');
			})
			.catch(() => {
				if (!cancelled) pngUrl = '';
			});
		return () => {
			cancelled = true;
		};
	});

	const pdfBlob = $derived(
		opts.format === 'pdf' && svg
			? svgToPdfBlob(svg, {
					page: opts.pdfPage,
					landscape: opts.pdfLandscape,
					scale: opts.pdfScale,
					background: exportBackground(opts)
				})
			: null
	);

	$effect(() => {
		if (!pdfBlob) {
			pdfUrl = '';
			return;
		}
		const url = URL.createObjectURL(pdfBlob);
		pdfUrl = url;
		return () => URL.revokeObjectURL(url);
	});

	$effect(() => {
		const format = opts.format;
		if (format === 'png' && !pngUrl) return;
		if (format === 'pdf' && !pdfUrl) return;
		if (format !== 'png' && format !== 'pdf' && !svg) return;
		if (cam.fittedFormat === format) return;
		queueMicrotask(() => {
			requestAnimationFrame(() => fitToView(cam, viewerEl, opts.format));
		});
	});

	function bindViewer(node: HTMLDivElement) {
		viewerEl = node;
		return () => {
			if (viewerEl === node) viewerEl = undefined;
		};
	}

	function attachPreview(raw: string) {
		return (node: HTMLElement) => {
			const el = parseSvgElement(raw);
			if (el) {
				el.removeAttribute('width');
				el.removeAttribute('height');
				el.setAttribute('preserveAspectRatio', 'xMidYMid meet');
				node.replaceChildren(el);
				queueMicrotask(() => syncPreviewView(cam, viewerEl, opts.format));
			}
		};
	}

	function onFit() {
		fitToView(cam, viewerEl, opts.format);
	}

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		applyPreviewWheel(cam, viewerEl, e);
	}

	function onPointerDown(e: PointerEvent) {
		if (e.button !== 0) return;
		dragging = true;
		lastX = e.clientX;
		lastY = e.clientY;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging) return;
		cam.panX += e.clientX - lastX;
		cam.panY += e.clientY - lastY;
		lastX = e.clientX;
		lastY = e.clientY;
	}

	function onPointerUp() {
		dragging = false;
	}
</script>

<div class="preview-col">
	<div
		class={[
			'viewer',
			{
				paper: opts.format === 'pdf' || opts.format === 'print'
			}
		]}
		{@attach bindViewer}
		role="region"
		aria-label={title}
		onwheel={onWheel}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
	>
		<div class="stage" style:transform="translate({cam.panX}px, {cam.panY}px) scale({cam.zoom})">
			{#if opts.format === 'png'}
				{#if pngUrl}
					<img
						class={['art', { crisp: !opts.antiAlias }]}
						src={pngUrl}
						alt=""
						draggable="false"
						onload={() => syncPreviewView(cam, viewerEl, opts.format)}
					/>
				{/if}
			{:else if opts.format === 'pdf'}
				{#if pdfUrl}
					<iframe class="art pdf" title="PDF" src={pdfUrl}></iframe>
				{/if}
			{:else if svg}
				<div class="art svg-art" {@attach attachPreview(svg)}></div>
			{/if}
		</div>
	</div>
	<div class="preview-bar">
		<button type="button" onclick={onFit}>{app.t.exportFitView}</button>
		{#if sheet}
			<span>{app.t.exportSize}: {sheet.label}</span>
		{/if}
		{#if pngInfo}
			<span>{pngInfo.w} × {pngInfo.h} px</span>
		{/if}
		{#if scaledSheetLabel}
			<span>{app.t.exportScale}: {scaledSheetLabel}</span>
		{/if}
		{#if pdfPageLabel}
			<span>{app.t.exportPdfPage}: {pdfPageLabel}</span>
		{/if}
		{#if printPaperLabel}
			<span>{app.t.exportPrintPage}: {printPaperLabel}</span>
		{/if}
	</div>
</div>

<style>
	.preview-col {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		gap: 8px;
	}
	.viewer {
		flex: 1;
		min-height: 0;
		position: relative;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: 8px;
		cursor: grab;
		background-color: var(--export-checker-base);
		background-image:
			linear-gradient(45deg, var(--export-checker-tile) 25%, transparent 25%),
			linear-gradient(-45deg, var(--export-checker-tile) 25%, transparent 25%),
			linear-gradient(45deg, transparent 75%, var(--export-checker-tile) 75%),
			linear-gradient(-45deg, transparent 75%, var(--export-checker-tile) 75%);
		background-size: 16px 16px;
		background-position:
			0 0,
			0 8px,
			8px -8px,
			-8px 0;
	}
	.viewer.paper {
		background: var(--export-checker-paper);
	}
	.viewer:active {
		cursor: grabbing;
	}
	.stage {
		transform-origin: 0 0;
		width: max-content;
		height: max-content;
	}
	.art {
		display: block;
		max-width: none;
		user-select: none;
	}
	.art.crisp {
		image-rendering: pixelated;
	}
	.svg-art :global(svg) {
		display: block;
		width: 480px;
		height: auto;
		overflow: visible;
	}
	.pdf {
		width: 480px;
		height: 640px;
		border: 0;
		background: #fff;
	}
	.preview-bar {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 12px;
		color: var(--fg-muted);
		flex-wrap: wrap;
	}
</style>
