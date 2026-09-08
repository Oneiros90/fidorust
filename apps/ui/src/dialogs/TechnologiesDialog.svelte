<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { TECHNOLOGIES } from '../lib/technologies';
	import Modal from './Modal.svelte';

	const app = getAppSession();

	let items = $derived(
		TECHNOLOGIES.map((tech) => ({
			...tech,
			role: {
				rust: app.t.techRust,
				tauri: app.t.techTauri,
				svelte: app.t.techSvelte,
				colorPicker: app.t.techColorPicker
			}[tech.id]
		}))
	);
</script>

<Modal
	title={app.t.technologies}
	titleId="technologies-title"
	maxWidth="480px"
	onClose={() => app.dialogs.close()}
>
	<ul>
		{#each items as tech (tech.id)}
			<li>
				<a href={tech.url} target="_blank" rel="noopener noreferrer">{tech.name}</a>
				<span>{tech.role}</span>
			</li>
		{/each}
	</ul>
	<button type="button" onclick={() => app.dialogs.close()}>{app.t.close}</button>
</Modal>

<style>
	ul {
		margin: 0 0 16px;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 10px;
		text-align: left;
		line-height: 1.4;
	}
	li {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	a {
		color: var(--accent);
		font-weight: 650;
	}
	span {
		color: var(--fg-muted);
		font-size: 12px;
	}
</style>
