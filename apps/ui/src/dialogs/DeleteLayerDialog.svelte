<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { otherLayers } from '../app/layerOps';
	import Modal from './Modal.svelte';

	let { index }: { index: number } = $props();

	const app = getAppSession();

	let others = $derived(otherLayers(app.layers.layers, index));
	let moveTo = $state<number | null>(null);

	function dest() {
		const v = moveTo === null ? (others[0]?.i ?? 0) : Number(moveTo);
		return others.some((o) => o.i === v) ? v : (others[0]?.i ?? 0);
	}
</script>

<Modal
	title={app.t.deleteLayerTitle}
	titleId="delete-layer-title"
	maxWidth="420px"
	onClose={app.cancelDeleteLayer}
>
	<p>{app.t.deleteLayerBody}</p>
	<label class="move">
		{app.t.moveLayerObjects}
		<select value={dest()} onchange={(e) => (moveTo = Number(e.currentTarget.value))}>
			{#each others as o (o.i)}
				<option value={o.i}>{o.name}</option>
			{/each}
		</select>
	</label>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="danger" onclick={() => app.confirmDeleteLayer('objects', 0)}>
				{app.t.deleteLayerObjects}
			</button>
			<button type="button" class="primary" onclick={() => app.confirmDeleteLayer('move', dest())}>
				{app.t.moveLayer}
			</button>
			<button type="button" onclick={app.cancelDeleteLayer}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
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
	.dialog-actions {
		flex-wrap: wrap;
	}
	.danger {
		color: var(--danger);
		border-color: var(--danger);
		margin-right: auto;
	}
</style>
