<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import LayersPanel from '../layers/LayersPanel.svelte';
	import LibraryPanel from '../library/LibraryPanel.svelte';

	const app = getAppSession();
	let toggleLabel = $derived(app.rightCollapsed ? app.t.showLibrary : app.t.hideLibrary);
	let panelLabel = $derived(
		app.rightTab === 'layers' ? app.t.layersTab : app.t.libraries
	);
	const showLayers = $derived(app.rightTab === 'layers');
</script>

<aside class={['libs', { collapsed: app.rightCollapsed }]} aria-label={panelLabel}>
	<div class="head">
		<button
			type="button"
			class="collapse-btn"
			onclick={() => (app.rightCollapsed = !app.rightCollapsed)}
			aria-expanded={!app.rightCollapsed}
			aria-controls="right-panel-body"
			title={toggleLabel}
			aria-label={toggleLabel}
		>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				{#if app.rightCollapsed}
					<polyline points="15 6 9 12 15 18" />
				{:else}
					<polyline points="9 6 15 12 9 18" />
				{/if}
			</svg>
		</button>
		<div class="tabs" role="tablist" aria-label={panelLabel}>
			<button
				type="button"
				role="tab"
				id="tab-layers"
				aria-selected={app.rightTab === 'layers'}
				aria-controls="right-panel-body"
				tabindex={app.rightTab === 'layers' ? 0 : -1}
				onclick={() => (app.rightTab = 'layers')}
			>
				{app.t.layersTab}
			</button>
			<button
				type="button"
				role="tab"
				id="tab-library"
				aria-selected={app.rightTab === 'library'}
				aria-controls="right-panel-body"
				tabindex={app.rightTab === 'library' ? 0 : -1}
				onclick={() => (app.rightTab = 'library')}
			>
				{app.t.libraries}
			</button>
		</div>
	</div>
	<div class="body" id="right-panel-body" inert={app.rightCollapsed} role="tabpanel">
		{#if showLayers}
			<LayersPanel />
		{:else}
			<LibraryPanel />
		{/if}
	</div>
</aside>

<style>
	.libs {
		position: relative;
		width: 300px;
		border-left: 1px solid var(--border);
		background: var(--bg-panel);
		display: flex;
		flex-direction: column;
		min-height: 0;
		font-size: 12px;
		line-height: 1.35;
		font-family: var(--font);
	}
	.libs.collapsed {
		width: 0;
		border: none;
		background: transparent;
		overflow: visible;
		pointer-events: none;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px 6px 8px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}
	.libs.collapsed .head {
		position: absolute;
		top: 8px;
		right: 100%;
		margin-right: 8px;
		padding: 0;
		border: none;
		pointer-events: auto;
		z-index: calc(var(--z-menubar) + 1);
	}
	.collapse-btn {
		width: 26px;
		height: 26px;
		padding: 0;
		flex-shrink: 0;
		display: grid;
		place-items: center;
		background: var(--bg-menu);
	}
	.libs.collapsed .collapse-btn {
		box-shadow: var(--shadow);
	}
	.collapse-btn svg {
		width: 14px;
		height: 14px;
		display: block;
	}
	.tabs {
		display: flex;
		gap: 3px;
		min-width: 0;
		flex: 1;
		padding: 2px;
		background: color-mix(in srgb, var(--fg) 8%, transparent);
		border: 1px solid var(--border);
		border-radius: 8px;
	}
	.tabs button {
		flex: 1;
		padding: 4px 6px;
		font-size: 12px;
		font-weight: 600;
		color: var(--fg-muted);
		background: transparent;
		border-color: transparent;
	}
	.tabs button:hover {
		color: var(--fg);
		background: color-mix(in srgb, var(--bg-menu) 85%, transparent);
	}
	.tabs button[aria-selected='true'] {
		color: var(--fg);
		background: var(--bg-menu);
		border-color: var(--border);
		font-weight: 650;
	}
	.libs.collapsed .tabs,
	.libs.collapsed .body {
		display: none;
	}
	.body {
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
		font: inherit;
	}
</style>
