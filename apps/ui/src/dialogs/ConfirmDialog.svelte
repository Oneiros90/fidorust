<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			app.cancelDiscard();
			return;
		}
		if (e.key === 'Enter') {
			e.preventDefault();
			app.acceptDiscard();
		}
	}
</script>

<svelte:window onkeydown={onKey} />

<Modal labelledBy="discard-title" maxWidth="420px">
	<h2 id="discard-title">{app.t.unsavedTitle}</h2>
	<p>{app.t.discardChanges}</p>
	<div class="actions">
		<button type="button" class="ok" onclick={app.acceptDiscard}>{app.t.ok}</button>
		<button type="button" onclick={app.cancelDiscard}>{app.t.cancel}</button>
	</div>
</Modal>

<style>
	h2 {
		margin: 0 0 12px;
		font-size: 15px;
		font-weight: 600;
	}
	p {
		margin: 0 0 16px;
		font-size: 13px;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
	.ok {
		background: var(--accent);
		color: var(--accent-fg);
		border-color: var(--accent);
	}
</style>
