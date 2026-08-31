<script lang="ts">
	import type { Dict } from './i18n';

	let {
		t,
		hasSelection,
		canUndo,
		canRedo,
		onCut,
		onCopy,
		onPaste,
		onPasteNew,
		onDelete,
		onUndo,
		onRedo,
		onRotate,
		onMirror,
		onSplit,
		onSelectAll,
		onInvert,
		onDone
	}: {
		t: Dict;
		hasSelection: boolean;
		canUndo: boolean;
		canRedo: boolean;
		onCut: () => void;
		onCopy: () => void;
		onPaste: () => void;
		onPasteNew: () => void;
		onDelete: () => void;
		onUndo: () => void;
		onRedo: () => void;
		onRotate: () => void;
		onMirror: () => void;
		onSplit: () => void;
		onSelectAll: () => void;
		onInvert: () => void;
		onDone?: () => void;
	} = $props();

	function run(fn: () => void) {
		fn();
		onDone?.();
	}
</script>

<button type="button" disabled={!hasSelection} onclick={() => run(onCut)}>
	{t.cut}<span class="acc">Ctrl+X</span>
</button>
<button type="button" disabled={!hasSelection} onclick={() => run(onCopy)}>
	{t.copy}<span class="acc">Ctrl+C</span>
</button>
<button type="button" onclick={() => run(onPaste)}>
	{t.paste}<span class="acc">Ctrl+V</span>
</button>
<button type="button" onclick={() => run(onPasteNew)}>{t.pasteNewDoc}</button>
<button type="button" disabled={!hasSelection} onclick={() => run(onDelete)}>
	{t.delete}<span class="acc">Del</span>
</button>
<hr />
<button type="button" disabled={!canUndo} onclick={() => run(onUndo)}>
	{t.undo}<span class="acc">Ctrl+Z</span>
</button>
<button type="button" disabled={!canRedo} onclick={() => run(onRedo)}>
	{t.redo}<span class="acc">Ctrl+Y</span>
</button>
<hr />
<button type="button" disabled={!hasSelection} onclick={() => run(onRotate)}>
	{t.rotate}<span class="acc">R</span>
</button>
<button type="button" disabled={!hasSelection} onclick={() => run(onMirror)}>
	{t.mirror}<span class="acc">S</span>
</button>
<button type="button" disabled={!hasSelection} onclick={() => run(onSplit)}>{t.splitMacro}</button>
<hr />
<button type="button" onclick={() => run(onSelectAll)}>{t.selectAll}</button>
<button type="button" onclick={() => run(onInvert)}>{t.invertSelection}</button>

<style>
	button {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		width: 100%;
		text-align: left;
		border: none;
		background: transparent;
		padding: 6px 8px;
		border-radius: 4px;
		font: inherit;
		font-size: 13px;
		color: inherit;
	}
	button:hover:not(:disabled) {
		background: var(--bg-panel);
	}
	button:disabled {
		opacity: 0.45;
	}
	.acc {
		color: var(--fg-muted);
		font-size: 11px;
		font-family: var(--mono);
	}
	hr {
		border: none;
		border-top: 1px solid var(--border);
		margin: 4px 2px;
		width: 100%;
	}
</style>
