<script lang="ts">
	import { untrack } from 'svelte';
	import type { GridValues } from '../app/appSession.svelte';
	import type { Dict } from '../i18n';
	import { clampInt } from '../lib/num';
	import Modal from './Modal.svelte';

	let {
		t,
		values,
		onApply,
		onCancel
	}: {
		t: Dict;
		values: GridValues;
		onApply: (values: GridValues) => void;
		onCancel: () => void;
	} = $props();

	let edit = $state(untrack(() => ({ ...values })));

	function apply() {
		onApply({
			gridX: clampInt(edit.gridX, 1, 40),
			gridY: clampInt(edit.gridY, 1, 40),
			snapX: clampInt(edit.snapX, 1, 20),
			snapY: clampInt(edit.snapY, 1, 20),
			showGrid: edit.showGrid,
			snapEnable: edit.snapEnable,
			hideComponentOrigin: edit.hideComponentOrigin
		});
	}
</script>

<Modal
	title={t.gridSnap}
	titleId="grid-dlg-title"
	maxWidth="420px"
	onClose={onCancel}
	onSubmit={apply}
>
	<div class="form">
		<label>
			{t.gridX}
			<input type="number" min="1" max="40" step="1" bind:value={edit.gridX} />
		</label>
		<label>
			{t.snapX}
			<input type="number" min="1" max="20" step="1" bind:value={edit.snapX} />
		</label>
		<label>
			{t.gridY}
			<input type="number" min="1" max="40" step="1" bind:value={edit.gridY} />
		</label>
		<label>
			{t.snapY}
			<input type="number" min="1" max="20" step="1" bind:value={edit.snapY} />
		</label>
		<label class="chk">
			<input type="checkbox" bind:checked={edit.showGrid} />
			{t.showGrid}
		</label>
		<label class="chk">
			<input type="checkbox" bind:checked={edit.snapEnable} />
			{t.enableSnap}
		</label>
		<label class="chk full">
			<input type="checkbox" bind:checked={edit.hideComponentOrigin} />
			{t.hideComponentOrigin}
		</label>
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
	}
	.chk {
		flex-direction: row;
		align-items: center;
		gap: 6px;
	}
	.full {
		grid-column: 1 / -1;
		margin-top: 4px;
	}
	input[type='number'] {
		width: 100%;
	}
	.dialog-actions {
		grid-column: 1 / -1;
		margin-top: 8px;
	}
</style>
