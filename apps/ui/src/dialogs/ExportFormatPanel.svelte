<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { A4_PT, LETTER_PT, PNG_PPI_MAX, PNG_PPI_MIN } from '../lib/constants';
	import {
		formatCm,
		luToCm,
		ppiChoices,
		ptToCm,
		type ExportFormat,
		type ExportPreviewOpts
	} from '../lib/exportOptions';
	import {
		currentFlipH,
		currentFlipV,
		rotateExport90,
		toggleExportFlipH,
		toggleExportFlipV
	} from '../lib/exportLayout';
	import { pdfLayout } from '../lib/svgPdf';
	import { parseSvgViewBox } from '../lib/svgGeom';

	let { opts = $bindable(), svg }: { opts: ExportPreviewOpts; svg: string } = $props();

	const app = getAppSession();
	const fileFormats: Exclude<ExportFormat, 'print'>[] = ['svg', 'png', 'pdf', 'emf'];
	const a4Cm = formatCm(ptToCm(A4_PT.w), ptToCm(A4_PT.h));
	const letterCm = formatCm(ptToCm(LETTER_PT.w), ptToCm(LETTER_PT.h));

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
	const scaledSheetLabel = $derived(
		sheet && (opts.format === 'svg' || opts.format === 'emf') && opts.scale !== 1
			? formatCm(sheet.wCm * opts.scale, sheet.hCm * opts.scale)
			: null
	);
</script>

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
<div class="xform">
	<button
		type="button"
		class={['icon', { on: currentFlipH(opts) }]}
		title={app.t.exportFlipH}
		aria-label={app.t.exportFlipH}
		aria-pressed={currentFlipH(opts)}
		onclick={() => toggleExportFlipH(opts)}
	>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.75"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="M12 3.2v17.6" />
			<path d="M9.2 7.2 4.2 12l5 4.8" />
			<path d="M4.2 12H10" />
			<path d="M14.8 7.2 19.8 12l-5 4.8" />
			<path d="M19.8 12H14" />
		</svg>
	</button>
	<button
		type="button"
		class={['icon', { on: currentFlipV(opts) }]}
		title={app.t.exportFlipV}
		aria-label={app.t.exportFlipV}
		aria-pressed={currentFlipV(opts)}
		onclick={() => toggleExportFlipV(opts)}
	>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.75"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
		>
			<path d="M3.2 12h17.6" />
			<path d="M7.2 9.2 12 4.2l4.8 5" />
			<path d="M12 4.2V10" />
			<path d="M7.2 14.8 12 19.8l4.8-5" />
			<path d="M12 19.8V14" />
		</svg>
	</button>
	<button
		type="button"
		class={['icon', { on: opts.rotate !== 0 }]}
		title={app.t.exportRotate}
		aria-label={app.t.exportRotate}
		onclick={() => rotateExport90(opts)}
	>
		<svg class="rotate" viewBox="0 0 458.228 458.229" aria-hidden="true">
			<path
				fill="currentColor"
				d="M405.958 81.303c-15.021-17.78-32.731-33.265-52.6-45.852C316.754 12.259 274.402 0 230.884 0 169.685 0 112.149 23.832 68.875 67.106 25.601 110.38 1.769 167.916 1.769 229.114S25.601 347.85 68.875 391.123c43.274 43.273 100.81 67.105 162.009 67.105 47.021 0 92.212-14.145 130.688-40.906 37.567-26.131 66.195-62.395 82.788-104.87l-52.094-20.351c-26.149 66.943-89.498 110.199-161.384 110.199-95.496 0-173.188-77.691-173.188-173.188S135.387 55.925 230.883 55.925c48.337 0 93.034 19.639 125.137 53.303l-60.872 34.043 79.391 47.296 79.392 47.299 1.263-92.403 1.267-92.403L405.958 81.303z"
			/>
		</svg>
	</button>
</div>
{#if opts.format === 'png'}
	<label>
		{app.t.exportPpi}
		<input
			type="number"
			min={PNG_PPI_MIN}
			max={PNG_PPI_MAX}
			step="1"
			list="export-ppi-presets"
			bind:value={opts.ppi}
		/>
		<datalist id="export-ppi-presets">
			{#each ppiChoices as p (p)}
				<option value={p}>{p}</option>
			{/each}
		</datalist>
	</label>
	<label class="chk">
		<input type="checkbox" bind:checked={opts.antiAlias} />
		{app.t.exportAntiAlias}
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

<style>
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
	.xform {
		display: flex;
		gap: 6px;
	}
	.xform .icon {
		width: 36px;
		height: 36px;
		padding: 0;
		flex-shrink: 0;
		display: grid;
		place-items: center;
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--fg);
	}
	.xform .icon svg {
		width: 30px;
		height: 30px;
		overflow: visible;
	}
	.xform .icon svg.rotate {
		width: 20px;
		height: 20px;
	}
	.xform .icon.on {
		color: var(--accent);
		border-color: var(--accent);
		background: color-mix(in srgb, var(--accent) 16%, transparent);
	}
	input[type='number'],
	select {
		width: 100%;
	}
</style>
