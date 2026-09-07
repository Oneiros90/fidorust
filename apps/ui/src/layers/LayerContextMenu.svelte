<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from '../menus/MenuItem.svelte';
	import MenuSubmenu from '../menus/MenuSubmenu.svelte';

	let { index, onDone }: { index: number; onDone: () => void } = $props();

	const app = getAppSession();
	let layer = $derived(app.layers.layers[index]);
	let last = $derived(app.layers.layers.length - 1);
	let others = $derived(
		app.layers.layers.map((l, i) => ({ name: l.name, i })).filter((x) => x.i !== index)
	);

	function run(fn: () => void) {
		fn();
		onDone();
	}
</script>

{#if layer}
	<MenuItem label={app.t.renameLayer} onclick={() => run(() => app.beginRenameLayer(index))} />
	<MenuItem
		label={app.t.moveLayerUp}
		disabled={index <= 0}
		onclick={() => run(() => app.reorderLayer(index, index - 1))}
	/>
	<MenuItem
		label={app.t.moveLayerDown}
		disabled={index >= last}
		onclick={() => run(() => app.reorderLayer(index, index + 1))}
	/>
	<MenuItem
		label={layer.show ? app.t.hideLayer : app.t.showLayer}
		onclick={() => run(() => app.setLayerShow(index, !layer.show))}
	/>
	{#if last < 1}
		<MenuItem label={app.t.deleteLayer} disabled />
	{:else}
		<MenuSubmenu label={app.t.deleteLayer} side="left">
			<MenuItem
				label={app.t.deleteLayerObjects}
				onclick={() => run(() => app.applyDeleteLayer(index, 'objects', 0))}
			/>
			{#each others as o (o.i)}
				<MenuItem
					label="{app.t.moveLayerObjects} {o.name}"
					onclick={() => run(() => app.applyDeleteLayer(index, 'move', o.i))}
				/>
			{/each}
		</MenuSubmenu>
	{/if}
{/if}
