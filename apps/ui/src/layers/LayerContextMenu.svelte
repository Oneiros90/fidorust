<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { otherLayers } from '../app/layerOps';
	import MenuItem from '../menus/MenuItem.svelte';
	import MenuSubmenu from '../menus/MenuSubmenu.svelte';

	let { index }: { index: number } = $props();

	const app = getAppSession();
	let layer = $derived(app.layers.layers[index]);
	let last = $derived(app.layers.layers.length - 1);
	let others = $derived(otherLayers(app.layers.layers, index));
</script>

{#if layer}
	<MenuItem label={app.t.renameLayer} onclick={() => app.beginRenameLayer(index)} />
	<MenuItem
		label={app.t.moveLayerUp}
		disabled={index <= 0}
		onclick={() => app.reorderLayer(index, index - 1)}
	/>
	<MenuItem
		label={app.t.moveLayerDown}
		disabled={index >= last}
		onclick={() => app.reorderLayer(index, index + 1)}
	/>
	<MenuItem
		label={layer.show ? app.t.hideLayer : app.t.showLayer}
		onclick={() => app.setLayerShow(index, !layer.show)}
	/>
	{#if last < 1}
		<MenuItem label={app.t.deleteLayer} disabled />
	{:else}
		<MenuSubmenu label={app.t.deleteLayer} side="left">
			<MenuItem
				label={app.t.deleteLayerObjects}
				onclick={() => app.applyDeleteLayer(index, 'objects', 0)}
			/>
			{#each others as o (o.i)}
				<MenuItem
					label="{app.t.moveLayerObjects} {o.name}"
					onclick={() => app.applyDeleteLayer(index, 'move', o.i)}
				/>
			{/each}
		</MenuSubmenu>
	{/if}
{/if}
