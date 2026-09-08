<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	const dialog = $derived(app.dialogs.dialog);
	const stem = $derived(dialog?.kind === 'deleteLibrary' ? dialog.stem : '');
	const isProject = $derived(stem === 'project');
</script>

<Modal
	title={isProject ? app.t.deleteProjectLibraryTitle : app.t.deleteUserLibraryTitle}
	titleId="delete-library-title"
	maxWidth="420px"
	onClose={app.cancelDeleteLibrary}
	onSubmit={() => app.confirmDeleteLibrary(stem)}
	ignoreEnterOnButton={false}
>
	<p>{isProject ? app.t.deleteProjectLibraryBody : app.t.deleteUserLibraryBody}</p>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="danger" onclick={() => app.confirmDeleteLibrary(stem)}
				>{app.t.delete}</button
			>
			<button type="button" onclick={app.cancelDeleteLibrary}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
	.danger {
		color: var(--danger);
		border-color: var(--danger);
		margin-right: auto;
	}
</style>
