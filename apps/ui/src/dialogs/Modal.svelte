<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		labelledBy,
		maxWidth = '720px',
		children
	}: {
		labelledBy?: string;
		maxWidth?: string;
		children: Snippet;
	} = $props();
</script>

<div class="modal" role="dialog" aria-modal="true" aria-labelledby={labelledBy}>
	<div class="card" style:--card-max={maxWidth}>{@render children()}</div>
</div>

<style>
	.modal {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		display: grid;
		place-items: center;
		z-index: 10;
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
</style>
