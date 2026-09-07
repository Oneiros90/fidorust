<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { macroFullName } from '../lib/libraryDrag';
	import LibraryItem from './LibraryItem.svelte';

	const app = getAppSession();
</script>

<div class="tree" id="library-tree">
	{#each app.libs as lib (lib.stem)}
		<details class="node" open={lib.stem === 'stdlib'}>
			<summary class="node-label">{lib.title}</summary>
			<div class="kids">
				{#each lib.categories as cat (`${lib.stem}:${cat.name}`)}
					<details class="node">
						<summary class="node-label">{cat.name}</summary>
						<div class="kids">
							{#each cat.macros as [key, name] (`${lib.stem}:${key}`)}
								<LibraryItem
									engine={app.engine}
									stem={lib.stem}
									macroKey={key}
									label={name}
									selected={app.status.pending_macro === macroFullName(lib.stem, key)}
									theme={app.theme}
									onPick={app.pickMacro}
									onArmDrag={app.armLibraryDrag}
								/>
							{/each}
						</div>
					</details>
				{/each}
			</div>
		</details>
	{/each}
</div>

<style>
	.tree {
		overflow: auto;
		padding: 8px 8px 12px;
		font: inherit;
		min-height: 0;
		flex: 1;
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
