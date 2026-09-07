<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal
	title={app.t.unsavedTitle}
	titleId="discard-title"
	maxWidth="420px"
	onClose={app.cancelDiscard}
	onSubmit={app.acceptDiscard}
	ignoreEnterOnButton={false}
>
	<p>{app.t.discardChanges}</p>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={app.acceptDiscard}>{app.t.ok}</button>
			<button type="button" onclick={app.cancelDiscard}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
</style>
