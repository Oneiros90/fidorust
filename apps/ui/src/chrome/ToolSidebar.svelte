<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import ToolIcon from './ToolIcon.svelte';
	import { TOOLS } from '../app/types';

	const app = getAppSession();
</script>

<nav class="tools" aria-label={app.t.tools}>
	{#each TOOLS as [id, key] (id)}
		<button
			class={['icon-btn', { active: app.status.tool === id }]}
			title={app.t[key]}
			aria-label={app.t[key]}
			aria-pressed={app.status.tool === id}
			onclick={() => app.tool(id)}
		>
			<ToolIcon name={id} />
		</button>
	{/each}
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
</style>
