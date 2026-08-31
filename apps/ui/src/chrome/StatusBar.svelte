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
	{#if app.status.selected}<span>sel {app.status.selected}</span>{/if}
	{#if app.status.pending_macro}<span>{app.t.macro}: {app.status.pending_macro}</span>{/if}
	<span
		>snap {app.status.snap}×{app.status.snap_y}{app.status.snap_enable ? '' : ' off'} / grid {app
			.status.grid}×{app.status.grid_y}</span
	>
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
		align-items: center;
		gap: 16px;
		padding: 4px 12px;
		font-size: 12px;
		font-family: var(--mono);
		background: var(--bg-menu);
		border-top: 1px solid var(--border);
		color: var(--fg-muted);
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
