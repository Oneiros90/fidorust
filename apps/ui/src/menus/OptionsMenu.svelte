<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import type { Locale } from '../i18n';
	import type { Theme } from '../app/types';
	import MenuItem from './MenuItem.svelte';
	import MenuSubmenu from './MenuSubmenu.svelte';

	const app = getAppSession();

	const locales: { id: Locale; labelKey: 'localeItalian' | 'localeEnglish' }[] = [
		{ id: 'it', labelKey: 'localeItalian' },
		{ id: 'en', labelKey: 'localeEnglish' }
	];

	const themes: { id: Theme; labelKey: 'light' | 'dark' }[] = [
		{ id: 'light', labelKey: 'light' },
		{ id: 'dark', labelKey: 'dark' }
	];
</script>

<MenuItem label={app.t.projectSettings} onclick={app.openProjectSettings} />
<MenuSubmenu label={app.t.language}>
	{#each locales as loc (loc.id)}
		<MenuItem
			label={app.t[loc.labelKey]}
			checkable
			active={app.locale === loc.id}
			onclick={() => app.setLocale(loc.id)}
		/>
	{/each}
</MenuSubmenu>
<MenuSubmenu label={app.t.theme}>
	{#each themes as th (th.id)}
		<MenuItem
			label={app.t[th.labelKey]}
			checkable
			active={app.theme === th.id}
			onclick={() => app.setTheme(th.id)}
		/>
	{/each}
</MenuSubmenu>
