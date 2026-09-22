<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import CanvasHost from './CanvasHost.svelte';
	import SheetTabBar from './SheetTabBar.svelte';

	let { pane }: { pane: number } = $props();

	const app = getAppSession();
</script>

<div class="pane">
	<CanvasHost {pane} />
	{#if !app.status.editing_component}
		<SheetTabBar {pane} />
	{/if}
</div>

<style>
	.pane {
		flex: 1 1 0;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
</style>
