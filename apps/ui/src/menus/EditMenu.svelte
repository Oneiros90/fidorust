<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { commonNodeActionLabel } from '../i18n';
	import MenuItem from './MenuItem.svelte';
	import MenuSeparator from './MenuSeparator.svelte';
	import MenuSubmenu from './MenuSubmenu.svelte';

	const app = getAppSession();
	const hasSelection = $derived(app.status.selected > 0);
	const editing = $derived(!!app.status.editing_component);
	const writableLibs = $derived(app.libs.filter((l) => l.writable));
	const commonNodeLabel = $derived(
		commonNodeActionLabel(
			app.t,
			app.status.common_node_names,
			app.status.common_node_count,
			app.status.common_node_on
		)
	);
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
<MenuItem
	label={app.t.duplicate}
	shortcut="Ctrl+D"
	disabled={!hasSelection}
	onclick={app.duplicateSelection}
/>
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
		label={
			app.status.selected_library_name
				? app.t.editComponent.replace('{name}', app.status.selected_library_name)
				: app.t.editComponentNone
		}
		disabled={!app.status.can_edit_component || editing}
		onclick={app.editSelectedComponent}
	/>
	<MenuItem
		label={
			app.status.selected_library_name
				? app.t.selectInLibrary.replace('{name}', app.status.selected_library_name)
				: app.t.selectInLibraryNone
		}
		disabled={!app.status.selected_library_stem}
		onclick={app.selectSelectedLibraryComponent}
	/>
</MenuSubmenu>
{#if app.pro.licensed || app.pro.wasmPro}
	<MenuSubmenu label={app.t.circuitMode} pro>
		<MenuItem
			label={commonNodeLabel}
			title={app.status.common_node_on ? app.t.separateNodeHint : app.t.commonNodeHint}
			disabled={!app.status.can_common_node}
			onclick={app.doToggleCommonNode}
		/>
	</MenuSubmenu>
{/if}
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
