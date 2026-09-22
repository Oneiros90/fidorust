<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();
</script>

<MenuItem label={app.t.fit} onclick={app.fit} />
<MenuItem
	label={app.t.dualSheet}
	checkable
	active={app.status.split}
	disabled={!!app.status.editing_component}
	onclick={app.toggleSplit}
/>
<MenuItem label={app.t.pcbMode} onclick={app.togglePcb} />
