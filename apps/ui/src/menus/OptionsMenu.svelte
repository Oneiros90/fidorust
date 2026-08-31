<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from './MenuItem.svelte';

	const app = getAppSession();
</script>

<MenuItem label={app.t.splitMacros} onclick={app.toggleSplitMacros} />
<MenuItem
	label="{app.t.language}: {app.locale.toUpperCase()}"
	onclick={app.toggleLocale}
/>
<MenuItem
	label="{app.t.theme}: {app.theme === 'light' ? app.t.light : app.t.dark}"
	onclick={app.toggleTheme}
/>
