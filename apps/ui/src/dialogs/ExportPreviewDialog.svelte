<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { queryExportSvg } from '../app/exportOps';
	import {
		defaultExportOpts,
		type ExportFormat,
		type ExportPreviewOpts
	} from '../lib/exportOptions';
	import Modal from './Modal.svelte';
	import ExportPreviewStage from './ExportPreviewStage.svelte';
	import ExportFormatPanel from './ExportFormatPanel.svelte';
	import ExportLayerPanel from './ExportLayerPanel.svelte';
	import { untrack } from 'svelte';

	let { format }: { format: ExportFormat } = $props();

	const app = getAppSession();
	let opts = $state<ExportPreviewOpts>(untrack(() => defaultExportOpts(format, app.layers.layers)));
	const svg = $derived(queryExportSvg(app, $state.snapshot(opts)));
	const title = $derived(opts.format === 'print' ? app.t.printPreview : app.t.exportPreview);
	const confirmLabel = $derived(opts.format === 'print' ? app.t.print : app.t.exportAction);

	function confirm() {
		void app.confirmExport(opts, svg);
	}
</script>

<Modal
	{title}
	titleId="export-dlg-title"
	closable
	closeLabel={app.t.close}
	maxWidth="1120px"
	maxHeight="none"
	overflow="visible"
	onClose={() => app.dialogs.close()}
>
	<div class="export-dlg">
		<div class="body">
			<ExportPreviewStage {svg} {opts} {title} />
			<div class="side">
				<ExportFormatPanel bind:opts {svg} />
				<ExportLayerPanel bind:opts />
			</div>
		</div>
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={confirm}>{confirmLabel}</button>
			<button type="button" onclick={() => app.dialogs.close()}>{app.t.cancel}</button>
		</div>
	</div>
</Modal>

<style>
	.export-dlg {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: min(82vh, 760px);
		min-height: 420px;
	}
	.body {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 280px;
		gap: 16px;
		flex: 1;
		min-height: 0;
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow: auto;
		min-height: 0;
		padding-right: 4px;
	}
	.side:has(:global(.is-open)) {
		overflow: visible;
		position: relative;
		z-index: calc(var(--z-modal) + 1);
	}
	.dialog-actions {
		margin-top: 0;
		flex-shrink: 0;
	}
</style>
