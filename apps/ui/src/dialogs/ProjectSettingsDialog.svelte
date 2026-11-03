<script lang="ts">
	import { untrack } from 'svelte';
	import type { DrawingDefaultsValues } from '../app/appSession.svelte';
	import type { PrimitiveDefaults } from '../app/engineTypes';
	import { TOOL_DEFAULT_SECTIONS, type ProjectSettingsSection } from '../app/projectSettings';
	import type { Dict } from '../i18n';
	import { clampInt } from '../lib/num';
	import Modal from './Modal.svelte';

	const STYLE_BOLD = 1;
	const STYLE_ITALIC = 2;
	const STYLE_MIRRORED = 4;
	const STYLE_UNDERLINE = 8;

	let {
		t,
		values,
		initialSection = 'general',
		onApply,
		onCancel
	}: {
		t: Dict;
		values: DrawingDefaultsValues;
		initialSection?: ProjectSettingsSection;
		onApply: (values: DrawingDefaultsValues) => void;
		onCancel: () => void;
	} = $props();

	let section = $state<ProjectSettingsSection>(
		untrack(() =>
			initialSection === 'general' || TOOL_DEFAULT_SECTIONS.some((s) => s.id === initialSection)
				? initialSection
				: 'general'
		)
	);
	function cloneDefaults(d: PrimitiveDefaults): PrimitiveDefaults {
		return {
			line_stroke_hundredths: d.line_stroke_hundredths,
			bezier_stroke_hundredths: d.bezier_stroke_hundredths,
			rect: { ...d.rect },
			ellipse: { ...d.ellipse },
			poly: { ...d.poly },
			text: { ...d.text },
			track_width: d.track_width,
			pad: { ...d.pad }
		};
	}

	let hideComponentOrigin = $state(untrack(() => values.hideComponentOrigin));
	let defaults = $state<PrimitiveDefaults>(untrack(() => cloneDefaults(values.defaults)));
	let lineStrokeLu = $state(untrack(() => values.defaults.line_stroke_hundredths / 100));
	let bezierStrokeLu = $state(untrack(() => values.defaults.bezier_stroke_hundredths / 100));
	let rectStrokeLu = $state(untrack(() => values.defaults.rect.stroke_hundredths / 100));
	let ellipseStrokeLu = $state(untrack(() => values.defaults.ellipse.stroke_hundredths / 100));
	let polyStrokeLu = $state(untrack(() => values.defaults.poly.stroke_hundredths / 100));

	function clampStrokeLu(lu: number): number {
		return clampInt(lu * 100, 1, 2000);
	}

	function styleBit(mask: number): boolean {
		return (defaults.text.style & mask) !== 0;
	}

	function setStyleBit(mask: number, on: boolean) {
		defaults.text.style = on ? defaults.text.style | mask : defaults.text.style & ~mask;
	}

	function apply() {
		onApply({
			hideComponentOrigin,
			defaults: {
				...defaults,
				line_stroke_hundredths: clampStrokeLu(lineStrokeLu),
				bezier_stroke_hundredths: clampStrokeLu(bezierStrokeLu),
				rect: {
					...defaults.rect,
					stroke_hundredths: clampStrokeLu(rectStrokeLu)
				},
				ellipse: {
					...defaults.ellipse,
					stroke_hundredths: clampStrokeLu(ellipseStrokeLu)
				},
				poly: {
					...defaults.poly,
					stroke_hundredths: clampStrokeLu(polyStrokeLu)
				},
				text: {
					...defaults.text,
					sy: clampInt(defaults.text.sy, 1, 100),
					sx: clampInt(defaults.text.sx, 1, 100),
					font: defaults.text.font.trim() || 'Courier Prime'
				},
				track_width: clampInt(defaults.track_width, 1, 100),
				pad: {
					...defaults.pad,
					dx: clampInt(defaults.pad.dx, 1, 100),
					dy: clampInt(defaults.pad.dy, 1, 100),
					hole: clampInt(defaults.pad.hole, 0, 100)
				}
			}
		});
	}
</script>

<Modal
	title={t.projectSettings}
	titleId="project-settings-title"
	maxWidth="720px"
	maxHeight="520px"
	overflow="hidden"
	onClose={onCancel}
	onSubmit={apply}
>
	<div class="settings">
		<nav class="toc" aria-label={t.projectSettings}>
			<button
				type="button"
				class={['toc-item', { active: section === 'general' }]}
				aria-current={section === 'general' ? 'true' : undefined}
				onclick={() => (section = 'general')}
			>
				{t.sectionGeneral}
			</button>
			<div class="toc-header">{t.sectionDefaults}</div>
			{#each TOOL_DEFAULT_SECTIONS as s (s.id)}
				<button
					type="button"
					class={['toc-item', { active: section === s.id }]}
					aria-current={section === s.id ? 'true' : undefined}
					onclick={() => (section = s.id)}
				>
					{t[s.label]}
				</button>
			{/each}
		</nav>
		<div class="pane">
			{#if section === 'general'}
				<h3>{t.sectionGeneral}</h3>
				<label class="chk">
					<input type="checkbox" bind:checked={hideComponentOrigin} />
					{t.hideComponentOrigin}
				</label>
			{:else if section === 'line'}
				<h3>{t.line}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={lineStrokeLu} />
				</label>
			{:else if section === 'bezier'}
				<h3>{t.bezier}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={bezierStrokeLu} />
				</label>
			{:else if section === 'rect'}
				<h3>{t.rect}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={rectStrokeLu} />
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={defaults.rect.filled} />
					{t.propFilled}
				</label>
			{:else if section === 'ellipse'}
				<h3>{t.ellipse}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={ellipseStrokeLu} />
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={defaults.ellipse.filled} />
					{t.propFilled}
				</label>
			{:else if section === 'poly'}
				<h3>{t.poly}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={polyStrokeLu} />
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={defaults.poly.filled} />
					{t.propFilled}
				</label>
			{:else if section === 'text'}
				<h3>{t.text}</h3>
				<label>
					{t.propFontHeight}
					<input type="number" min="1" max="100" step="1" bind:value={defaults.text.sy} />
				</label>
				<label>
					{t.propFontWidth}
					<input type="number" min="1" max="100" step="1" bind:value={defaults.text.sx} />
				</label>
				<label>
					{t.propFontFace}
					<input type="text" bind:value={defaults.text.font} />
				</label>
				<label class="chk">
					<input
						type="checkbox"
						checked={styleBit(STYLE_BOLD)}
						onchange={(e) => setStyleBit(STYLE_BOLD, e.currentTarget.checked)}
					/>
					{t.propBold}
				</label>
				<label class="chk">
					<input
						type="checkbox"
						checked={styleBit(STYLE_ITALIC)}
						onchange={(e) => setStyleBit(STYLE_ITALIC, e.currentTarget.checked)}
					/>
					{t.propItalic}
				</label>
				<label class="chk">
					<input
						type="checkbox"
						checked={styleBit(STYLE_MIRRORED)}
						onchange={(e) => setStyleBit(STYLE_MIRRORED, e.currentTarget.checked)}
					/>
					{t.propMirrored}
				</label>
				<label class="chk">
					<input
						type="checkbox"
						checked={styleBit(STYLE_UNDERLINE)}
						onchange={(e) => setStyleBit(STYLE_UNDERLINE, e.currentTarget.checked)}
					/>
					{t.propUnderlined}
				</label>
			{:else if section === 'pcb-track'}
				<h3>{t.pcbTrack}</h3>
				<label>
					{t.propThickness}
					<input type="number" min="1" max="100" step="1" bind:value={defaults.track_width} />
				</label>
			{:else}
				<h3>{t.pcbPad}</h3>
				<label>
					{t.propSizeX}
					<input type="number" min="1" max="100" step="1" bind:value={defaults.pad.dx} />
				</label>
				<label>
					{t.propSizeY}
					<input type="number" min="1" max="100" step="1" bind:value={defaults.pad.dy} />
				</label>
				<label>
					{t.propIntDiam}
					<input type="number" min="0" max="100" step="1" bind:value={defaults.pad.hole} />
				</label>
				<label>
					{t.propPadStyle}
					<select bind:value={defaults.pad.style}>
						<option value="Oval">{t.padRound}</option>
						<option value="Rectangular">{t.padSquare}</option>
						<option value="RoundedRect">{t.padSquareRounded}</option>
					</select>
				</label>
			{/if}
		</div>
	</div>
	<div class="dialog-actions">
		<button type="button" class="primary" onclick={apply}>{t.ok}</button>
		<button type="button" onclick={onCancel}>{t.cancel}</button>
	</div>
</Modal>

<style>
	.settings {
		display: grid;
		grid-template-columns: 168px 1fr;
		min-height: 280px;
		max-height: 360px;
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
		margin-bottom: 16px;
	}
	.toc {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 6px;
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
		overflow: auto;
	}
	.toc-header {
		margin: 8px 10px 2px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--fg-muted);
	}
	.toc-item {
		text-align: left;
		padding: 7px 10px;
		border: 0;
		border-radius: 4px;
		background: transparent;
		color: var(--fg);
		font: inherit;
		font-size: 13px;
		cursor: pointer;
	}
	.toc-item:hover {
		background: color-mix(in srgb, var(--fg) 8%, transparent);
	}
	.toc-item.active {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	.pane {
		padding: 16px 20px;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.pane h3 {
		margin: 0 0 4px;
		font-size: 14px;
		font-weight: 600;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 13px;
		max-width: 280px;
	}
	.chk {
		flex-direction: row;
		align-items: center;
		gap: 8px;
		max-width: none;
	}
	input[type='number'],
	input[type='text'],
	select {
		width: 100%;
	}
</style>
