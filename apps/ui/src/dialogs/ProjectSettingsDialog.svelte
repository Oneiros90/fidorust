<script lang="ts">
	import { untrack } from 'svelte';
	import type { DrawingDefaultsValues } from '../app/appSession.svelte';
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
		values: DrawingDefaultsValues;
		onApply: (values: DrawingDefaultsValues) => void;
		onCancel: () => void;
	} = $props();

	let edit = $state(
		untrack(() => ({
			...values,
			strokeLu: values.strokeHundredths / 100
		}))
	);

	function apply() {
		onApply({
			hideComponentOrigin: edit.hideComponentOrigin,
			strokeHundredths: clampInt(edit.strokeLu * 100, 1, 2000),
			defaultFilled: edit.defaultFilled
		});
	}
</script>

<Modal
	title={t.projectSettings}
	titleId="project-settings-title"
	maxWidth="480px"
	onClose={onCancel}
	onSubmit={apply}
>
	<div class="pane">
		<h3>{t.sectionDefaults}</h3>
		<label class="chk">
			<input type="checkbox" bind:checked={edit.hideComponentOrigin} />
			{t.hideComponentOrigin}
		</label>
		<label>
			{t.defaultLineWidth}
			<input type="number" min="0.01" max="20" step="0.05" bind:value={edit.strokeLu} />
		</label>
		<label class="chk">
			<input type="checkbox" bind:checked={edit.defaultFilled} />
			{t.defaultFill}
		</label>
	</div>
	<div class="dialog-actions">
		<button type="button" class="primary" onclick={apply}>{t.ok}</button>
		<button type="button" onclick={onCancel}>{t.cancel}</button>
	</div>
</Modal>

<style>
	.pane {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 16px;
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
	input[type='number'] {
		width: 100%;
	}
</style>
