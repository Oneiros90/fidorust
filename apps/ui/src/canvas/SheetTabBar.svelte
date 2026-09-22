<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { DRAG_THRESHOLD_PX } from '../lib/constants';
	import { focusAndSelectDeferred } from '../lib/focus';

	let { pane }: { pane: number } = $props();

	const app = getAppSession();

	let dragging = $state<number | null>(null);
	let over = $state<number | null>(null);
	let dragOrigin = $state<{ x: number; y: number; index: number } | null>(null);
	let skipRenameCommit = false;

	const sheets = $derived(app.status.sheets ?? []);
	const activeIndex = $derived(app.status.pane_sheets?.[pane] ?? 0);
	const editing = $derived(
		app.editingSheetName?.pane === pane ? app.editingSheetName.index : null
	);

	function commitRename(i: number, name: string) {
		if (skipRenameCommit) {
			skipRenameCommit = false;
			app.editingSheetName = null;
			return;
		}
		const trimmed = name.trim();
		if (trimmed) app.renameSheet(i, trimmed);
		else app.editingSheetName = null;
	}

	function cancelRename() {
		skipRenameCommit = true;
		app.editingSheetName = null;
	}

	function onTabPointerDown(e: PointerEvent, i: number) {
		if (e.button !== 0) return;
		if (editing === i) return;
		e.stopPropagation();
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		dragOrigin = { x: e.clientX, y: e.clientY, index: i };
		dragging = null;
		over = i;
	}

	function onTabPointerMove(e: PointerEvent) {
		if (!dragOrigin) return;
		if (dragging === null) {
			const dx = e.clientX - dragOrigin.x;
			const dy = e.clientY - dragOrigin.y;
			if (dx * dx + dy * dy < DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) return;
			dragging = dragOrigin.index;
		}
		const el = document.elementFromPoint(e.clientX, e.clientY);
		const tab = el?.closest('[data-sheet-index]');
		if (tab instanceof HTMLElement) {
			over = Number(tab.dataset.sheetIndex);
		}
	}

	function onTabPointerUp(e: PointerEvent) {
		if (!dragOrigin) return;
		const from = dragOrigin.index;
		const dragged = dragging;
		const dest = over;
		dragOrigin = null;
		dragging = null;
		over = null;
		if (dragged !== null && dest !== null && dragged !== dest) {
			app.reorderSheets(dragged, dest);
			return;
		}
		if (e.button === 0) app.selectSheet(pane, from);
	}

	function onTabContextMenu(e: MouseEvent, i: number) {
		e.preventDefault();
		e.stopPropagation();
		if (dragging !== null) return;
		app.openSheetContextMenu(e.clientX, e.clientY, pane, i);
	}
</script>

<nav class="tabs" aria-label={app.t.sheets} onpointerenter={() => app.hoverPane(pane)}>
	{#each sheets as name, i (name + ':' + i)}
		<button
			type="button"
			class={[
				'tab',
				{
					active: activeIndex === i,
					over: over === i && dragging !== null && dragging !== i,
					dragging: dragging === i
				}
			]}
			data-sheet-index={i}
			onpointerdown={(e) => onTabPointerDown(e, i)}
			onpointermove={onTabPointerMove}
			onpointerup={onTabPointerUp}
			onpointercancel={onTabPointerUp}
			ondblclick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				app.beginRenameSheet(pane, i);
			}}
			oncontextmenu={(e) => onTabContextMenu(e, i)}
		>
			{#if editing === i}
				<input
					class="rename"
					value={name}
					{@attach focusAndSelectDeferred}
					onblur={(e) => commitRename(i, e.currentTarget.value)}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							commitRename(i, e.currentTarget.value);
						} else if (e.key === 'Escape') {
							e.preventDefault();
							cancelRename();
						}
					}}
					onclick={(e) => e.stopPropagation()}
					onpointerdown={(e) => e.stopPropagation()}
				/>
			{:else}
				<span class="label">{name}</span>
			{/if}
		</button>
	{/each}
	<button
		type="button"
		class="add"
		title={app.t.addSheet}
		aria-label={app.t.addSheet}
		onclick={() => app.addSheet(pane)}
	>
		+
	</button>
</nav>

<style>
	.tabs {
		display: flex;
		flex-direction: row;
		align-items: stretch;
		min-width: 0;
		overflow-x: auto;
		overflow-y: hidden;
		border-top: 1px solid var(--border);
		background: var(--bg);
		flex: 0 0 auto;
		height: 26px;
	}
	.tab {
		flex: 0 0 auto;
		display: flex;
		align-items: center;
		max-width: 160px;
		padding: 0 10px;
		font: inherit;
		font-size: 12px;
		line-height: 1;
		color: var(--fg-muted);
		cursor: pointer;
		user-select: none;
		border: 0;
		border-bottom: 2px solid transparent;
		border-radius: 0;
		background: transparent;
	}
	.tab:hover {
		color: var(--fg);
	}
	.tab.active {
		color: var(--fg);
		border-bottom-color: var(--accent);
		font-weight: 600;
	}
	.tab.over {
		box-shadow: inset 2px 0 0 var(--accent);
	}
	.tab.dragging {
		opacity: 0.45;
	}
	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rename {
		width: 110px;
		min-width: 0;
		border: 0;
		padding: 1px 2px;
		font: inherit;
		font-size: 12px;
		color: inherit;
		background: var(--bg-panel);
		outline: 1px solid var(--accent);
	}
	.add {
		flex: 0 0 auto;
		border: 0;
		background: transparent;
		color: var(--fg-muted);
		font: inherit;
		font-size: 16px;
		line-height: 1;
		padding: 0 8px;
		cursor: pointer;
	}
	.add:hover {
		color: var(--accent);
	}
</style>
