<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { queryExportSvg } from '../app/fileOps';
	import { rgbToHex } from '../lib/color';
	import { parseSvgElement } from '../lib/attachSvg';
	import {
		defaultExportOpts,
		formatCm,
		luToCm,
		ppiChoices,
		ptToCm,
		type ExportFormat,
		type ExportPreviewOpts
	} from '../lib/exportOptions';
	import { pngPixelSize, rasterizeSvg } from '../lib/svgRaster';
	import { pdfLayout, svgToPdfBlob } from '../lib/svgPdf';
	import { A4_PT, LETTER_PT, LU_PER_INCH } from '../lib/constants';
	import { parseSvgViewBox } from '../lib/svgGeom';
	import Modal from './Modal.svelte';
	import { untrack } from 'svelte';

	let { format }: { format: ExportFormat } = $props();

	const app = getAppSession();
	const fileFormats: Exclude<ExportFormat, 'print'>[] = ['svg', 'png', 'pdf', 'emf'];
	const a4Cm = formatCm(ptToCm(A4_PT.w), ptToCm(A4_PT.h));
	const letterCm = formatCm(ptToCm(LETTER_PT.w), ptToCm(LETTER_PT.h));

	let opts = $state<ExportPreviewOpts>(untrack(() => defaultExportOpts(format, app.layers.layers)));
	let zoom = $state(1);
	let panX = $state(0);
	let panY = $state(0);
	let pngUrl = $state('');
	let pdfUrl = $state('');
	let viewerEl = $state<HTMLDivElement | undefined>();
	let dragging = false;
	let lastX = 0;
	let lastY = 0;

	const svg = $derived.by(() => {
		void opts.marginMm;
		void opts.bw;
		for (const layer of opts.layers) {
			void layer.show;
			void layer.invert;
		}
		return queryExportSvg(app, opts);
	});
	const title = $derived(opts.format === 'print' ? app.t.printPreview : app.t.exportPreview);
	const confirmLabel = $derived(opts.format === 'print' ? app.t.print : app.t.exportAction);
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
			const box = parseSvgViewBox(svg);
			const w = Math.max(1, Math.round((box.w * opts.ppi) / LU_PER_INCH));
			const h = Math.max(1, Math.round((box.h * opts.ppi) / LU_PER_INCH));
			const clipped = pngPixelSize(svg, opts.ppi).clipped;
			return { w, h, clipped };
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
			whiteBg: opts.whiteBg,
			antiAlias: opts.antiAlias,
			maxEdge: 2048
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
					scale: opts.pdfScale
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
		void svg;
		void opts.format;
		void pngUrl;
		void pdfUrl;
		queueMicrotask(() => {
			requestAnimationFrame(() => fitToView());
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
			}
		};
	}

	function fitToView() {
		const view = viewerEl;
		if (!view) return;
		const art = view.querySelector('.art') as HTMLElement | null;
		if (!art) return;
		const pad = 24;
		const vw = Math.max(1, view.clientWidth - pad);
		const vh = Math.max(1, view.clientHeight - pad);
		const aw = Math.max(1, art.offsetWidth || art.scrollWidth);
		const ah = Math.max(1, art.offsetHeight || art.scrollHeight);
		const z = Math.min(vw / aw, vh / ah, 8);
		zoom = Number.isFinite(z) && z > 0 ? z : 1;
		panX = (view.clientWidth - aw * zoom) / 2;
		panY = (view.clientHeight - ah * zoom) / 2;
	}

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		const view = viewerEl;
		if (!view) return;
		const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
		const rect = view.getBoundingClientRect();
		const cx = e.clientX - rect.left;
		const cy = e.clientY - rect.top;
		const next = Math.min(32, Math.max(0.05, zoom * factor));
		const k = next / zoom;
		panX = cx - (cx - panX) * k;
		panY = cy - (cy - panY) * k;
		zoom = next;
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
		panX += e.clientX - lastX;
		panY += e.clientY - lastY;
		lastX = e.clientX;
		lastY = e.clientY;
	}

	function onPointerUp() {
		dragging = false;
	}

	function layerSwatch(i: number): string {
		const src = app.layers.layers[i]?.color ?? [0, 0, 0];
		let c = [...src];
		if (opts.layers[i]?.invert && !opts.bw) c = c.map((v) => 255 - v);
		if (opts.bw) c = [0, 0, 0];
		return rgbToHex(c);
	}

	function confirm() {
		void app.confirmExport(opts, svg);
	}
</script>

<Modal
	{title}
	titleId="export-dlg-title"
	closable
	closeLabel={app.t.close}
	maxWidth="1120px"
	maxHeight="90vh"
	overflow="hidden"
	onClose={() => app.dialogs.close()}
>
	<div class="export-dlg">
		<div class="body">
			<div class="preview-col">
				<div
					class={[
						'viewer',
						{
							paper:
								opts.format === 'pdf' ||
								opts.format === 'print' ||
								(opts.format === 'png' && opts.whiteBg)
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
					<div class="stage" style:transform="translate({panX}px, {panY}px) scale({zoom})">
						{#if opts.format === 'png'}
							{#if pngUrl}
								<img class="art" src={pngUrl} alt="" draggable="false" onload={fitToView} />
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
					<button type="button" onclick={fitToView}>{app.t.exportFitView}</button>
					{#if sheet}
						<span>{app.t.exportSize}: {sheet.label}</span>
					{/if}
					{#if pngInfo}
						<span>{pngInfo.w} × {pngInfo.h} px</span>
						{#if pngInfo.clipped}
							<span class="hint">{app.t.exportClipped}</span>
						{/if}
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
			<div class="side">
				{#if opts.format !== 'print'}
					<label>
						{app.t.exportFormat}
						<select bind:value={opts.format}>
							{#each fileFormats as f (f)}
								<option value={f}>{f.toUpperCase()}</option>
							{/each}
						</select>
					</label>
				{/if}
				<label>
					{app.t.exportMargin}
					<input type="number" min="0" max="50" step="0.5" bind:value={opts.marginMm} />
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={opts.bw} />
					{app.t.exportBw}
				</label>
				{#if opts.format === 'png'}
					<label>
						{app.t.exportPpi}
						<select bind:value={opts.ppi}>
							{#each ppiChoices as p (p)}
								<option value={p}>{p}</option>
							{/each}
						</select>
					</label>
					<label class="chk">
						<input type="checkbox" bind:checked={opts.antiAlias} />
						{app.t.exportAntiAlias}
					</label>
					<label class="chk">
						<input type="checkbox" bind:checked={opts.whiteBg} />
						{app.t.exportWhiteBg}
					</label>
				{/if}
				{#if opts.format === 'svg' || opts.format === 'emf'}
					<label>
						{app.t.exportScale}
						<input type="number" min="0.1" max="20" step="0.1" bind:value={opts.scale} />
						{#if scaledSheetLabel}
							<span class="size-hint">{scaledSheetLabel}</span>
						{:else if sheet}
							<span class="size-hint">{sheet.label}</span>
						{/if}
					</label>
				{/if}
				{#if opts.format === 'pdf'}
					<label>
						{app.t.exportPdfPage}
						<select bind:value={opts.pdfPage}>
							<option value="drawing">{app.t.exportPageDrawing}{sheet ? ` (${sheet.label})` : ''}</option>
							<option value="a4">{app.t.exportPageA4} ({a4Cm})</option>
							<option value="letter">{app.t.exportPageLetter} ({letterCm})</option>
						</select>
					</label>
					<label>
						{app.t.exportPdfScale}
						<input type="number" min="0.1" max="10" step="0.1" bind:value={opts.pdfScale} />
						{#if pdfPageLabel}
							<span class="size-hint">{pdfPageLabel}</span>
						{/if}
					</label>
					<label class="chk">
						<input type="checkbox" bind:checked={opts.pdfLandscape} />
						{app.t.exportLandscape}
					</label>
				{/if}
				{#if opts.format === 'print'}
					<label>
						{app.t.exportPrintPage}
						<select bind:value={opts.printPage}>
							<option value="drawing">{app.t.exportPageDrawing}{sheet ? ` (${sheet.label})` : ''}</option>
							<option value="a4">{app.t.exportPageA4} ({a4Cm})</option>
							<option value="letter">{app.t.exportPageLetter} ({letterCm})</option>
						</select>
					</label>
					<label>
						{app.t.exportPrintScale}
						<select bind:value={opts.printScale}>
							<option value="fit">{app.t.exportPrintFit}</option>
							<option value="1:1">{app.t.exportPrintActual}{sheet ? ` (${sheet.label})` : ''}</option>
						</select>
					</label>
					<label>
						{app.t.exportPrintMargin}
						<input type="number" min="0" max="40" step="1" bind:value={opts.printMarginMm} />
					</label>
					<label class="chk">
						<input type="checkbox" bind:checked={opts.printLandscape} />
						{app.t.exportLandscape}
					</label>
				{/if}
				<div class="layers">
					<div class="layers-head">{app.t.exportLayers}</div>
					<ul>
						{#each app.layers.layers as layer, i (i)}
							{#if opts.layers[i]}
								<li>
									<span class="swatch" style:background={layerSwatch(i)}></span>
									<span class="lname">{layer.name}</span>
									<button
										type="button"
										class={['icon', { off: !opts.layers[i].show }]}
										title={opts.layers[i].show ? app.t.hideLayer : app.t.showLayer}
										aria-label={opts.layers[i].show ? app.t.hideLayer : app.t.showLayer}
										aria-pressed={opts.layers[i].show}
										onclick={() => (opts.layers[i].show = !opts.layers[i].show)}
									>
										{#if opts.layers[i].show}
											<svg viewBox="0 0 24 24" aria-hidden="true">
												<path
													d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7z"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
												/>
												<circle
													cx="12"
													cy="12"
													r="3"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
												/>
											</svg>
										{:else}
											<svg viewBox="0 0 24 24" aria-hidden="true">
												<path
													d="M3 3l18 18M10.6 10.6A3 3 0 0 0 12 15a3 3 0 0 0 2.4-1.2M9.9 5.1A11 11 0 0 1 12 5c6 0 10 7 10 7a18 18 0 0 1-3.2 3.8M6.1 6.1C3.7 8 2 12 2 12s4 7 10 7a10 10 0 0 0 4-.8"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
													stroke-linecap="round"
												/>
											</svg>
										{/if}
									</button>
									<button
										type="button"
										class={['icon', { off: !opts.layers[i].invert }]}
										title={app.t.exportInvert}
										aria-label={app.t.exportInvert}
										aria-pressed={opts.layers[i].invert}
										onclick={() => (opts.layers[i].invert = !opts.layers[i].invert)}
									>
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<circle
												cx="12"
												cy="12"
												r="9"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
											/>
											<path d="M12 3a9 9 0 010 18z" fill="currentColor" />
										</svg>
									</button>
								</li>
							{/if}
						{/each}
					</ul>
				</div>
			</div>
		</div>
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={confirm}>{confirmLabel}</button>
			<button type="button" onclick={() => app.dialogs.close()}>{app.t.cancel}</button>
		</div>
	</div>
</Modal>

<style>
	.export-dlg {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: min(82vh, 760px);
		min-height: 420px;
	}
	.body {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 280px;
		gap: 16px;
		flex: 1;
		min-height: 0;
	}
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
		background-color: #d8d2cb;
		background-image:
			linear-gradient(45deg, #eeeae4 25%, transparent 25%),
			linear-gradient(-45deg, #eeeae4 25%, transparent 25%),
			linear-gradient(45deg, transparent 75%, #eeeae4 75%),
			linear-gradient(-45deg, transparent 75%, #eeeae4 75%);
		background-size: 16px 16px;
		background-position:
			0 0,
			0 8px,
			8px -8px,
			-8px 0;
	}
	.viewer.paper {
		background: #e8e4de;
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
	.hint {
		color: var(--accent);
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow: auto;
		min-height: 0;
		padding-right: 4px;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-size: 13px;
	}
	.chk {
		flex-direction: row;
		align-items: center;
		gap: 6px;
	}
	.size-hint {
		font-size: 11px;
		color: var(--fg-muted);
	}
	input[type='number'],
	select {
		width: 100%;
	}
	.layers {
		margin-top: 6px;
		border-top: 1px solid var(--border);
		padding-top: 8px;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.layers-head {
		font-size: 13px;
		font-weight: 650;
		margin-bottom: 6px;
	}
	.layers ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.layers li {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.swatch {
		width: 12px;
		height: 12px;
		border-radius: 3px;
		border: 1px solid var(--border);
		flex-shrink: 0;
	}
	.lname {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 12px;
	}
	.icon {
		width: 24px;
		height: 24px;
		padding: 0;
		flex-shrink: 0;
		display: grid;
		place-items: center;
		background: transparent;
		border-color: transparent;
		color: var(--fg-muted);
	}
	.icon svg {
		width: 14px;
		height: 14px;
		display: block;
	}
	.icon.off {
		opacity: 0.45;
	}
	.dialog-actions {
		margin-top: 0;
		flex-shrink: 0;
	}
</style>
