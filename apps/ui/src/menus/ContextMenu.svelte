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
	let width = $state(0);
	let height = $state(0);

	const pad = 8;
	let left = $derived.by(() => {
		const maxW = window.innerWidth - pad * 2;
		const w = Math.min(width, maxW);
		return Math.min(Math.max(pad, x), window.innerWidth - w - pad);
	});
	let top = $derived.by(() => {
		const maxH = window.innerHeight - pad * 2;
		const h = Math.min(height, maxH);
		return Math.min(Math.max(pad, y), window.innerHeight - h - pad);
	});
</script>

<button type="button" class="scrim" aria-label={app.t.cancel} onclick={onClose}></button>
<div
	class="ctx"
	bind:offsetWidth={width}
	bind:offsetHeight={height}
	style:left="{left}px"
	style:top="{top}px"
	role="menu"
>
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
