<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { componentFullName } from '../lib/libraryDrag';
	import LibraryItem from './LibraryItem.svelte';

	const app = getAppSession();
	const builtin = $derived(app.libs.filter((l) => l.kind === 'builtin'));
	const user = $derived(app.libs.filter((l) => l.kind !== 'builtin'));

	function titleFor(stem: string, fallback: string) {
		if (stem === 'project') return app.t.projectLibrary;
		if (stem === 'local') return app.t.localLibrary;
		return fallback;
	}
</script>

<div class="tree" id="library-tree">
	<section class="group">
		<h2 class="group-title">{app.t.builtinLibraries}</h2>
		<div class="group-body">
			{#each builtin as lib (lib.stem)}
				<details class="node" open={lib.stem === 'stdlib'}>
					<summary class="node-label">{lib.title}</summary>
					<div class="kids">
						{#each lib.categories as cat (`${lib.stem}:${cat.name}`)}
							<details class="node">
								<summary class="node-label">{cat.name}</summary>
								<div class="kids">
									{#each cat.components as item (`${lib.stem}:${item.key}`)}
										<LibraryItem
											engine={app.engine}
											stem={lib.stem}
											componentKey={item.key}
											label={item.name}
											description={item.description}
											writable={false}
											selected={app.status.pending_component ===
												componentFullName(lib.stem, item.key)}
											theme={app.theme}
											onPick={app.pickComponent}
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
	</section>
	<section class="group">
		<h2 class="group-title">{app.t.userLibraries}</h2>
		<div class="group-body">
			{#each user as lib (lib.stem)}
				<details
					class="node"
					open={app.expandedUserLibs[lib.stem] ?? lib.stem === 'project'}
					ontoggle={(e) => {
						app.expandedUserLibs = {
							...app.expandedUserLibs,
							[lib.stem]: (e.currentTarget as HTMLDetailsElement).open
						};
					}}
				>
					<summary class="node-label">{titleFor(lib.stem, lib.title)}</summary>
					<div class="kids">
						{#each lib.categories as cat (`${lib.stem}:${cat.name}`)}
							{#each cat.components as item (`${lib.stem}:${item.key}`)}
								<LibraryItem
									engine={app.engine}
									stem={lib.stem}
									componentKey={item.key}
									label={item.name}
									description={item.description}
									writable={lib.writable}
									selected={app.status.pending_component ===
										componentFullName(lib.stem, item.key) ||
										(app.libraryFocus?.stem === lib.stem && app.libraryFocus?.key === item.key)}
									theme={app.theme}
									onPick={app.pickComponent}
									onArmDrag={app.armLibraryDrag}
								/>
							{/each}
						{/each}
					</div>
				</details>
			{/each}
		</div>
	</section>
</div>

<style>
	.tree {
		overflow: auto;
		padding: 8px 8px 12px;
		font: inherit;
		min-height: 0;
		flex: 1;
	}
	.group + .group {
		margin-top: 16px;
	}
	.group-title {
		margin: 0 6px 8px;
		padding: 0 0 6px;
		border-bottom: 1px solid var(--border);
		font: inherit;
		font-size: 13px;
		font-weight: 700;
		line-height: 1.3;
		color: var(--fg);
	}
	.group-body {
		display: flex;
		flex-direction: column;
		gap: 1px;
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
