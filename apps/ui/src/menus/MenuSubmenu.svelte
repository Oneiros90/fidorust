<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		label,
		children
	}: {
		label: string;
		children: Snippet;
	} = $props();
</script>

<div class="sub">
	<button type="button" class="sub-btn" aria-haspopup="menu">
		{label}<span class="acc">›</span>
	</button>
	<div class="flyout">{@render children()}</div>
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
	.sub:hover > .sub-btn,
	.sub:focus-within > .sub-btn {
		background: var(--bg-panel);
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
		z-index: 6;
	}
	.flyout::before {
		content: '';
		position: absolute;
		left: -8px;
		top: 0;
		bottom: 0;
		width: 8px;
	}
	.sub:hover > .flyout,
	.sub:focus-within > .flyout {
		display: flex;
	}
</style>
