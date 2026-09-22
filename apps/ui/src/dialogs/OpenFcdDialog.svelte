<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	const canInclude = $derived(!app.status.editing_component);
</script>

<Modal
	title={app.t.unsavedTitle}
	titleId="open-fcd-title"
	maxWidth="420px"
	onClose={app.cancelDiscard}
>
	<p>{app.t.openFcdBody}</p>
	{#snippet actions()}
		<div class="choices">
			<button type="button" class="danger" onclick={app.acceptDiscard}
				>{app.t.openFcdDiscard}</button
			>
			<button type="button" class="primary" disabled={!canInclude} onclick={app.acceptIncludeFcd}
				>{app.t.openFcdInclude}</button
			>
			<button type="button" onclick={app.cancelDiscard}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
	.choices {
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 8px;
	}
	.choices button {
		width: 100%;
	}
	.danger {
		color: var(--danger);
		border-color: var(--danger);
	}
</style>
