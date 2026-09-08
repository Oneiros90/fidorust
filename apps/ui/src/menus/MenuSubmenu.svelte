<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		label,
		side = 'right',
		disabled = false,
		children
	}: {
		label: string;
		side?: 'left' | 'right';
		disabled?: boolean;
		children: Snippet;
	} = $props();
</script>

<div class={['sub', { disabled }]}>
	<button type="button" class="sub-btn" aria-haspopup="menu" {disabled}>
		{label}<span class="acc">›</span>
	</button>
	{#if !disabled}
		<div class={['flyout', { left: side === 'left' }]}>{@render children()}</div>
	{/if}
</div>

<style>
	.sub {
		position: relative;
	}
	.sub-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		width: 100%;
		text-align: left;
		border: none;
		background: transparent;
		padding: 6px 8px;
		border-radius: 4px;
		font: inherit;
		font-size: 13px;
		color: inherit;
	}
	.sub:hover:not(.disabled) > .sub-btn,
	.sub:focus-within:not(.disabled) > .sub-btn {
		background: var(--bg-panel);
	}
	button:disabled {
		opacity: 0.45;
	}
	.acc {
		color: var(--fg-muted);
		font-size: 13px;
		line-height: 1;
	}
	.flyout {
		display: none;
		position: absolute;
		left: 100%;
		top: 0;
		min-width: 220px;
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		padding: 6px;
		flex-direction: column;
		z-index: var(--z-flyout);
	}
	.flyout::before {
		content: '';
		position: absolute;
		left: -8px;
		top: 0;
		bottom: 0;
		width: 8px;
	}
	.flyout.left {
		left: auto;
		right: 100%;
	}
	.flyout.left::before {
		left: auto;
		right: -8px;
	}
	.sub:hover > .flyout,
	.sub:focus-within > .flyout {
		display: flex;
	}
</style>
