<script lang="ts">
	import { untrack } from 'svelte';
	import type { Dict } from '../i18n';
	import type { LayersData } from '../app/engineTypes';
	import { rgbaCss } from '../lib/color';
	import { clampInt } from '../lib/num';
	import {
		editStateToPatch,
		fieldLabels,
		type PropFieldId,
		type PropFieldKind,
		type PropFieldValue,
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

	let edit = $state<Partial<Record<PropFieldId, PropFieldValue>>>(
		untrack(() => Object.fromEntries(fields.map((f) => [f.id, f.value])))
	);

	function layerColor(i: number): string {
		const l = layers.layers[i];
		if (!l) return '#888';
		return rgbaCss(l.color);
	}

	function apply() {
		for (const f of fields) {
			if (f.kind.kind !== 'int') continue;
			const st = edit[f.id];
			if (st?.state === 'int') {
				edit[f.id] = {
					state: 'int',
					value: clampInt(st.value, f.kind.min, f.kind.max)
				};
			}
		}
		onApply(editStateToPatch(edit));
	}

	function getValue(id: PropFieldId): string {
		const st = edit[id];
		if (!st || st.state === 'unset') return '';
		if (st.state === 'bool') return st.value ? 'true' : 'false';
		if (st.state === 'int' || st.state === 'layer') return String(st.value);
		return st.value;
	}

	function setValue(id: PropFieldId, kind: PropFieldKind['kind'], raw: string) {
		if (raw === '') {
			edit[id] = { state: 'unset' };
			return;
		}
		if (kind === 'bool') edit[id] = { state: 'bool', value: raw === 'true' };
		else if (kind === 'int') edit[id] = { state: 'int', value: Number(raw) };
		else if (kind === 'string' || kind === 'choice') edit[id] = { state: 'string', value: raw };
		else if (kind === 'layer') edit[id] = { state: 'layer', value: Number(raw) };
		else edit[id] = { state: 'padStyle', value: raw };
	}

	const useComponentLayersOn = $derived(
		edit.useComponentLayers?.state === 'bool' && edit.useComponentLayers.value
	);
</script>

<Modal
	title={t.propTitle}
	titleId="props-dlg-title"
	maxWidth="480px"
	onClose={onCancel}
	onSubmit={apply}
>
	<div class="form">
		{#each fields as field (field.id)}
			<label
				class={[
					field.id === 'text' || field.id === 'useComponentLayers' ? 'full' : '',
					field.id === 'useComponentLayers' && 'check',
					field.kind.kind === 'layer' && (field.readOnly || useComponentLayersOn) && 'off'
				]}
				title={field.id === 'useComponentLayers' ? t.propUseComponentLayersTooltip : undefined}
			>
				{t[fieldLabels[field.id]]}
				{#if field.id === 'useComponentLayers'}
					<input
						type="checkbox"
						disabled={field.readOnly}
						checked={edit.useComponentLayers?.state === 'bool' && edit.useComponentLayers.value}
						{@attach (node) => {
							$effect(() => {
								const st = edit.useComponentLayers;
								node.indeterminate = !st || st.state === 'unset';
							});
						}}
						onchange={(e) =>
							setValue(field.id, 'bool', e.currentTarget.checked ? 'true' : 'false')}
					/>
				{:else if field.kind.kind === 'bool'}
					<select
						disabled={field.readOnly}
						value={getValue(field.id)}
						onchange={(e) => setValue(field.id, 'bool', e.currentTarget.value)}
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
						value={getValue(field.id)}
						oninput={(e) => setValue(field.id, 'int', e.currentTarget.value)}
					/>
				{:else if field.kind.kind === 'string'}
					<input
						type="text"
						disabled={field.readOnly}
						value={getValue(field.id)}
						oninput={(e) => setValue(field.id, 'string', e.currentTarget.value)}
					/>
				{:else if field.kind.kind === 'choice'}
					<select
						disabled={field.readOnly}
						value={getValue(field.id)}
						onchange={(e) => setValue(field.id, 'choice', e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						{#each field.kind.options as name (name)}
							<option value={name}>{name}</option>
						{/each}
					</select>
				{:else if field.kind.kind === 'layer'}
					<select
						disabled={field.readOnly || useComponentLayersOn}
						value={getValue(field.id)}
						onchange={(e) => setValue(field.id, 'layer', e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						{#each layers.layers as l, i (i)}
							<option value={String(i)}>
								{l.name || `${t.layer} ${i}`}
							</option>
						{/each}
					</select>
					{#if getValue(field.id) !== ''}
						<span
							class="swatch"
							style:background={layerColor(Number(getValue(field.id)))}
							aria-hidden="true"
						></span>
					{/if}
				{:else if field.kind.kind === 'padStyle'}
					<select
						disabled={field.readOnly}
						value={getValue(field.id)}
						onchange={(e) => setValue(field.id, 'padStyle', e.currentTarget.value)}
					>
						<option value="">{t.indeterminate}</option>
						<option value="Round">{t.padRound}</option>
						<option value="Square">{t.padSquare}</option>
						<option value="SquareRounded">{t.padSquareRounded}</option>
					</select>
				{/if}
			</label>
		{/each}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={apply}>{t.ok}</button>
			<button type="button" onclick={onCancel}>{t.cancel}</button>
		</div>
	</div>
</Modal>

<style>
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
	.check {
		flex-direction: row;
		align-items: center;
		gap: 8px;
	}
	.check input[type='checkbox'] {
		width: auto;
		margin: 0;
	}
	select,
	input[type='number'],
	input[type='text'] {
		width: 100%;
	}
	.off {
		color: var(--fg-muted);
	}
	.off select:disabled {
		background: var(--bg-panel);
	}
	.off .swatch {
		opacity: 0.4;
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
	.dialog-actions {
		grid-column: 1 / -1;
		margin-top: 8px;
	}
</style>
