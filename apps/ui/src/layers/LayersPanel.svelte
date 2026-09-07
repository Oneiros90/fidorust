<script lang="ts">
	import { getAppSession } from '../app/appContext';

	const app = getAppSession();

	let dragging = $state<number | null>(null);
	let over = $state<number | null>(null);
	let skipRenameCommit = false;

	function hex(color: number[]) {
		return '#' + color.map((c) => c.toString(16).padStart(2, '0')).join('');
	}

	function parseHex(value: string): [number, number, number] {
		return [
			parseInt(value.slice(1, 3), 16),
			parseInt(value.slice(3, 5), 16),
			parseInt(value.slice(5, 7), 16)
		];
	}

	function focusAndSelect(node: HTMLInputElement) {
		queueMicrotask(() => {
			node.focus();
			node.select();
		});
	}

	function commitRename(i: number, name: string) {
		if (skipRenameCommit) {
			skipRenameCommit = false;
			app.editingLayerName = null;
			return;
		}
		app.setLayerName(i, name);
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
				<input
					type="color"
					value={hex(l.color)}
					title={app.t.layer}
					aria-label={app.t.layer}
					onclick={() => app.setLayer(i)}
					onchange={(e) => {
						const [r, g, b] = parseHex(e.currentTarget.value);
						app.setLayerColor(i, r, g, b);
					}}
				/>
				{#if app.editingLayerName === i}
					<input
						class="name"
						value={l.name}
						{@attach focusAndSelect}
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
						{l.name}
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
					{#if l.show}
						<svg viewBox="0 0 24 24" aria-hidden="true">
							<path
								d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7z"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							/>
							<circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2" />
						</svg>
					{:else}
						<svg viewBox="0 0 24 24" aria-hidden="true">
							<path
								d="M3 3l18 18M10.6 10.6A3 3 0 0 0 12 15a3 3 0 0 0 2.4-1.2M9.9 5.1A11 11 0 0 1 12 5c6 0 10 7 10 7a18 18 0 0 1-3.2 3.8M6.1 6.1C3.7 8 2 12 2 12s4 7 10 7a10 10 0 0 0 4-.8"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
							/>
						</svg>
					{/if}
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
		padding: 2px 4px;
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
	input[type='color'] {
		width: 22px;
		height: 22px;
		padding: 0;
		border: 1px solid var(--border);
		background: transparent;
		flex-shrink: 0;
		cursor: pointer;
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
