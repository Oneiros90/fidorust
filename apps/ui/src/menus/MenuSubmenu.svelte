<script lang="ts">
	import type { Snippet } from 'svelte';
	import { attachAnchoredMenu, type MenuAnchorSide } from './clampMenu';

	let {
		label,
		side = 'right',
		disabled = false,
		children
	}: {
		label: string;
		side?: Exclude<MenuAnchorSide, 'below'>;
		disabled?: boolean;
		children: Snippet;
	} = $props();

	const clampFlyout = attachAnchoredMenu(() => side);
</script>

<div class={['sub', { disabled }]}>
	<button type="button" class="sub-btn" aria-haspopup="menu" {disabled}>
		{label}<span class="acc">›</span>
	</button>
	{#if !disabled}
		<div class="flyout" {@attach clampFlyout}>{@render children()}</div>
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
		visibility: hidden;
		position: fixed;
		left: 0;
		top: 0;
		min-width: min(220px, calc(100vw - 16px));
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		padding: 6px;
		flex-direction: column;
		z-index: var(--z-flyout);
		overscroll-behavior: contain;
	}
	.flyout::before,
	.flyout::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		width: 8px;
	}
	.flyout::before {
		left: -8px;
	}
	.flyout::after {
		right: -8px;
	}
	.sub:hover > .flyout,
	.sub:focus-within > .flyout {
		display: flex;
	}
</style>
