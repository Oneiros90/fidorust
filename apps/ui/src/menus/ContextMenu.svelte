<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getAppSession } from '../app/appContext';
	import Scrim from '../chrome/Scrim.svelte';
	import { clampMenuOrigin } from './clampMenu';
	import { setCloseMenu } from './menuContext';
	import { innerHeight, innerWidth } from 'svelte/reactivity/window';

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
	setCloseMenu(() => onClose());
	let width = $state(0);
	let height = $state(0);

	let box = $derived.by(() => {
		void innerWidth.current;
		void innerHeight.current;
		return clampMenuOrigin(x, y, width, height);
	});
</script>

<Scrim z="var(--z-context)" label={app.t.cancel} onclick={onClose} />
<div
	class="ctx"
	bind:offsetWidth={width}
	bind:offsetHeight={height}
	style:left="{box.left}px"
	style:top="{box.top}px"
	style:max-width="{box.maxW}px"
	style:max-height="{box.maxH}px"
	role="menu"
>
	{@render children()}
</div>

<style>
	.ctx {
		position: fixed;
		z-index: calc(var(--z-context) + 1);
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		min-width: min(260px, calc(100vw - 16px));
		padding: 6px;
		display: flex;
		flex-direction: column;
		overflow: visible;
	}
</style>
