<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import DialogHeader from './DialogHeader.svelte';
	import Modal from './Modal.svelte';

	const app = getAppSession();

	let copied = $state(false);

	async function copy() {
		if (!app.shareLinkUrl) return;
		await navigator.clipboard.writeText(app.shareLinkUrl);
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

<Modal labelledBy="share-link-title" maxWidth="560px">
	<DialogHeader
		title={app.t.shareLinkTitle}
		titleId="share-link-title"
		closeLabel={app.t.close}
		onClose={app.closeShare}
	/>
	<div class="row">
		<input type="text" readonly value={app.shareLinkUrl ?? ''} />
		<button type="button" class="ok" disabled={!app.shareLinkUrl} onclick={() => void copy()}>
			{copied ? app.t.copied : app.t.copyLink}
		</button>
	</div>
</Modal>

<style>
	.row {
		display: flex;
		gap: 8px;
		margin-top: 12px;
	}
	input {
		flex: 1;
		min-width: 0;
		padding: 6px 8px;
		font-family: var(--mono);
		font-size: 12px;
	}
	.ok {
		flex-shrink: 0;
		background: var(--accent);
		color: var(--accent-fg);
		border-color: var(--accent);
	}
</style>
