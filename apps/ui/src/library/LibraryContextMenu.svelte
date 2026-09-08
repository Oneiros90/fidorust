<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from '../menus/MenuItem.svelte';
	import MenuSeparator from '../menus/MenuSeparator.svelte';

	let { stem }: { stem: string } = $props();

	const app = getAppSession();
	const isProject = $derived(stem === 'project');
</script>

{#if !isProject}
	<MenuItem label={app.t.renameLibrary} onclick={() => app.beginRenameLibrary(stem)} />
{/if}
<MenuItem label={app.t.exportLibrary} onclick={() => app.exportLibrary(stem)} />
<MenuSeparator />
<MenuItem label={app.t.deleteLibrary} onclick={() => app.requestDeleteLibrary(stem)} />
