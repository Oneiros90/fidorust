<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();
</script>

<MenuItem
	label={app.t.about}
	onclick={() => {
		app.showAbout = true;
		app.closeMenu();
	}}
/>
