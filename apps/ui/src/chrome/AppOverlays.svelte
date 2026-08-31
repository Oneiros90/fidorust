<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import AboutDialog from '../dialogs/AboutDialog.svelte';
	import ConfirmDialog from '../dialogs/ConfirmDialog.svelte';
	import ErrorDialog from '../dialogs/ErrorDialog.svelte';
	import GridDialog from '../dialogs/GridDialog.svelte';
	import LayersDialog from '../dialogs/LayersDialog.svelte';
	import ShareFcdDialog from '../dialogs/ShareFcdDialog.svelte';
	import ShareLinkDialog from '../dialogs/ShareLinkDialog.svelte';
	import MacroGhost from '../library/MacroGhost.svelte';
	import ContextMenu from '../menus/ContextMenu.svelte';
	import EditMenu from '../menus/EditMenu.svelte';

	const app = getAppSession();

	function bindFilePicker(node: HTMLInputElement) {
		app.filePicker = node;
		return () => {
			if (app.filePicker === node) app.filePicker = undefined;
		};
	}
</script>

<input
	{@attach bindFilePicker}
	type="file"
	accept=".fcd,.txt"
	hidden
	onchange={app.onPickedFile}
/>

{#if app.libGhost}
	<MacroGhost {...app.libGhost} />
{/if}

{#if app.ctxMenu}
	<ContextMenu
		x={app.ctxMenu.x}
		y={app.ctxMenu.y}
		onClose={() => (app.ctxMenu = null)}
	>
		<EditMenu onDone={() => (app.ctxMenu = null)} />
	</ContextMenu>
{/if}

{#if app.menu}
	<button type="button" class="scrim" onclick={app.closeMenu} aria-label="close menu"></button>
{/if}

{#if app.showLayers}
	<LayersDialog />
{/if}

{#if app.showAbout}
	<AboutDialog />
{/if}

{#if app.showGridDlg}
	<GridDialog
		t={app.t}
		gridX={app.status.grid}
		gridY={app.status.grid_y}
		snapX={app.status.snap}
		snapY={app.status.snap_y}
		showGrid={app.status.show_grid}
		snapEnable={app.status.snap_enable}
		hideMacroOrigin={app.status.hide_macro_origin}
		onApply={app.applyGrid}
		onCancel={() => (app.showGridDlg = false)}
	/>
{/if}

{#if app.error}
	<ErrorDialog />
{/if}

{#if app.showDiscardConfirm}
	<ConfirmDialog />
{/if}

{#if app.showShareLink}
	<ShareLinkDialog />
{/if}

{#if app.shareFcdText !== null}
	<ShareFcdDialog />
{/if}

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: transparent;
		border: none;
		z-index: 2;
	}
</style>
