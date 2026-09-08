<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import type { SaveLibraryPolicy } from '../app/fileOps';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	let policy = $state<SaveLibraryPolicy>('keep');
</script>

<Modal
	title={app.t.saveLocalComponentsTitle}
	titleId="save-local-components-title"
	maxWidth="480px"
	onClose={app.cancelSaveLocalComponents}
	onSubmit={() => app.confirmSaveLocalComponents(policy)}
	ignoreEnterOnButton={false}
>
	<p>{app.t.saveLocalComponentsBody}</p>
	<fieldset>
		<label>
			<input type="radio" name="save-local-policy" value="keep" bind:group={policy} />
			<span>
				<strong>{app.t.saveKeepLocal}</strong>
				<small>{app.t.saveKeepLocalHint}</small>
			</span>
		</label>
		<label>
			<input type="radio" name="save-local-policy" value="fold" bind:group={policy} />
			<span>
				<strong>{app.t.saveFoldProject}</strong>
				<small>{app.t.saveFoldProjectHint}</small>
			</span>
		</label>
		<label>
			<input type="radio" name="save-local-policy" value="explode" bind:group={policy} />
			<span>
				<strong>{app.t.saveExplodeLocal}</strong>
				<small>{app.t.saveExplodeLocalHint}</small>
			</span>
		</label>
	</fieldset>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={() => app.confirmSaveLocalComponents(policy)}
				>{app.t.ok}</button
			>
			<button type="button" onclick={app.cancelSaveLocalComponents}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 12px;
		font-size: 13px;
	}
	fieldset {
		margin: 0 0 16px;
		padding: 0;
		border: none;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	label {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		font-size: 13px;
		cursor: pointer;
	}
	input {
		margin-top: 3px;
	}
	span {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	small {
		color: var(--fg-muted);
		font-size: 12px;
	}
</style>
