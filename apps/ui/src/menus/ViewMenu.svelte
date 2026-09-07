<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();
</script>

<MenuItem label={app.t.gridSnap} onclick={app.openGrid} />
<MenuItem label={app.t.fit} onclick={app.fit} />
<MenuItem label={app.t.pcbMode} onclick={app.togglePcb} />
