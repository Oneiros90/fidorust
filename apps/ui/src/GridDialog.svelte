<script lang="ts">
	import { untrack } from 'svelte';
	import type { Dict } from './i18n';

	interface Values {
		gridX: number;
		gridY: number;
		snapX: number;
		snapY: number;
		showGrid: boolean;
		snapEnable: boolean;
		hideMacroOrigin: boolean;
	}

	let {
		t,
		gridX,
		gridY,
		snapX,
		snapY,
		showGrid,
		snapEnable,
		hideMacroOrigin,
		onApply,
		onCancel
	}: {
		t: Dict;
		gridX: number;
		gridY: number;
		snapX: number;
		snapY: number;
		showGrid: boolean;
		snapEnable: boolean;
		hideMacroOrigin: boolean;
		onApply: (values: Values) => void;
		onCancel: () => void;
	} = $props();

	let editGridX = $state(untrack(() => gridX));
	let editGridY = $state(untrack(() => gridY));
	let editSnapX = $state(untrack(() => snapX));
	let editSnapY = $state(untrack(() => snapY));
	let editShowGrid = $state(untrack(() => showGrid));
	let editSnapEnable = $state(untrack(() => snapEnable));
	let editHideMacroOrigin = $state(untrack(() => hideMacroOrigin));

	function clamp(n: number, min: number, max: number) {
		const v = Math.round(Number(n));
		if (!Number.isFinite(v)) return min;
		return Math.min(max, Math.max(min, v));
	}

	function apply() {
		onApply({
			gridX: clamp(editGridX, 1, 40),
			gridY: clamp(editGridY, 1, 40),
			snapX: clamp(editSnapX, 1, 20),
			snapY: clamp(editSnapY, 1, 20),
			showGrid: editShowGrid,
			snapEnable: editSnapEnable,
			hideMacroOrigin: editHideMacroOrigin
		});
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
</script>

<svelte:window onkeydown={onKey} />

<div class="modal" role="dialog" aria-modal="true" aria-labelledby="grid-dlg-title">
	<div class="card">
		<h2 id="grid-dlg-title">{t.gridSnap}</h2>
		<div class="form">
			<label>
				{t.gridX}
				<input type="number" min="1" max="40" step="1" bind:value={editGridX} />
			</label>
			<label>
				{t.snapX}
				<input type="number" min="1" max="20" step="1" bind:value={editSnapX} />
			</label>
			<label>
				{t.gridY}
				<input type="number" min="1" max="40" step="1" bind:value={editGridY} />
			</label>
			<label>
				{t.snapY}
				<input type="number" min="1" max="20" step="1" bind:value={editSnapY} />
			</label>
			<label class="chk">
				<input type="checkbox" bind:checked={editShowGrid} />
				{t.showGrid}
			</label>
			<label class="chk">
				<input type="checkbox" bind:checked={editSnapEnable} />
				{t.enableSnap}
			</label>
			<label class="chk full">
				<input type="checkbox" bind:checked={editHideMacroOrigin} />
				{t.hideMacroOrigin}
			</label>
			<div class="actions">
				<button type="button" class="ok" onclick={apply}>{t.ok}</button>
				<button type="button" onclick={onCancel}>{t.cancel}</button>
			</div>
		</div>
	</div>
</div>

<style>
	.modal {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		display: grid;
		place-items: center;
		z-index: 10;
	}
	.card {
		background: var(--bg-menu);
		padding: 16px 18px;
		border-radius: var(--radius);
		border: 1px solid var(--border);
		max-width: 420px;
		width: min(420px, 92vw);
		box-shadow: var(--shadow);
	}
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
