<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import DialogHeader from './DialogHeader.svelte';
	import Modal from './Modal.svelte';

	const app = getAppSession();

	let copied = $state(false);

	async function copy() {
		if (!app.shareFcdText) return;
		await navigator.clipboard.writeText(app.shareFcdText);
		copied = true;
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			app.closeShare();
		}
	}
</script>

<svelte:window onkeydown={onKey} />

<Modal labelledBy="share-fcd-title" maxWidth="640px">
	<DialogHeader
		title={app.t.shareFcdTitle}
		titleId="share-fcd-title"
		closeLabel={app.t.close}
		onClose={app.closeShare}
	/>
	<textarea readonly value={app.shareFcdText ?? ''} rows="16"></textarea>
	<div class="actions">
		<button type="button" class="ok" onclick={() => void copy()}>
			{copied ? app.t.copied : app.t.copyLink}
		</button>
	</div>
</Modal>

<style>
	textarea {
		display: block;
		width: 100%;
		margin-top: 12px;
		padding: 8px;
		resize: vertical;
		font-family: var(--mono);
		font-size: 12px;
		color: var(--fg);
		background: var(--bg-panel);
		border: 1px solid var(--border);
		border-radius: 6px;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 8px;
	}
	.ok {
		background: var(--accent);
		color: var(--accent-fg);
		border-color: var(--accent);
	}
</style>
