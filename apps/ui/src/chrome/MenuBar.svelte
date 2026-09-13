<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import EditMenu from '../menus/EditMenu.svelte';
	import FileMenu from '../menus/FileMenu.svelte';
	import HelpMenu from '../menus/HelpMenu.svelte';
	import MenuDropdown from '../menus/MenuDropdown.svelte';
	import OptionsMenu from '../menus/OptionsMenu.svelte';
	import ViewMenu from '../menus/ViewMenu.svelte';
	import ToolIcon from './ToolIcon.svelte';

	const app = getAppSession();
</script>

<header class="menubar">
	<img
		class="brand-mark"
		src={app.assetUrl('favicon.svg')}
		width="22"
		height="22"
		alt=""
		aria-hidden="true"
	/>
	<strong class="brand">{app.t.app}</strong>
	<MenuDropdown id="file" label={app.t.file}><FileMenu /></MenuDropdown>
	<MenuDropdown id="edit" label={app.t.edit}>
		<EditMenu />
	</MenuDropdown>
	<MenuDropdown id="view" label={app.t.view}><ViewMenu /></MenuDropdown>
	<MenuDropdown id="options" label={app.t.options}><OptionsMenu /></MenuDropdown>
	<MenuDropdown id="help" label={app.t.help}><HelpMenu /></MenuDropdown>
	<div class="grow"></div>
	<button
		type="button"
		class={['circuit-btn', { active: app.pro.circuitOn, locked: !app.pro.licensed }]}
		title={app.pro.licensed ? app.t.circuitMode : app.t.circuitModeLocked}
		aria-label={app.pro.licensed ? app.t.circuitMode : app.t.circuitModeLocked}
		aria-pressed={app.pro.circuitOn}
		disabled={app.pro.loading}
		onclick={() => void app.pro.requestCircuit()}
	>
		<ToolIcon name="circuit" />
		<span>{app.t.circuitMode}</span>
		{#if !app.pro.licensed}
			<svg
				class="lock"
				viewBox="0 0 16 16"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				aria-hidden="true"
			>
				<rect x="3.2" y="7.2" width="9.6" height="6.4" rx="1.2" />
				<path d="M5.2 7.2V5.4a2.8 2.8 0 0 1 5.6 0V7.2" />
			</svg>
		{/if}
	</button>
</header>

<style>
	.menubar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		background: var(--bg-menu);
		border-bottom: 1px solid var(--border);
		z-index: var(--z-menubar);
	}
	.brand-mark {
		width: 22px;
		height: 22px;
		flex-shrink: 0;
		margin-right: 2px;
	}
	.brand {
		margin-right: 12px;
		letter-spacing: 0.04em;
		color: var(--accent);
	}
	.grow {
		flex: 1;
	}
	.circuit-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-left: 8px;
		padding: 5px 12px 5px 8px;
		border-radius: 999px;
		border: 1px solid #5aa0e6;
		background: linear-gradient(180deg, #1c4f8f 0%, #123a6b 100%);
		color: #e8f3ff;
		font: 600 12px/1 var(--font, system-ui);
		letter-spacing: 0.04em;
		text-transform: uppercase;
		box-shadow: 0 1px 0 rgba(255, 255, 255, 0.12) inset, 0 4px 12px rgba(12, 40, 80, 0.28);
		cursor: pointer;
	}
	.circuit-btn :global(svg) {
		width: 16px;
		height: 16px;
	}
	.circuit-btn .lock {
		width: 12px;
		height: 12px;
		opacity: 0.85;
	}
	.circuit-btn:hover:not(:disabled) {
		background: linear-gradient(180deg, #2560a8 0%, #184a82 100%);
	}
	.circuit-btn.active {
		border-color: #f03a3a;
		background: linear-gradient(180deg, #c42828 0%, #8e1c1c 100%);
		box-shadow: 0 1px 0 rgba(255, 255, 255, 0.16) inset, 0 0 0 1px rgba(240, 58, 58, 0.35);
	}
	.circuit-btn:disabled {
		opacity: 0.65;
		cursor: wait;
	}
	.circuit-btn.locked {
		padding-right: 10px;
	}
</style>
