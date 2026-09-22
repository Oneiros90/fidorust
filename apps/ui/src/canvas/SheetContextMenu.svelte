<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import MenuItem from '../menus/MenuItem.svelte';
	import MenuSeparator from '../menus/MenuSeparator.svelte';

	let { pane, index }: { pane: number; index: number } = $props();

	const app = getAppSession();
	const last = $derived((app.status.sheets?.length ?? 1) <= 1);
</script>

<MenuItem label={app.t.renameSheet} onclick={() => app.beginRenameSheet(pane, index)} />
<MenuItem label={app.t.duplicateSheet} onclick={() => app.duplicateSheet(pane, index)} />
<MenuItem
	label={app.t.deleteSheet}
	disabled={last}
	onclick={() => app.requestDeleteSheet(index)}
/>
<MenuSeparator />
<MenuItem label={app.t.sheetSettings} onclick={() => app.openSheetSettings(pane, index)} />
