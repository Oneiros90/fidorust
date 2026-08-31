<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import EditMenu from '../menus/EditMenu.svelte';
	import FileMenu from '../menus/FileMenu.svelte';
	import HelpMenu from '../menus/HelpMenu.svelte';
	import MenuDropdown from '../menus/MenuDropdown.svelte';
	import OptionsMenu from '../menus/OptionsMenu.svelte';
	import ViewMenu from '../menus/ViewMenu.svelte';
	import LayerSelect from './LayerSelect.svelte';

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
		<EditMenu onDone={app.closeMenu} />
	</MenuDropdown>
	<MenuDropdown id="view" label={app.t.view}><ViewMenu /></MenuDropdown>
	<MenuDropdown id="options" label={app.t.options}><OptionsMenu /></MenuDropdown>
	<MenuDropdown id="help" label={app.t.help}><HelpMenu /></MenuDropdown>
	<div class="grow"></div>
	<LayerSelect />
</header>

<style>
	.menubar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		background: var(--bg-menu);
		border-bottom: 1px solid var(--border);
		z-index: 3;
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
</style>
