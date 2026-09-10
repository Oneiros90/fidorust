<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import AboutDialog from '../dialogs/AboutDialog.svelte';
	import TechnologiesDialog from '../dialogs/TechnologiesDialog.svelte';
	import ConfirmDialog from '../dialogs/ConfirmDialog.svelte';
	import ErrorDialog from '../dialogs/ErrorDialog.svelte';
	import ProjectSettingsDialog from '../dialogs/ProjectSettingsDialog.svelte';
	import DeleteLayerDialog from '../dialogs/DeleteLayerDialog.svelte';
	import DeleteComponentDialog from '../dialogs/DeleteComponentDialog.svelte';
	import DeleteLibraryDialog from '../dialogs/DeleteLibraryDialog.svelte';
	import SaveLocalComponentsDialog from '../dialogs/SaveLocalComponentsDialog.svelte';
	import PropertiesDialog from '../dialogs/PropertiesDialog.svelte';
	import ShareFcdDialog from '../dialogs/ShareFcdDialog.svelte';
	import ShareLinkDialog from '../dialogs/ShareLinkDialog.svelte';
	import ExportPreviewDialog from '../dialogs/ExportPreviewDialog.svelte';
	import UnresolvedComponentsDialog from '../dialogs/UnresolvedComponentsDialog.svelte';
	import ComponentGhost from '../library/ComponentGhost.svelte';
	import LibraryItemContextMenu from '../library/LibraryItemContextMenu.svelte';
	import LibraryContextMenu from '../library/LibraryContextMenu.svelte';
	import ContextMenu from '../menus/ContextMenu.svelte';
	import EditMenu from '../menus/EditMenu.svelte';
	import LayerContextMenu from '../layers/LayerContextMenu.svelte';
	import Scrim from './Scrim.svelte';

	const app = getAppSession();
	const dialog = $derived(app.dialogs.dialog);

	function bindFilePicker(node: HTMLInputElement) {
		app.filePicker = node;
		return () => {
			if (app.filePicker === node) app.filePicker = undefined;
		};
	}
	function bindLibraryPicker(node: HTMLInputElement) {
		app.libraryPicker = node;
		return () => {
			if (app.libraryPicker === node) app.libraryPicker = undefined;
		};
	}
</script>

<input {@attach bindFilePicker} type="file" accept=".fcd,.txt" hidden onchange={app.onPickedFile} />
<input
	{@attach bindLibraryPicker}
	type="file"
	accept=".fcl,.FCL"
	multiple
	hidden
	onchange={app.onPickedLibraries}
/>

{#if app.libGhost}
	<ComponentGhost {...app.libGhost} />
{/if}

{#if app.ctxMenu}
	<ContextMenu x={app.ctxMenu.x} y={app.ctxMenu.y} onClose={() => (app.ctxMenu = null)}>
		{#if app.ctxMenu.kind === 'edit'}
			<EditMenu />
		{:else if app.ctxMenu.kind === 'layer'}
			<LayerContextMenu index={app.ctxMenu.index} />
		{:else if app.ctxMenu.kind === 'library'}
			<LibraryContextMenu stem={app.ctxMenu.stem} />
		{:else}
			<LibraryItemContextMenu stem={app.ctxMenu.stem} componentKey={app.ctxMenu.key} />
		{/if}
	</ContextMenu>
{/if}

{#if app.menu}
	<Scrim z="var(--z-scrim)" label="close menu" onclick={app.closeMenu} />
{/if}

{#if dialog?.kind === 'deleteLayer'}
	<DeleteLayerDialog index={dialog.index} />
{:else if dialog?.kind === 'deleteComponent'}
	<DeleteComponentDialog />
{:else if dialog?.kind === 'deleteLibrary'}
	<DeleteLibraryDialog />
{:else if dialog?.kind === 'saveLocalComponents'}
	<SaveLocalComponentsDialog />
{:else if dialog?.kind === 'about'}
	<AboutDialog />
{:else if dialog?.kind === 'technologies'}
	<TechnologiesDialog />
{:else if dialog?.kind === 'projectSettings'}
	<ProjectSettingsDialog
		t={app.t}
		values={{
			gridX: app.status.grid,
			gridY: app.status.grid_y,
			snapX: app.status.snap,
			snapY: app.status.snap_y,
			showGrid: app.status.show_grid,
			snapEnable: app.status.snap_enable,
			hideComponentOrigin: app.status.hide_component_origin,
			strokeHundredths: app.status.stroke_hundredths,
			defaultFilled: app.status.default_filled
		}}
		onApply={app.applyProjectSettings}
		onCancel={() => app.dialogs.close()}
	/>
{:else if dialog?.kind === 'properties'}
	<PropertiesDialog
		t={app.t}
		fields={dialog.fields}
		layers={app.layers}
		onApply={app.applyProperties}
		onCancel={() => app.dialogs.close()}
	/>
{:else if dialog?.kind === 'error'}
	<ErrorDialog />
{:else if dialog?.kind === 'discard'}
	<ConfirmDialog />
{:else if dialog?.kind === 'shareLink'}
	<ShareLinkDialog url={dialog.url} />
{:else if dialog?.kind === 'shareFcd'}
	<ShareFcdDialog text={dialog.text} />
{:else if dialog?.kind === 'export'}
	<ExportPreviewDialog format={dialog.format} />
{:else if dialog?.kind === 'unresolvedComponents'}
	<UnresolvedComponentsDialog />
{/if}
