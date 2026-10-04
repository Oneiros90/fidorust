<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { sectionForTool, toolHasDefaults } from '../app/projectSettings';
	import MenuItem from '../menus/MenuItem.svelte';

	let { toolId }: { toolId: string } = $props();

	const app = getAppSession();
</script>

<MenuItem label={app.t.selectTool} onclick={() => app.tool(toolId)} />
{#if toolHasDefaults(toolId)}
	<MenuItem
		label={app.t.editDefaultProperties}
		onclick={() => app.openProjectSettings(sectionForTool(toolId))}
	/>
{/if}
