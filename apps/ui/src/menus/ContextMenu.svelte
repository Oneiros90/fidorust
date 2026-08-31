<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getAppSession } from '../app/appContext';

	let {
		x,
		y,
		onClose,
		children
	}: {
		x: number;
		y: number;
		onClose: () => void;
		children: Snippet;
	} = $props();

	const app = getAppSession();
	const left = $derived(Math.min(x, Math.max(8, window.innerWidth - 280)));
	const top = $derived(Math.min(y, Math.max(8, window.innerHeight - 400)));
</script>

<button type="button" class="scrim" aria-label={app.t.cancel} onclick={onClose}></button>
<div class="ctx" style:left="{left}px" style:top="{top}px" role="menu">
	{@render children()}
</div>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: transparent;
		border: none;
		z-index: 20;
	}
	.ctx {
		position: fixed;
		z-index: 21;
		min-width: 260px;
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		padding: 6px;
		display: flex;
		flex-direction: column;
	}
</style>
