<script lang="ts">
	import { onMount } from 'svelte';
	import { AppSession } from './app/appSession.svelte';
	import { setAppSession } from './app/appContext';
	import CanvasHost from './canvas/CanvasHost.svelte';
	import AppOverlays from './chrome/AppOverlays.svelte';
	import MenuBar from './chrome/MenuBar.svelte';
	import StatusBar from './chrome/StatusBar.svelte';
	import ToolSidebar from './chrome/ToolSidebar.svelte';
	import RightSidebar from './chrome/RightSidebar.svelte';
	import { syncDesktopTitle } from './lib/desktopFiles';

	const app = new AppSession();
	setAppSession(app);

	onMount(() => {
		void app.init();
	});

	$effect(() => {
		const title = app.windowTitle;
		document.title = title;
		void syncDesktopTitle(title);
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
	{#if app.status.editing_component}
		<div class="prefab-bar">
			<button type="button" class="primary" onclick={app.saveComponentEdit}
				>{app.t.saveComponent}</button
			>
			<button type="button" onclick={app.cancelComponentEdit}>{app.t.cancelComponentEdit}</button>
			<span
				>{app.t.editingComponent}: {app.status.editing_component_name ??
					app.status.editing_component}</span
			>
		</div>
	{/if}
	<div class="body">
		<ToolSidebar />
		<CanvasHost />
		<RightSidebar />
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
	.prefab-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		background: color-mix(in srgb, var(--accent) 16%, var(--bg-menu));
		border-bottom: 1px solid var(--border);
		font-size: 13px;
	}
	.prefab-bar span {
		margin-left: 4px;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
