<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { displayLayerName } from '../app/layerOps';
	import { copyRgba } from '../lib/color';
	import type { ExportPreviewOpts } from '../lib/exportOptions';
	import LayerColorPicker from '../layers/LayerColorPicker.svelte';
	import VisibilityIcon from '../chrome/VisibilityIcon.svelte';

	let { opts = $bindable() }: { opts: ExportPreviewOpts } = $props();

	const app = getAppSession();

	function setLayerColor(i: number, color: [number, number, number, number]) {
		const layer = opts.layers[i];
		if (layer) layer.color = copyRgba(color);
	}
</script>

<div class="layers">
	<div class="layers-head">{app.t.exportLayers}</div>
	<ul>
		<li>
			<div class="export-picker">
				<LayerColorPicker
					color={opts.bgColor}
					label={app.t.exportBackground}
					onChange={(c) => (opts.bgColor = copyRgba(c))}
				/>
			</div>
			<span class="lname">{app.t.exportBackground}</span>
			<button
				type="button"
				class={['icon', { off: !opts.bgEnabled }]}
				title={opts.bgEnabled ? app.t.hideLayer : app.t.showLayer}
				aria-label={opts.bgEnabled ? app.t.hideLayer : app.t.showLayer}
				aria-pressed={opts.bgEnabled}
				onclick={() => (opts.bgEnabled = !opts.bgEnabled)}
			>
				<VisibilityIcon show={opts.bgEnabled} />
			</button>
		</li>
		{#each app.layers.layers as layer, i (i)}
			{#if opts.layers[i]}
				<li>
					<div class="export-picker">
						<LayerColorPicker
							color={opts.layers[i].color}
							label={app.t.exportLayerColor}
							onChange={(c) => setLayerColor(i, c)}
						/>
					</div>
					<span class="lname">{displayLayerName(layer.name, i, app.t)}</span>
					<button
						type="button"
						class={['icon', { off: !opts.layers[i].show }]}
						title={opts.layers[i].show ? app.t.hideLayer : app.t.showLayer}
						aria-label={opts.layers[i].show ? app.t.hideLayer : app.t.showLayer}
						aria-pressed={opts.layers[i].show}
						onclick={() => (opts.layers[i].show = !opts.layers[i].show)}
					>
						<VisibilityIcon show={opts.layers[i].show} />
					</button>
				</li>
			{/if}
		{/each}
	</ul>
</div>

<style>
	.export-picker {
		flex-shrink: 0;
		--cp-input-size: 22px;
		--cp-z: calc(var(--z-modal) + 1);
	}
	.layers {
		margin-top: 6px;
		border-top: 1px solid var(--border);
		padding-top: 8px;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.layers-head {
		font-size: 13px;
		font-weight: 650;
		margin-bottom: 6px;
	}
	.layers ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.layers li {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.layers ul:has(:global(.is-open)),
	.layers:has(:global(.is-open)) {
		overflow: visible;
		position: relative;
		z-index: calc(var(--z-modal) + 1);
	}
	.lname {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 12px;
	}
	.icon {
		width: 24px;
		height: 24px;
		padding: 0;
		flex-shrink: 0;
		display: grid;
		place-items: center;
		background: transparent;
		border-color: transparent;
		color: var(--fg-muted);
	}
	.icon :global(svg) {
		width: 14px;
		height: 14px;
		display: block;
	}
	.icon.off {
		opacity: 0.45;
	}
</style>
