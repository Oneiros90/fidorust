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

	function pickLocale(loc: Locale) {
		app.setLocale(loc);
		app.closeMenu();
	}

	function pickTheme(theme: Theme) {
		app.setTheme(theme);
		app.closeMenu();
	}
</script>

<MenuItem label={app.t.splitMacros} onclick={app.toggleSplitMacros} />
<MenuSubmenu label={app.t.language}>
	{#each locales as loc (loc.id)}
		<MenuItem
			label={app.t[loc.labelKey]}
			checkable
			active={app.locale === loc.id}
			onclick={() => pickLocale(loc.id)}
		/>
	{/each}
</MenuSubmenu>
<MenuSubmenu label={app.t.theme}>
	{#each themes as th (th.id)}
		<MenuItem
			label={app.t[th.labelKey]}
			checkable
			active={app.theme === th.id}
			onclick={() => pickTheme(th.id)}
		/>
	{/each}
</MenuSubmenu>
