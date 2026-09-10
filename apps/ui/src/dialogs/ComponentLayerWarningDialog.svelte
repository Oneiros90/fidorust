<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal
	title={app.t.componentLayerWarningTitle}
	titleId="component-layer-warning-title"
	maxWidth="420px"
	onClose={() => app.dialogs.close()}
	onSubmit={() => app.dialogs.close()}
	ignoreEnterOnButton={false}
>
	<p>{app.t.componentLayerWarningBody}</p>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={() => app.dialogs.close()}>{app.t.ok}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
</style>
