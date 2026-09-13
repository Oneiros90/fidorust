<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	let key = $state('');

	function activate() {
		if (!key.trim()) return;
		app.pro.activate(key);
	}
</script>

<Modal
	title={app.t.licenseTitle}
	titleId="license-title"
	closable
	closeLabel={app.t.close}
	onClose={() => app.dialogs.close()}
	onSubmit={activate}
>
	<p class="lead">{app.t.licenseBody}</p>
	<label class="field">
		<span>{app.t.licenseKey}</span>
		<input
			type="text"
			bind:value={key}
			placeholder={app.t.licenseKeyPlaceholder}
			autocomplete="off"
		/>
	</label>
	<p class="hint">{app.t.licenseHint}</p>
	{#snippet actions()}
		<div class="actions">
			<a class="buy" href={app.pro.purchaseUrl} target="_blank" rel="noopener noreferrer"
				>{app.t.licenseBuy}</a
			>
			<button type="button" onclick={() => app.dialogs.close()}>{app.t.cancel}</button>
			<button type="button" class="primary" disabled={!key.trim()} onclick={activate}
				>{app.t.licenseActivate}</button
			>
		</div>
	{/snippet}
</Modal>

<style>
	.lead {
		margin: 0 0 12px;
		line-height: 1.45;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-size: 13px;
	}
	.field input {
		padding: 8px 10px;
	}
	.hint {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		align-items: center;
		gap: 8px;
		margin-top: 16px;
	}
	.buy {
		margin-right: auto;
		color: var(--accent);
		font-size: 13px;
	}
</style>
