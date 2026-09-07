<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();

	let others = $derived(
		app.layers.layers
			.map((l, i) => ({ name: l.name, i }))
			.filter((x) => x.i !== app.pendingDeleteLayer)
	);
	let moveTo = $state<number | null>(null);

	function dest() {
		const v = moveTo === null ? (others[0]?.i ?? 0) : Number(moveTo);
		return others.some((o) => o.i === v) ? v : (others[0]?.i ?? 0);
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			app.cancelDeleteLayer();
		}
	}
</script>

<svelte:window onkeydown={onKey} />

<Modal labelledBy="delete-layer-title" maxWidth="420px">
	<h2 id="delete-layer-title">{app.t.deleteLayerTitle}</h2>
	<p>{app.t.deleteLayerBody}</p>
	<label class="move">
		{app.t.moveLayerObjects}
		<select value={dest()} onchange={(e) => (moveTo = Number(e.currentTarget.value))}>
			{#each others as o (o.i)}
				<option value={o.i}>{o.name}</option>
			{/each}
		</select>
	</label>
	<div class="actions">
		<button type="button" class="danger" onclick={() => app.confirmDeleteLayer('objects', 0)}>
			{app.t.deleteLayerObjects}
		</button>
		<button type="button" class="ok" onclick={() => app.confirmDeleteLayer('move', dest())}>
			{app.t.moveLayer}
		</button>
		<button type="button" onclick={app.cancelDeleteLayer}>{app.t.cancel}</button>
	</div>
</Modal>

<style>
	h2 {
		margin: 0 0 12px;
		font-size: 15px;
		font-weight: 600;
	}
	p {
		margin: 0 0 12px;
		font-size: 13px;
	}
	.move {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0 0 16px;
		font-size: 13px;
	}
	.move select {
		flex: 1;
		min-width: 0;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		flex-wrap: wrap;
		gap: 8px;
	}
	.ok {
		background: var(--accent);
		color: var(--accent-fg);
		border-color: var(--accent);
	}
	.danger {
		color: var(--danger);
		border-color: var(--danger);
		margin-right: auto;
	}
</style>
