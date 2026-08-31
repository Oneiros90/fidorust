<script lang="ts">
	import { onMount } from 'svelte';
	import { AppSession } from './app/appSession.svelte';
	import { setAppSession } from './app/appContext';
	import CanvasHost from './canvas/CanvasHost.svelte';
	import AppOverlays from './chrome/AppOverlays.svelte';
	import MenuBar from './chrome/MenuBar.svelte';
	import StatusBar from './chrome/StatusBar.svelte';
	import ToolSidebar from './chrome/ToolSidebar.svelte';
	import LibraryPanel from './library/LibraryPanel.svelte';

	const app = new AppSession();
	setAppSession(app);

	onMount(() => {
		void app.init();
	});

	$effect(() => {
		document.documentElement.lang = app.locale;
		app.engine?.set_locale(app.locale);
	});
</script>

<svelte:window onkeydown={app.onKey} />
<svelte:document
	ondragover={app.onDragOver}
	ondrop={app.onDropFile}
	oncontextmenu={(e) => e.preventDefault()}
/>

<div class="shell">
	<MenuBar />
	<div class="body">
		<ToolSidebar />
		<CanvasHost />
		<LibraryPanel />
	</div>
	<StatusBar />
</div>
<AppOverlays />

<style>
	.shell {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	.body {
		flex: 1;
		display: flex;
		min-height: 0;
	}
</style>
