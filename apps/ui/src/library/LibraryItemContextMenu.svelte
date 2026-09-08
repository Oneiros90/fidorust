<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from '../menus/MenuItem.svelte';
	import MenuSeparator from '../menus/MenuSeparator.svelte';
	import MenuSubmenu from '../menus/MenuSubmenu.svelte';

	let { stem, componentKey }: { stem: string; componentKey: string } = $props();

	const app = getAppSession();
	const destinations = $derived(
		app.libs.filter((l) => l.writable && l.stem !== stem).map((l) => l.stem)
	);
	const editing = $derived(!!app.status.editing_component);

	function titleFor(s: string) {
		return app.libraryTitle(s);
	}
</script>

<MenuItem
	label={app.t.editComponent}
	disabled={editing}
	onclick={() => app.enterComponentEdit(stem, componentKey)}
/>
<MenuItem
	label={app.t.renameComponent}
	onclick={() => app.beginRenameComponent(stem, componentKey)}
/>
<MenuItem
	label={app.t.editDescription}
	onclick={() => app.beginEditComponentDescription(stem, componentKey)}
/>
{#if destinations.length > 0}
	<MenuSubmenu label={app.t.moveComponentTo} side="left">
		{#each destinations as dest (dest)}
			<MenuItem
				label={titleFor(dest)}
				onclick={() => app.moveComponent(stem, componentKey, dest)}
			/>
		{/each}
	</MenuSubmenu>
{/if}
<MenuSeparator />
<MenuItem
	label={app.t.deleteComponent}
	onclick={() => app.requestDeleteComponent(stem, componentKey)}
/>
