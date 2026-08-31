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
		onPick: (stem: string, key: string) => void;
		onArmDrag: (name: string, e: PointerEvent) => void;
	} = $props();
</script>

<aside class="libs">
	<h3>{title}</h3>
	<div class="tree">
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
	h3 {
		margin: 0;
		padding: 10px 12px;
		font-size: 13px;
		font-weight: 650;
		border-bottom: 1px solid var(--border);
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
