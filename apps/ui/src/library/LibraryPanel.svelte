<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { componentFullName } from '../lib/libraryDrag';
	import { searchComponents } from './libraryList';
	import LibraryItem from './LibraryItem.svelte';
	import LibraryFolder from './LibraryFolder.svelte';

	const app = getAppSession();
	let query = $state('');
	const searching = $derived(query.trim().length > 0);
	const hits = $derived(
		searching ? searchComponents(app.libs, query, (lib) => app.libraryTitle(lib.stem)) : []
	);

	function itemSelected(stem: string, key: string) {
		return (
			app.status.pending_component === componentFullName(stem, key) ||
			(app.libraryFocus?.stem === stem && app.libraryFocus?.key === key)
		);
	}
</script>

<div class="panel">
	<div class="search">
		<input
			type="search"
			bind:value={query}
			placeholder={app.t.searchComponents}
			aria-label={app.t.searchComponents}
			onkeydown={(e) => {
				if (e.key === 'Escape') {
					e.preventDefault();
					query = '';
				}
			}}
		/>
	</div>
	<div class="tree" id="library-tree">
		{#if searching}
			{#if hits.length === 0}
				<p class="empty">{app.t.noMatchingComponents}</p>
			{:else}
				{#each hits as hit (`${hit.stem}:${hit.key}`)}
					<LibraryItem
						engine={app.engine}
						stem={hit.stem}
						componentKey={hit.key}
						label={hit.name}
						origin={hit.origin}
						writable={hit.writable}
						selected={itemSelected(hit.stem, hit.key)}
						theme={app.theme}
						onPick={app.pickComponent}
						onArmDrag={app.armLibraryDrag}
					/>
				{/each}
			{/if}
		{:else}
			<LibraryFolder />
		{/if}
	</div>
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
	}
	.search {
		flex-shrink: 0;
		padding: 8px 8px 0;
	}
	.search input {
		width: 100%;
		padding: 5px 8px;
	}
	.tree {
		overflow: auto;
		padding: 8px 8px 12px;
		font: inherit;
		min-height: 0;
		flex: 1;
	}
	.empty {
		margin: 12px 6px;
		font-size: 12px;
		color: var(--fg-muted);
	}
</style>
