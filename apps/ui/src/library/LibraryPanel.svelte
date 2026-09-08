<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import type { LibraryEntry } from '../app/engineTypes';
	import { componentFullName } from '../lib/libraryDrag';
	import LibraryItem from './LibraryItem.svelte';

	const app = getAppSession();
	const project = $derived(app.libs.find((l) => l.kind === 'project') ?? null);
	const user = $derived(app.libs.filter((l) => l.kind === 'local'));
	const builtin = $derived(app.libs.filter((l) => l.kind === 'builtin'));
	let skipRenameCommit = false;

	function focusAndSelect(node: HTMLInputElement) {
		queueMicrotask(() => {
			node.focus();
			node.select();
		});
	}

	function commitRename(stem: string, name: string) {
		if (skipRenameCommit) {
			skipRenameCommit = false;
			app.editingLibraryTitle = null;
			return;
		}
		app.renameLibrary(stem, name);
	}

	function cancelRename() {
		skipRenameCommit = true;
		app.editingLibraryTitle = null;
	}

	function onFolderContextMenu(e: MouseEvent, stem: string) {
		e.preventDefault();
		e.stopPropagation();
		app.openLibraryContextMenu(e.clientX, e.clientY, stem);
	}

	function stopToggle(e: Event) {
		e.preventDefault();
		e.stopPropagation();
	}

	function stopBubble(e: Event) {
		e.stopPropagation();
	}
</script>

{#snippet writableItems(lib: LibraryEntry)}
	{#each lib.categories as cat (`${lib.stem}:${cat.name}`)}
		{#each cat.components as item (`${lib.stem}:${item.key}`)}
			<LibraryItem
				engine={app.engine}
				stem={lib.stem}
				componentKey={item.key}
				label={item.name}
				description={item.description}
				writable={lib.writable}
				selected={app.status.pending_component === componentFullName(lib.stem, item.key) ||
					(app.libraryFocus?.stem === lib.stem && app.libraryFocus?.key === item.key)}
				theme={app.theme}
				onPick={app.pickComponent}
				onArmDrag={app.armLibraryDrag}
			/>
		{/each}
	{/each}
{/snippet}

{#snippet exportBtn(stem: string, inSummary: boolean)}
	<button
		type="button"
		class="icon"
		title={app.t.exportLibrary}
		aria-label={app.t.exportLibrary}
		onmousedown={inSummary ? stopToggle : undefined}
		onclick={(e) => {
			if (inSummary) stopToggle(e);
			app.exportLibrary(stem);
		}}
	>
		<svg viewBox="0 0 16 16" aria-hidden="true">
			<path
				d="M8 2v8M5 8l3 3 3-3M3 13h10"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</button>
{/snippet}

{#snippet deleteBtn(stem: string, inSummary: boolean)}
	<button
		type="button"
		class="icon danger"
		title={app.t.deleteLibrary}
		aria-label={app.t.deleteLibrary}
		onmousedown={inSummary ? stopToggle : undefined}
		onclick={(e) => {
			if (inSummary) stopToggle(e);
			app.requestDeleteLibrary(stem);
		}}
	>
		<svg viewBox="0 0 16 16" aria-hidden="true">
			<path
				d="M3 5h10M6 5V3.5h4V5M5.5 5v8h5V5"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</button>
{/snippet}

<div class="panel">
	<div class="tree" id="library-tree">
		<section class="group">
			<div class="group-head">
				<h2
					class="group-title"
					title={app.t.projectLibraryHint}
					oncontextmenu={(e) => onFolderContextMenu(e, 'project')}
				>
					{app.t.projectLibrary}
				</h2>
				<span class="group-actions">
					{@render exportBtn('project', false)}
					{@render deleteBtn('project', false)}
				</span>
			</div>
			<div class="group-body">
				{#if project}
					{@render writableItems(project)}
				{/if}
			</div>
		</section>
		<section class="group">
			<div class="group-head">
				<h2 class="group-title" title={app.t.userLibrariesHint}>{app.t.userLibraries}</h2>
				<span class="group-actions">
					<button
						type="button"
						class="icon"
						title={app.t.createLibrary}
						aria-label={app.t.createLibrary}
						onclick={app.createUserLibrary}
					>
						<svg viewBox="0 0 16 16" aria-hidden="true">
							<path
								d="M8 3v10M3 8h10"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
								stroke-linecap="round"
							/>
						</svg>
					</button>
					<button
						type="button"
						class="icon"
						title={app.t.importLibrary}
						aria-label={app.t.importLibrary}
						onclick={app.importLibrary}
					>
						<svg viewBox="0 0 16 16" aria-hidden="true">
							<path
								d="M8 14V6M5 9l3-3 3 3M3 3h10"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
								stroke-linecap="round"
								stroke-linejoin="round"
							/>
						</svg>
					</button>
				</span>
			</div>
			<div class="group-body">
				{#each user as lib (lib.stem)}
					<details
						class="node"
						open={app.expandedUserLibs[lib.stem] ?? false}
						ontoggle={(e) => {
							app.expandedUserLibs = {
								...app.expandedUserLibs,
								[lib.stem]: (e.currentTarget as HTMLDetailsElement).open
							};
						}}
					>
						<summary
							class="node-label folder"
							oncontextmenu={(e) => onFolderContextMenu(e, lib.stem)}
						>
							{#if app.editingLibraryTitle === lib.stem}
								<input
									class="rename"
									value={lib.title}
									{@attach focusAndSelect}
									onmousedown={stopBubble}
									onclick={stopBubble}
									onblur={(e) => commitRename(lib.stem, e.currentTarget.value)}
									onkeydown={(e) => {
										if (e.key === 'Enter') {
											e.preventDefault();
											e.currentTarget.blur();
										} else if (e.key === 'Escape') {
											e.preventDefault();
											cancelRename();
										}
									}}
								/>
							{:else}
								<span class="title">{lib.title}</span>
							{/if}
							<span class="lib-actions">
								{@render exportBtn(lib.stem, true)}
								{@render deleteBtn(lib.stem, true)}
							</span>
						</summary>
						<div class="kids">
							{@render writableItems(lib)}
						</div>
					</details>
				{/each}
			</div>
		</section>
		<section class="group">
			<h2 class="group-title standalone" title={app.t.builtinLibrariesHint}>
				{app.t.builtinLibraries}
			</h2>
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
	</div>
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
	}
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
	.group-head {
		display: flex;
		align-items: center;
		gap: 4px;
		margin: 0 6px 8px;
		padding: 0 0 6px;
		border-bottom: 1px solid var(--border);
	}
	.group-title {
		margin: 0;
		padding: 0;
		border: none;
		font: inherit;
		font-size: 13px;
		font-weight: 700;
		line-height: 1.3;
		color: var(--fg);
		min-width: 0;
		flex: 1;
	}
	.group-title.standalone {
		margin: 0 6px 8px;
		padding: 0 0 6px;
		border-bottom: 1px solid var(--border);
		flex: none;
	}
	.group-actions {
		display: flex;
		align-items: center;
		margin-left: auto;
		flex-shrink: 0;
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
	.title {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rename {
		flex: 1;
		min-width: 0;
		padding: 1px 4px;
		font: inherit;
		font-size: 12px;
	}
	.lib-actions {
		display: flex;
		align-items: center;
		margin-left: auto;
		flex-shrink: 0;
		opacity: 0.55;
	}
	.node-label:hover .lib-actions,
	.lib-actions:focus-within {
		opacity: 1;
	}
	.icon {
		width: 22px;
		height: 22px;
		padding: 0;
		display: grid;
		place-items: center;
		background: transparent;
		border-color: transparent;
		color: var(--fg-muted);
	}
	.icon svg {
		width: 14px;
		height: 14px;
		display: block;
	}
	.icon:hover:not(:disabled) {
		color: var(--fg);
	}
	.icon.danger:hover:not(:disabled) {
		color: var(--danger);
		border-color: var(--danger);
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
