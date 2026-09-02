<script lang="ts">
	import { untrack } from 'svelte';
	import type { Dict } from '../i18n';
	import type { LayersData } from '../app/types';
	import {
		editStateToPatch,
		initEditState,
		type PropEditState,
		type PropFieldId,
		type PropFormField
	} from '../lib/propForm';
	import Modal from './Modal.svelte';

	let {
		t,
		fields,
		layers,
		onApply,
		onCancel
	}: {
		t: Dict;
		fields: PropFormField[];
		layers: LayersData;
		onApply: (patch: ReturnType<typeof editStateToPatch>) => void;
		onCancel: () => void;
	} = $props();

	let edit = $state<PropEditState>(untrack(() => initEditState(fields)));

	const fieldLabels: Record<PropFieldId, keyof Dict> = {
		filled: 'propFilled',
		layer: 'layer',
		thickness: 'propThickness',
		sizeX: 'propSizeX',
		sizeY: 'propSizeY',
		intDiam: 'propIntDiam',
		padStyle: 'propPadStyle',
		text: 'propText',
		fontFace: 'propFontFace',
		fontHeight: 'propFontHeight',
		fontWidth: 'propFontWidth',
		rotationAngle: 'propRotationAngle',
		bold: 'propBold',
		italic: 'propItalic',
		mirrored: 'propMirrored',
		underlined: 'propUnderlined'
	};

	function layerColor(i: number): string {
		const l = layers.layers[i];
		if (!l) return '#888';
		return (
			'#' +
			l.color.map((c) => c.toString(16).padStart(2, '0')).join('')
		);
	}

	function clamp(n: number, min: number, max: number) {
		const v = Math.round(Number(n));
		if (!Number.isFinite(v)) return min;
		return Math.min(max, Math.max(min, v));
	}

	function apply() {
		for (const f of fields) {
			if (f.kind.kind !== 'int') continue;
			const st = edit[f.id];
			if (st?.mode === 'int') {
				edit[f.id] = {
					mode: 'int',
					value: clamp(st.value, f.kind.min, f.kind.max)
				};
			}
		}
		onApply(editStateToPatch(edit));
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onCancel();
			return;
		}
		if (e.key === 'Enter') {
			if (e.target instanceof HTMLButtonElement) return;
			e.preventDefault();
			apply();
		}
	}

	function setBool(id: PropFieldId, raw: string) {
		if (raw === '') edit[id] = { mode: 'unset' };
		else edit[id] = { mode: 'bool', value: raw === 'true' };
	}

	function setInt(id: PropFieldId, raw: string) {
		if (raw === '') edit[id] = { mode: 'unset' };
		else edit[id] = { mode: 'int', value: Number(raw) };
	}

	function setString(id: PropFieldId, raw: string) {
		if (raw === '') edit[id] = { mode: 'unset' };
		else edit[id] = { mode: 'string', value: raw };
	}

	function setLayer(raw: string) {
		if (raw === '') edit.layer = { mode: 'unset' };
		else edit.layer = { mode: 'layer', value: Number(raw) };
	}

	function setPadStyle(raw: string) {
		if (raw === '') edit.padStyle = { mode: 'unset' };
		else edit.padStyle = { mode: 'padStyle', value: raw };
	}

	function boolSelectValue(id: PropFieldId): string {
		const st = edit[id];
		if (!st || st.mode === 'unset') return '';
		if (st.mode === 'bool') return st.value ? 'true' : 'false';
		return '';
	}

	function intInputValue(id: PropFieldId): string {
		const st = edit[id];
		if (!st || st.mode === 'unset') return '';
		if (st.mode === 'int') return String(st.value);
		return '';
	}

	function stringInputValue(id: PropFieldId): string {
		const st = edit[id];
		if (!st || st.mode === 'unset') return '';
		if (st.mode === 'string') return st.value;
		return '';
	}

	function layerSelectValue(): string {
		const st = edit.layer;
		if (!st || st.mode === 'unset') return '';
		if (st.mode === 'layer') return String(st.value);
		return '';
	}

	function padStyleSelectValue(): string {
		const st = edit.padStyle;
		if (!st || st.mode === 'unset') return '';
		if (st.mode === 'padStyle') return st.value;
		return '';
	}
</script>

<svelte:window onkeydown={onKey} />

<Modal labelledBy="props-dlg-title" maxWidth="480px">
	<h2 id="props-dlg-title">{t.propTitle}</h2>
	<div class="form">
		{#each fields as field (field.id)}
			<label class={field.id === 'text' ? 'full' : ''}>
				{t[fieldLabels[field.id]]}
				{#if field.kind.kind === 'bool'}
					<select
						disabled={field.readOnly}
						value={boolSelectValue(field.id)}
						onchange={(e) => setBool(field.id, e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						<option value="true">{t.yes}</option>
						<option value="false">{t.no}</option>
					</select>
				{:else if field.kind.kind === 'int'}
					<input
						type="number"
						min={field.kind.min}
						max={field.kind.max}
						step="1"
						placeholder={t.indeterminate}
						disabled={field.readOnly}
						value={intInputValue(field.id)}
						oninput={(e) => setInt(field.id, e.currentTarget.value)}
					/>
				{:else if field.kind.kind === 'string'}
					<input
						type="text"
						disabled={field.readOnly}
						value={stringInputValue(field.id)}
						oninput={(e) => setString(field.id, e.currentTarget.value)}
					/>
				{:else if field.kind.kind === 'layer'}
					<select
						disabled={field.readOnly}
						value={layerSelectValue()}
						onchange={(e) => setLayer(e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						{#each layers.layers as l, i (i)}
							<option value={String(i)}>
								{l.name || `${t.layer} ${i}`}
							</option>
						{/each}
					</select>
					{#if layerSelectValue() !== ''}
						<span
							class="swatch"
							style:background={layerColor(Number(layerSelectValue()))}
							aria-hidden="true"
						></span>
					{/if}
				{:else if field.kind.kind === 'padStyle'}
					<select
						disabled={field.readOnly}
						value={padStyleSelectValue()}
						onchange={(e) => setPadStyle(e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						<option value="Round">{t.padRound}</option>
						<option value="Square">{t.padSquare}</option>
						<option value="SquareRounded">{t.padSquareRounded}</option>
					</select>
				{/if}
			</label>
		{/each}
		<div class="actions">
			<button type="button" class="ok" onclick={apply}>{t.ok}</button>
			<button type="button" onclick={onCancel}>{t.cancel}</button>
		</div>
	</div>
</Modal>

<style>
	h2 {
		margin: 0 0 12px;
		font-size: 15px;
		font-weight: 600;
	}
	.form {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px 16px;
		align-items: end;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 3px;
		font-size: 13px;
		position: relative;
	}
	.full {
		grid-column: 1 / -1;
	}
	select,
	input[type='number'],
	input[type='text'] {
		width: 100%;
	}
	.swatch {
		position: absolute;
		right: 4px;
		bottom: 6px;
		width: 14px;
		height: 14px;
		border: 1px solid var(--border);
		border-radius: 2px;
		pointer-events: none;
	}
	.actions {
		grid-column: 1 / -1;
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.ok {
		background: var(--accent);
		color: var(--accent-fg);
		border-color: var(--accent);
	}
</style>
