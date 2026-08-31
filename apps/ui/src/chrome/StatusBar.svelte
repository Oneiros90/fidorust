<script lang="ts">
	import { getAppSession } from '../app/appContext';

	const app = getAppSession();
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
</footer>

<style>
	.status {
		display: flex;
		gap: 16px;
		padding: 4px 12px;
		font-size: 12px;
		font-family: var(--mono);
		background: var(--bg-menu);
		border-top: 1px solid var(--border);
		color: var(--fg-muted);
	}
</style>
