<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import ToolIcon from './ToolIcon.svelte';
	import { TOOL_GROUPS } from '../app/types';

	const app = getAppSession();

	$effect(() => {
		if (
			app.status.tool === 'pad-link' &&
			(!app.pro.circuitOn || app.status.editing_component)
		) {
			app.tool('select');
		}
	});
</script>

<nav class="tools" aria-label={app.t.tools}>
	{#each TOOL_GROUPS as group, gi (gi)}
		{#if gi > 0}
			<hr class="sep" />
		{/if}
		{#each group as [id, key] (id)}
			<button
				class={['icon-btn', { active: app.status.tool === id }]}
				title={app.t[key]}
				aria-label={app.t[key]}
				aria-pressed={app.status.tool === id}
				onclick={() => app.tool(id)}
				oncontextmenu={(e) => {
					e.preventDefault();
					e.stopPropagation();
					app.openToolContextMenu(e.clientX, e.clientY, id);
				}}
			>
				<ToolIcon name={id} />
			</button>
		{/each}
	{/each}
	{#if app.pro.circuitOn && !app.status.editing_component}
		<hr class="sep" />
		<button
			class={['icon-btn', { active: app.status.tool === 'pad-link' }]}
			title={app.t.padLink}
			aria-label={app.t.padLink}
			aria-pressed={app.status.tool === 'pad-link'}
			onclick={() => app.tool('pad-link')}
		>
			<ToolIcon name="pad-link" />
		</button>
	{/if}
</nav>

<style>
	.tools {
		width: 56px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 8px 6px;
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
		overflow: auto;
	}

	.sep {
		width: 28px;
		border: none;
		border-top: 1px solid var(--border);
		margin: 6px 0;
	}
</style>
