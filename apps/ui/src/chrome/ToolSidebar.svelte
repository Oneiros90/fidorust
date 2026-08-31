<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { TOOLS } from '../app/types';
	import ToolIcon from './ToolIcon.svelte';

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
	<label class="chk">
		<input
			type="checkbox"
			checked={app.filled}
			onchange={(e) => app.setFilled(e.currentTarget.checked)}
		/>
		{app.t.filled}
	</label>
	<label class="chk">
		{app.t.trackWidth}
		<input
			type="number"
			min="1"
			max="100"
			value="4"
			onchange={(e) => app.setTrackWidth(Number(e.currentTarget.value))}
		/>
	</label>
</nav>

<style>
	.tools {
		width: 56px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 6px;
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
		overflow: auto;
	}
	.chk {
		font-size: 10px;
		color: var(--fg-muted);
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 8px;
	}
	.chk input[type='number'] {
		width: 100%;
	}
</style>
