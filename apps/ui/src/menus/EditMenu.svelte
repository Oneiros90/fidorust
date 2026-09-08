<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';
	import MenuSeparator from './MenuSeparator.svelte';
	import MenuSubmenu from './MenuSubmenu.svelte';

	const app = getAppSession();
	const hasSelection = $derived(app.status.selected > 0);
	const editing = $derived(!!app.status.editing_component);
	const writableLibs = $derived(app.libs.filter((l) => l.writable));
</script>

<MenuItem
	label={app.t.cut}
	shortcut="Ctrl+X"
	disabled={!hasSelection}
	onclick={() => void app.cutFcd()}
/>
<MenuItem
	label={app.t.copy}
	shortcut="Ctrl+C"
	disabled={!hasSelection}
	onclick={() => void app.copyFcd()}
/>
<MenuItem label={app.t.paste} shortcut="Ctrl+V" onclick={() => void app.pasteFcd()} />
<MenuItem label={app.t.pasteNewDoc} onclick={() => void app.pasteNewDoc()} />
<MenuItem label={app.t.delete} shortcut="Del" disabled={!hasSelection} onclick={app.doDelete} />
<MenuSeparator />
<MenuItem
	label={app.t.undo}
	shortcut="Ctrl+Z"
	disabled={!app.status.can_undo}
	onclick={app.doUndo}
/>
<MenuItem
	label={app.t.redo}
	shortcut="Ctrl+Y"
	disabled={!app.status.can_redo}
	onclick={app.doRedo}
/>
<MenuSeparator />
<MenuItem label={app.t.rotate} shortcut="R" disabled={!hasSelection} onclick={app.doRotate} />
<MenuItem label={app.t.mirror} shortcut="S" disabled={!hasSelection} onclick={app.doMirror} />
<MenuSubmenu label={app.t.component}>
	<MenuSubmenu label={app.t.createFromSelection} disabled={!hasSelection}>
		{#each writableLibs as lib (lib.stem)}
			<MenuItem
				label={lib.stem === 'project' ? app.t.createInProject : lib.title}
				disabled={!hasSelection}
				onclick={() => app.createComponentFromSelection(lib.stem)}
			/>
		{/each}
	</MenuSubmenu>
	<MenuItem
		label={app.t.splitComponent}
		disabled={!app.status.can_split_component}
		onclick={app.doSplit}
	/>
	<MenuItem
		label={app.t.editComponent}
		disabled={!app.status.can_edit_component || editing}
		onclick={app.editSelectedComponent}
	/>
</MenuSubmenu>
<MenuSeparator />
<MenuItem label={app.t.selectAll} onclick={app.doSelectAll} />
<MenuItem label={app.t.invertSelection} onclick={app.doInvert} />
<MenuSeparator />
<MenuItem
	label={app.t.properties}
	shortcut="Alt+Enter"
	disabled={!hasSelection}
	onclick={app.openProperties}
/>
