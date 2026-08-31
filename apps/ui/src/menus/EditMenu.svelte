<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	let { onDone }: { onDone?: () => void } = $props();

	const app = getAppSession();
	const hasSelection = $derived(app.status.selected > 0);

	function run(fn: () => void) {
		fn();
		onDone?.();
	}
</script>

<MenuItem
	label={app.t.cut}
	shortcut="Ctrl+X"
	disabled={!hasSelection}
	onclick={() => run(() => void app.cutFcd())}
/>
<MenuItem
	label={app.t.copy}
	shortcut="Ctrl+C"
	disabled={!hasSelection}
	onclick={() => run(() => void app.copyFcd())}
/>
<MenuItem label={app.t.paste} shortcut="Ctrl+V" onclick={() => run(() => void app.pasteFcd())} />
<MenuItem label={app.t.pasteNewDoc} onclick={() => run(() => void app.pasteNewDoc())} />
<MenuItem
	label={app.t.delete}
	shortcut="Del"
	disabled={!hasSelection}
	onclick={() => run(app.doDelete)}
/>
<hr />
<MenuItem
	label={app.t.undo}
	shortcut="Ctrl+Z"
	disabled={!app.status.can_undo}
	onclick={() => run(app.doUndo)}
/>
<MenuItem
	label={app.t.redo}
	shortcut="Ctrl+Y"
	disabled={!app.status.can_redo}
	onclick={() => run(app.doRedo)}
/>
<hr />
<MenuItem
	label={app.t.rotate}
	shortcut="R"
	disabled={!hasSelection}
	onclick={() => run(app.doRotate)}
/>
<MenuItem
	label={app.t.mirror}
	shortcut="S"
	disabled={!hasSelection}
	onclick={() => run(app.doMirror)}
/>
<MenuItem label={app.t.splitMacro} disabled={!hasSelection} onclick={() => run(app.doSplit)} />
<hr />
<MenuItem label={app.t.selectAll} onclick={() => run(app.doSelectAll)} />
<MenuItem label={app.t.invertSelection} onclick={() => run(app.doInvert)} />

<style>
	hr {
		border: none;
		border-top: 1px solid var(--border);
		margin: 4px 2px;
		width: 100%;
	}
</style>
