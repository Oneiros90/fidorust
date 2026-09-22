<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal
	title={app.t.deleteSheetTitle}
	titleId="delete-sheet-title"
	maxWidth="420px"
	onClose={app.cancelDeleteSheet}
	onSubmit={app.confirmDeleteSheet}
	ignoreEnterOnButton={false}
>
	<p>{app.t.deleteSheetBody}</p>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="danger" onclick={app.confirmDeleteSheet}>{app.t.delete}</button>
			<button type="button" onclick={app.cancelDeleteSheet}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
</style>
