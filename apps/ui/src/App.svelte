<script lang="ts">
	import { onMount } from 'svelte';
	import { AppSession } from './app/appSession.svelte';
	import { setAppSession } from './app/appContext';
	import SheetPane from './canvas/SheetPane.svelte';
	import AppOverlays from './chrome/AppOverlays.svelte';
	import MenuBar from './chrome/MenuBar.svelte';
	import StatusBar from './chrome/StatusBar.svelte';
	import ToolSidebar from './chrome/ToolSidebar.svelte';
	import RightSidebar from './chrome/RightSidebar.svelte';
	import { syncDesktopTitle } from './lib/desktopFiles';

	const app = new AppSession();
	setAppSession(app);

	let stage: HTMLDivElement | undefined;

	onMount(() => {
		void app.init();
	});

	$effect(() => {
		const title = app.windowTitle;
		document.title = title;
		void syncDesktopTitle(title);
	});

	function bindStage(node: HTMLDivElement) {
		stage = node;
		return () => {
			if (stage === node) stage = undefined;
		};
	}

	function onSplitterDown(e: PointerEvent) {
		if (!stage) return;
		e.preventDefault();
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		const rect = stage.getBoundingClientRect();
		const move = (ev: PointerEvent) => {
			if (rect.width <= 0) return;
			app.splitRatio = Math.min(0.8, Math.max(0.2, (ev.clientX - rect.left) / rect.width));
			app.schedulePersist();
		};
		const up = () => {
			(e.currentTarget as HTMLElement).removeEventListener('pointermove', move);
			(e.currentTarget as HTMLElement).removeEventListener('pointerup', up);
		};
		(e.currentTarget as HTMLElement).addEventListener('pointermove', move);
		(e.currentTarget as HTMLElement).addEventListener('pointerup', up);
	}
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
		<div class="stage" {@attach bindStage}>
			<div class="pane-slot" style:flex={`${app.status.split ? app.splitRatio : 1} 1 0`}>
				<SheetPane pane={0} />
			</div>
			{#if app.status.split}
				<div
					class="splitter"
					role="separator"
					aria-orientation="vertical"
					aria-label={app.t.dualSheet}
					onpointerdown={onSplitterDown}
				></div>
				<div class="pane-slot" style:flex={`${1 - app.splitRatio} 1 0`}>
					<SheetPane pane={1} />
				</div>
			{/if}
		</div>
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
	.stage {
		flex: 1;
		display: flex;
		min-width: 0;
		min-height: 0;
	}
	.pane-slot {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.splitter {
		flex: 0 0 6px;
		cursor: col-resize;
		background: var(--border);
	}
	.splitter:hover {
		background: var(--accent);
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
