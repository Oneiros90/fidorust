<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { EXAMPLES } from '../lib/examples';
	import MenuItem from './MenuItem.svelte';
	import MenuSubmenu from './MenuSubmenu.svelte';

	const app = getAppSession();

	function run(fn: () => void) {
		fn();
		app.closeMenu();
	}
</script>

<MenuItem label={app.t.new} onclick={() => run(app.requestNewDoc)} />
<hr />
<MenuSubmenu label={app.t.open}>
	<MenuItem label={app.t.openFromFile} shortcut="Ctrl+O" onclick={() => run(app.openFile)} />
	<MenuItem
		label={app.t.openFromClipboard}
		onclick={() => run(() => void app.pasteNewDoc())}
	/>
	<MenuSubmenu label={app.t.recent}>
		{#each app.recents as r (r.name)}
			<MenuItem label={r.name} onclick={() => run(() => app.openRecent(r))} />
		{/each}
		{#if app.recents.length === 0}
			<MenuItem label={app.t.noRecent} disabled />
		{/if}
	</MenuSubmenu>
	<MenuSubmenu label={app.t.examples}>
		{#each EXAMPLES as ex (ex.file)}
			<MenuItem label={ex.label} onclick={() => run(() => void app.openExample(ex.file))} />
		{/each}
	</MenuSubmenu>
</MenuSubmenu>
<MenuSubmenu label={app.t.save}>
	<MenuItem label={app.t.saveFcd} shortcut="Ctrl+S" onclick={() => run(app.saveFile)} />
	<MenuItem label={app.t.saveSvg} onclick={() => run(app.exportSvg)} />
	<MenuItem label={app.t.savePng} onclick={() => run(app.exportPng)} />
	<MenuItem label={app.t.savePdf} onclick={() => run(app.exportPdf)} />
</MenuSubmenu>
<MenuSubmenu label={app.t.share}>
	<MenuItem label={app.t.shareLink} onclick={() => run(() => void app.openShareLink())} />
	<MenuItem label={app.t.shareFcd} onclick={() => run(app.openShareFcd)} />
</MenuSubmenu>
<hr />
<MenuItem label={app.t.print} onclick={() => run(app.printDoc)} />

<style>
	hr {
		border: none;
		border-top: 1px solid var(--border);
		margin: 4px 2px;
		width: 100%;
	}
</style>
