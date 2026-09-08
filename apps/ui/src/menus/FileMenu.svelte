<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { EXAMPLES } from '../lib/examples';
	import MenuItem from './MenuItem.svelte';
	import MenuSeparator from './MenuSeparator.svelte';
	import MenuSubmenu from './MenuSubmenu.svelte';

	const app = getAppSession();
</script>

<MenuItem label={app.t.new} onclick={app.requestNewDoc} />
<MenuSeparator />
<MenuSubmenu label={app.t.open}>
	<MenuItem label={app.t.openFromFile} shortcut="Ctrl+O" onclick={app.openFile} />
	<MenuItem label={app.t.openFromClipboard} onclick={() => void app.pasteNewDoc()} />
	<MenuSubmenu label={app.t.recent}>
		{#each app.recents as r (r.name)}
			<MenuItem label={r.name} onclick={() => app.openRecent(r)} />
		{/each}
		{#if app.recents.length === 0}
			<MenuItem label={app.t.noRecent} disabled />
		{/if}
	</MenuSubmenu>
	<MenuSubmenu label={app.t.examples}>
		{#each EXAMPLES as ex (ex.file)}
			<MenuItem label={ex.label} onclick={() => app.openExample(ex)} />
		{/each}
	</MenuSubmenu>
</MenuSubmenu>
<MenuItem
	label={app.status.editing_component ? app.t.saveComponent : app.t.save}
	shortcut="Ctrl+S"
	onclick={app.saveFile}
/>
<MenuItem label={app.t.importLibrary} onclick={app.importLibrary} />
<MenuItem label={app.t.export} onclick={() => app.openExport('svg')} />
<MenuSubmenu label={app.t.share}>
	<MenuItem label={app.t.shareLink} onclick={() => void app.openShareLink()} />
	<MenuItem label={app.t.shareFcd} onclick={app.openShareFcd} />
</MenuSubmenu>
<MenuSeparator />
<MenuItem label={app.t.print} onclick={() => app.openExport('print')} />
