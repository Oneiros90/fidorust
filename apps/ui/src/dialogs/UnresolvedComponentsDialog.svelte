<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
	const dialog = $derived(app.dialogs.dialog);
	const items = $derived(dialog?.kind === 'unresolvedComponents' ? dialog.items : []);
</script>

<Modal
	title={app.t.unresolvedComponentsTitle}
	titleId="unresolved-components-title"
	maxWidth="520px"
	overflow="hidden"
	onClose={() => app.dialogs.close()}
	onSubmit={() => app.dialogs.close()}
	ignoreEnterOnButton={false}
>
	<p>{app.t.unresolvedComponentsBody}</p>
	<ul class="list">
		{#each items as item (item.name)}
			<li>
				<span class="name">{item.name}</span>
				{#if item.count > 1}
					<span class="count"
						>{app.t.unresolvedComponentTimes.replace('{n}', String(item.count))}</span
					>
				{/if}
			</li>
		{/each}
	</ul>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" onclick={() => app.dialogs.close()}>{app.t.ok}</button>
		</div>
	{/snippet}
</Modal>

<style>
	p {
		margin: 0 0 12px;
		font-size: 13px;
		line-height: 1.45;
	}
	.list {
		margin: 0 0 16px;
		padding: 0;
		list-style: none;
		max-height: min(40vh, 320px);
		overflow: auto;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		font-family: var(--mono);
		font-size: 12px;
	}
	li {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--border);
	}
	li:last-child {
		border-bottom: none;
	}
	.name {
		overflow-wrap: anywhere;
	}
	.count {
		flex-shrink: 0;
		color: var(--fg-muted);
	}
</style>
