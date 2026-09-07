<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	let { text }: { text: string } = $props();

	const app = getAppSession();

	let copied = $state(false);

	async function copy() {
		if (!text) return;
		await navigator.clipboard.writeText(text);
		copied = true;
	}
</script>

<Modal
	title={app.t.shareFcdTitle}
	titleId="share-fcd-title"
	maxWidth="640px"
	closable
	closeLabel={app.t.close}
	onClose={app.closeShare}
>
	<textarea readonly value={text} rows="16"></textarea>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={() => void copy()}>
				{copied ? app.t.copied : app.t.copyLink}
			</button>
		</div>
	{/snippet}
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
	.dialog-actions {
		margin-top: 8px;
	}
</style>
