<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { commitLayerName, displayLayerName } from '../app/layerOps';
	import { focusAndSelectDeferred } from '../lib/focus';
	import VisibilityIcon from '../chrome/VisibilityIcon.svelte';
	import LayerColorPicker from './LayerColorPicker.svelte';

	const app = getAppSession();

	let dragging = $state<number | null>(null);
	let over = $state<number | null>(null);
	let skipRenameCommit = false;

	function commitRename(i: number, name: string) {
		if (skipRenameCommit) {
			skipRenameCommit = false;
			app.editingLayerName = null;
			return;
		}
		app.setLayerName(i, commitLayerName(name, app.layers.layers[i]?.name ?? '', i, app.t));
	}

	function cancelRename() {
		skipRenameCommit = true;
		app.editingLayerName = null;
	}

	function onHandlePointerDown(e: PointerEvent, i: number) {
		e.stopPropagation();
		e.preventDefault();
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		dragging = i;
		over = i;
	}

	function onHandlePointerMove(e: PointerEvent) {
		if (dragging === null) return;
		const el = document.elementFromPoint(e.clientX, e.clientY);
		const row = el?.closest('[data-layer-index]');
		if (row instanceof HTMLElement) {
			over = Number(row.dataset.layerIndex);
		}
	}

	function onHandlePointerUp() {
		if (dragging !== null && over !== null && dragging !== over) {
			app.reorderLayer(dragging, over);
		}
		dragging = null;
		over = null;
	}

	function onRowContextMenu(e: MouseEvent, i: number) {
		e.preventDefault();
		e.stopPropagation();
		if (dragging !== null) return;
		app.openLayerContextMenu(e.clientX, e.clientY, i);
	}
</script>

<div class="layers">
	<ul>
		{#each app.layers.layers as l, i (i)}
			<li
				class={[
					'row',
					{
						current: app.status.layer === i,
						over: over === i && dragging !== null && dragging !== i,
						dragging: dragging === i
					}
				]}
				data-layer-index={i}
				oncontextmenu={(e) => onRowContextMenu(e, i)}
			>
				<button
					type="button"
					class="handle"
					title={app.t.reorderLayer}
					aria-label={app.t.reorderLayer}
					onpointerdown={(e) => onHandlePointerDown(e, i)}
					onpointermove={onHandlePointerMove}
					onpointerup={onHandlePointerUp}
					onpointercancel={onHandlePointerUp}
				>
					<svg viewBox="0 0 16 16" aria-hidden="true">
						<circle cx="6" cy="4" r="1.2" fill="currentColor" />
						<circle cx="10" cy="4" r="1.2" fill="currentColor" />
						<circle cx="6" cy="8" r="1.2" fill="currentColor" />
						<circle cx="10" cy="8" r="1.2" fill="currentColor" />
						<circle cx="6" cy="12" r="1.2" fill="currentColor" />
						<circle cx="10" cy="12" r="1.2" fill="currentColor" />
					</svg>
				</button>
				<LayerColorPicker index={i} color={l.color} label={app.t.layer} />
				{#if app.editingLayerName === i}
					<input
						class="name"
						value={displayLayerName(l.name, i, app.t)}
						{@attach focusAndSelectDeferred}
						onblur={(e) => commitRename(i, e.currentTarget.value)}
						onkeydown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault();
								e.currentTarget.blur();
							}
							if (e.key === 'Escape') {
								e.preventDefault();
								cancelRename();
							}
						}}
					/>
				{:else}
					<button
						type="button"
						class="name"
						onclick={() => app.setLayer(i)}
						ondblclick={() => app.beginRenameLayer(i)}
					>
						{displayLayerName(l.name, i, app.t)}
					</button>
				{/if}
				<button
					type="button"
					class={['icon', { off: !l.show }]}
					title={l.show ? app.t.hideLayer : app.t.showLayer}
					aria-label={l.show ? app.t.hideLayer : app.t.showLayer}
					aria-pressed={l.show}
					onclick={() => app.setLayerShow(i, !l.show)}
				>
					<VisibilityIcon show={l.show} />
				</button>
				<button
					type="button"
					class="icon danger"
					title={app.t.deleteLayer}
					aria-label={app.t.deleteLayer}
					disabled={app.layers.layers.length <= 1}
					onclick={() => app.requestDeleteLayer(i)}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path
							d="M4 7h16M9 7V5h6v2m-7 0l.7 12h6.6l.7-12"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
				</button>
			</li>
		{/each}
	</ul>
	<button type="button" class="add" onclick={app.addLayer}>{app.t.addLayer}</button>
</div>

<style>
	.layers {
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 6px;
		overflow: auto;
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 3px 4px;
		border-radius: 6px;
	}
	.row.current {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
	}
	.row.over {
		outline: 1px dashed var(--accent);
	}
	.row.dragging {
		opacity: 0.55;
	}
	.handle,
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
	.handle {
		cursor: grab;
	}
	.handle:active {
		cursor: grabbing;
	}
	.handle svg,
	.icon svg {
		width: 14px;
		height: 14px;
		display: block;
	}
	.icon.off {
		opacity: 0.45;
	}
	.icon.danger:hover:not(:disabled) {
		color: var(--danger);
		border-color: var(--danger);
	}
	ul:has(:global(.is-open)) {
		overflow: visible;
		position: relative;
		z-index: var(--z-flyout);
	}
	.row:has(:global(.is-open)) {
		position: relative;
		z-index: 2;
	}
	.name {
		flex: 1;
		min-width: 0;
		padding: 2px 4px;
		border-color: transparent;
		background: transparent;
		cursor: pointer;
		text-align: left;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font: inherit;
		font-size: 12px;
		border-radius: 4px;
	}
	.row.current .name {
		font-weight: 650;
	}
	input.name:focus {
		border-color: var(--border);
		background: var(--bg-menu);
		cursor: text;
		text-overflow: clip;
	}
	.add {
		margin: 0 8px 10px;
		flex-shrink: 0;
	}
</style>
