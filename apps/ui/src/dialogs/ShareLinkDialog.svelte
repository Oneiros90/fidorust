<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	let { url }: { url: string } = $props();

	const app = getAppSession();

	let copied = $state(false);

	async function copy() {
		if (!url) return;
		await navigator.clipboard.writeText(url);
		copied = true;
	}
</script>

<Modal
	title={app.t.shareLinkTitle}
	titleId="share-link-title"
	maxWidth="560px"
	closable
	closeLabel={app.t.close}
	onClose={app.closeShare}
>
	<div class="row">
		<input type="text" readonly value={url} />
		<button type="button" class="primary" disabled={!url} onclick={() => void copy()}>
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
	.primary {
		flex-shrink: 0;
	}
</style>
