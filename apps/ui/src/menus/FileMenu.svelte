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
			<MenuItem label={ex.label} onclick={() => void app.openExample(ex.file)} />
		{/each}
	</MenuSubmenu>
</MenuSubmenu>
<MenuSubmenu label={app.t.save}>
	<MenuItem label={app.t.saveFcd} shortcut="Ctrl+S" onclick={app.saveFile} />
	<MenuItem label={app.t.saveSvg} onclick={app.exportSvg} />
	<MenuItem label={app.t.savePng} onclick={app.exportPng} />
	<MenuItem label={app.t.savePdf} onclick={app.exportPdf} />
</MenuSubmenu>
<MenuSubmenu label={app.t.share}>
	<MenuItem label={app.t.shareLink} onclick={() => void app.openShareLink()} />
	<MenuItem label={app.t.shareFcd} onclick={app.openShareFcd} />
</MenuSubmenu>
<MenuSeparator />
<MenuItem label={app.t.print} onclick={app.printDoc} />
