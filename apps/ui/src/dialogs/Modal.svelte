<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		titleId,
		labelledBy,
		closable = false,
		closeLabel,
		onClose,
		onSubmit,
		ignoreEnterOnButton = true,
		maxWidth = '720px',
		children,
		actions
	}: {
		title?: string;
		titleId?: string;
		labelledBy?: string;
		closable?: boolean;
		closeLabel?: string;
		onClose?: () => void;
		onSubmit?: () => void;
		ignoreEnterOnButton?: boolean;
		maxWidth?: string;
		children: Snippet;
		actions?: Snippet;
	} = $props();

	const labelId = $derived(titleId ?? labelledBy);

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape' && onClose) {
			e.preventDefault();
			onClose();
			return;
		}
		if (e.key === 'Enter' && onSubmit) {
			if (ignoreEnterOnButton && e.target instanceof HTMLButtonElement) return;
			e.preventDefault();
			onSubmit();
		}
	}
</script>

<svelte:window onkeydown={onClose || onSubmit ? onKey : undefined} />

<div class="modal" role="dialog" aria-modal="true" aria-labelledby={labelId}>
	<div class="card" style:--card-max={maxWidth}>
		{#if title}
			{#if closable}
				<div class="card-head">
					<h2 id={titleId}>{title}</h2>
					<button type="button" class="card-close" onclick={onClose} aria-label={closeLabel}>
						<span aria-hidden="true">×</span>
					</button>
				</div>
			{:else}
				<h2 id={titleId} class="dialog-title">{title}</h2>
			{/if}
		{/if}
		{@render children()}
		{@render actions?.()}
	</div>
</div>

<style>
	.modal {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		display: grid;
		place-items: center;
		z-index: var(--z-modal);
	}
	.card {
		background: var(--bg-menu);
		padding: 20px;
		border-radius: 12px;
		max-width: var(--card-max);
		width: min(var(--card-max), 92vw);
		max-height: 80vh;
		overflow: auto;
		box-shadow: var(--shadow);
	}
	.card-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.card-head h2 {
		margin: 0;
	}
	.card-close {
		width: 28px;
		height: 28px;
		padding: 0;
		display: grid;
		place-items: center;
		font-size: 18px;
		line-height: 1;
		flex-shrink: 0;
	}
</style>
