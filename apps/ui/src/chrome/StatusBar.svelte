<script lang="ts">
	import { onMount } from 'svelte';
	import { getAppSession } from '../app/appContext';
	import {
		appVersion,
		checkDesktopUpdate,
		displayVersion,
		isDesktopApp,
		openReleasePage,
		type LatestRelease
	} from '../lib/updates';
	import StatusSettingPill from './StatusSettingPill.svelte';

	const app = getAppSession();

	let update = $state<LatestRelease | null>(null);

	const updateLabel = $derived(update ? app.t.updateTo.replace('{version}', update.tag) : '');

	onMount(() => {
		void (async () => {
			if (!(await isDesktopApp())) return;
			update = await checkDesktopUpdate(appVersion);
		})();
	});

	const openUpdate = () => {
		const url = update?.url;
		if (url) void openReleasePage(url);
	};
</script>

<footer class="status">
	<span>{app.status.title || app.t.statusReady}</span>
	<span>{app.t.layer} {app.status.layer}</span>
	<span>{app.status.x}, {app.status.y} LU</span>
	{#if app.status.pcb}
		<span>{app.status.xmm.toFixed(2)} × {app.status.ymm.toFixed(2)} mm</span>
	{/if}
	<span>{Math.round(app.status.zoom * 100)}%</span>
	<span>{app.status.n} obj</span>
	{#if app.status.selected}
		<span>sel {app.status.selected}</span>
	{:else if app.status.hover_op}
		<span>{app.status.hover_op}</span>
	{/if}
	{#if app.status.pending_component}<span>{app.t.component}: {app.status.pending_component}</span
		>{/if}
	<span class="pills">
		<StatusSettingPill
			label={app.t.statusSnap}
			x={app.status.snap}
			y={app.status.snap_y}
			enabled={app.status.snap_enable}
			enabledLabel={app.t.enableSnap}
			offLabel={app.t.statusOff}
			min={1}
			max={20}
			onX={(v) => app.setSnap(v, app.status.snap_y)}
			onY={(v) => app.setSnap(app.status.snap, v)}
			onEnabled={app.setSnapEnable}
		/>
		<StatusSettingPill
			label={app.t.statusGrid}
			x={app.status.grid}
			y={app.status.grid_y}
			enabled={app.status.show_grid}
			enabledLabel={app.t.showGrid}
			offLabel={app.t.statusOff}
			min={1}
			max={40}
			onX={(v) => app.setGrid(v, app.status.grid_y)}
			onY={(v) => app.setGrid(app.status.grid, v)}
			onEnabled={app.setShowGrid}
		/>
	</span>
	<span class="version">
		<span>{displayVersion(appVersion)}</span>
		{#if update}
			<button type="button" class="update-pill" onclick={openUpdate}>{updateLabel}</button>
		{/if}
	</span>
</footer>

<style>
	.status {
		display: flex;
		flex-wrap: nowrap;
		align-items: center;
		gap: 16px;
		padding: 4px 12px;
		font-size: 12px;
		font-family: var(--mono);
		background: var(--bg-menu);
		border-top: 1px solid var(--border);
		color: var(--fg-muted);
		white-space: nowrap;
		overflow: visible;
		position: relative;
		z-index: var(--z-menubar);
	}
	.status > * {
		flex-shrink: 0;
	}
	.pills {
		display: inline-flex;
		flex-wrap: nowrap;
		align-items: center;
		gap: 6px;
	}
	@media (hover: none), (max-width: 720px) {
		.status {
			overflow-x: auto;
			overflow-y: hidden;
			scrollbar-width: none;
			overscroll-behavior-x: contain;
		}
		.status::-webkit-scrollbar {
			display: none;
		}
	}
	.version {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
		white-space: nowrap;
	}
	.update-pill {
		font: inherit;
		font-size: inherit;
		line-height: 1.4;
		padding: 1px 8px;
		border: none;
		border-radius: 999px;
		background: var(--accent);
		color: var(--accent-fg);
		cursor: pointer;
	}
	.update-pill:hover {
		filter: brightness(1.08);
	}
	.update-pill:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}
</style>
