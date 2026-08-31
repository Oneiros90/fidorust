<script lang="ts">
	import type { App as WasmApp } from './wasm/fidocad_wasm.js';
	import { macroFullName } from './libraryDrag';
	import LibraryItem from './LibraryItem.svelte';

	let {
		engine,
		libs,
		pendingMacro,
		theme,
		title,
		hideLabel,
		showLabel,
		onPick,
		onArmDrag
	}: {
		engine: WasmApp | null;
		libs: {
			stem: string;
			title: string;
			categories: { name: string; macros: [string, string][] }[];
		}[];
		pendingMacro: string | null;
		theme: 'light' | 'dark';
		title: string;
		hideLabel: string;
		showLabel: string;
		onPick: (stem: string, key: string) => void;
		onArmDrag: (name: string, e: PointerEvent) => void;
	} = $props();

	let collapsed = $state(matchMedia('(max-width: 768px)').matches);
	let toggleLabel = $derived(collapsed ? showLabel : hideLabel);
</script>

<aside class={['libs', { collapsed }]} aria-label={title}>
	<div class="head">
		<button
			type="button"
			class="collapse-btn"
			onclick={() => (collapsed = !collapsed)}
			aria-expanded={!collapsed}
			aria-controls="library-tree"
			title={toggleLabel}
			aria-label={toggleLabel}
		>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				{#if collapsed}
					<polyline points="15 6 9 12 15 18" />
				{:else}
					<polyline points="9 6 15 12 9 18" />
				{/if}
			</svg>
		</button>
		<h3>{title}</h3>
	</div>
	<div class="tree" id="library-tree" inert={collapsed}>
		{#each libs as lib (lib.stem)}
			<details class="node" open={lib.stem === 'stdlib'}>
				<summary class="node-label">{lib.title}</summary>
				<div class="kids">
					{#each lib.categories as cat (`${lib.stem}:${cat.name}`)}
						<details class="node">
							<summary class="node-label">{cat.name}</summary>
							<div class="kids">
								{#each cat.macros as [key, name] (`${lib.stem}:${key}`)}
									<LibraryItem
										{engine}
										stem={lib.stem}
										macroKey={key}
										label={name}
										selected={pendingMacro === macroFullName(lib.stem, key)}
										{theme}
										{onPick}
										{onArmDrag}
									/>
								{/each}
							</div>
						</details>
					{/each}
				</div>
			</details>
		{/each}
	</div>
</aside>

<style>
	.libs {
		position: relative;
		width: 300px;
		border-left: 1px solid var(--border);
		background: var(--bg-panel);
		display: flex;
		flex-direction: column;
		min-height: 0;
		font-size: 12px;
		line-height: 1.35;
		font-family: var(--font);
	}
	.libs.collapsed {
		width: 0;
		border: none;
		background: transparent;
		overflow: visible;
		pointer-events: none;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px 6px 8px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}
	.libs.collapsed .head {
		position: absolute;
		top: 8px;
		right: 100%;
		margin-right: 8px;
		padding: 0;
		border: none;
		pointer-events: auto;
		z-index: 4;
	}
	.collapse-btn {
		width: 26px;
		height: 26px;
		padding: 0;
		flex-shrink: 0;
		display: grid;
		place-items: center;
		background: var(--bg-menu);
	}
	.libs.collapsed .collapse-btn {
		box-shadow: var(--shadow);
	}
	.collapse-btn svg {
		width: 14px;
		height: 14px;
		display: block;
	}
	h3 {
		margin: 0;
		padding: 4px 2px;
		font-size: 13px;
		font-weight: 650;
		min-width: 0;
		flex: 1;
	}
	.libs.collapsed h3,
	.libs.collapsed .tree {
		display: none;
	}
	.tree {
		overflow: auto;
		padding: 8px 8px 12px;
		font: inherit;
	}
	.node {
		margin: 0;
	}
	.node-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font: inherit;
		font-size: 12px;
		line-height: 1.35;
		color: inherit;
		cursor: pointer;
		padding: 4px 6px;
		border-radius: 4px;
		list-style: none;
	}
	.node-label::-webkit-details-marker {
		display: none;
	}
	.node-label::marker {
		content: none;
	}
	.node-label::before {
		content: '';
		width: 0;
		height: 0;
		border-style: solid;
		border-width: 4px 0 4px 6px;
		border-color: transparent transparent transparent var(--fg-muted);
		flex-shrink: 0;
		transform: rotate(0deg);
		transition: transform 0.12s ease;
	}
	.node[open] > .node-label::before {
		transform: rotate(90deg);
	}
	.node-label:hover {
		background: var(--bg-menu);
	}
	.kids {
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding-inline-start: 14px;
		margin-inline-start: 7px;
		border-inline-start: 1px solid var(--border);
	}
</style>
