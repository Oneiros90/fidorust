<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();

	function run(fn: () => void) {
		fn();
		app.closeMenu();
	}
</script>

<MenuItem label={app.t.new} onclick={() => run(app.newDoc)} />
<MenuItem label={app.t.open} onclick={() => run(app.openFile)} />
<MenuItem label={app.t.sample} onclick={() => run(() => void app.openSample())} />
<MenuItem label={app.t.save} onclick={() => run(app.saveFile)} />
<MenuItem label={app.t.exportSvg} onclick={() => run(app.exportSvg)} />
<MenuItem label={app.t.exportPng} onclick={() => run(app.exportPng)} />
<MenuItem label={app.t.print} onclick={() => run(app.printDoc)} />
<MenuItem label={app.t.copyFcd} onclick={() => run(() => void app.copyFcd())} />
<MenuItem label={app.t.pasteFcd} onclick={() => run(() => void app.pasteFcd())} />
