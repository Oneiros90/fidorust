<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	const dialog = $derived(app.dialogs.dialog);
	const stem = $derived(dialog?.kind === 'deleteComponent' ? dialog.stem : '');
	const componentKey = $derived(dialog?.kind === 'deleteComponent' ? dialog.key : '');
</script>

<Modal
	title={app.t.deleteComponentTitle}
	titleId="delete-component-title"
	maxWidth="420px"
	onClose={app.cancelDeleteComponent}
	onSubmit={() => app.confirmDeleteComponent(stem, componentKey)}
	ignoreEnterOnButton={false}
>
	<p>{app.t.deleteComponentBody}</p>
	{#snippet actions()}
		<div class="dialog-actions">
			<button
				type="button"
				class="danger"
				onclick={() => app.confirmDeleteComponent(stem, componentKey)}>{app.t.delete}</button
			>
			<button type="button" onclick={app.cancelDeleteComponent}>{app.t.cancel}</button>
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
