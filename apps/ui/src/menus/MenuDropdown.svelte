<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getAppSession } from '../app/appContext';
	import { attachAnchoredMenu } from './clampMenu';
	import { setCloseMenu } from './menuContext';

	let {
		id,
		label,
		children
	}: {
		id: string;
		label: string;
		children: Snippet;
	} = $props();

	const app = getAppSession();
	const open = $derived(app.menu === id);
	setCloseMenu(() => app.closeMenu());

	const clampDropdown = attachAnchoredMenu(() => 'below');
</script>

<div class="menu">
	<button type="button" class="menu-btn" onclick={() => app.toggleMenu(id)} aria-expanded={open}>
		{label}
	</button>
	{#if open}
		<div class="dropdown" {@attach clampDropdown}>{@render children()}</div>
	{/if}
</div>

<style>
	.menu {
		position: relative;
	}
	.menu-btn {
		border: none;
		background: transparent;
		padding: 6px 10px;
	}
	.dropdown {
		visibility: hidden;
		position: fixed;
		left: 0;
		top: 0;
		min-width: min(260px, calc(100vw - 16px));
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		padding: 6px;
		display: flex;
		flex-direction: column;
		z-index: var(--z-dropdown);
		overscroll-behavior: contain;
	}
</style>
