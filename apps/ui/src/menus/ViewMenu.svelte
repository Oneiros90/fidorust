<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();

	function run(fn: () => void) {
		fn();
		app.closeMenu();
	}
</script>

<MenuItem
	label={app.t.gridSnap}
	onclick={() =>
		run(() => {
			app.showGridDlg = true;
		})}
/>
<MenuItem label={app.t.fit} onclick={() => run(app.fit)} />
<MenuItem label={app.t.pcbMode} onclick={() => run(app.togglePcb)} />
<MenuItem
	label={app.t.layers}
	onclick={() =>
		run(() => {
			app.showLayers = true;
		})}
/>
