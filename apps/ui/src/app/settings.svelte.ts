import { dict, type Locale } from '../i18n';
import type { Theme } from './types';
import type { Engine } from './engine.svelte';

export class Settings {
	locale = $state<Locale>((navigator.language.startsWith('en') ? 'en' : 'it') as Locale);
	theme = $state<Theme>('light');
	t = $derived.by(() => dict(this.locale));

	constructor(
		private getEngine: () => Engine | null,
		private onApplyTheme: (theme: Theme) => void,
		initial?: { locale: Locale; theme: Theme }
	) {
		if (initial) {
			this.locale = initial.locale;
			this.theme = initial.theme;
		}
		$effect(() => {
			const loc = this.locale;
			const theme = this.theme;
			document.documentElement.lang = loc;
			document.documentElement.dataset.theme = theme;
			this.getEngine()?.query((app) => {
				app.set_locale(loc);
			});
		});
	}

	applyTheme = () => {
		document.documentElement.dataset.theme = this.theme;
		this.onApplyTheme(this.theme);
	};

	setLocale = (loc: Locale) => {
		this.locale = loc;
	};

	setTheme = (theme: Theme) => {
		this.theme = theme;
		this.applyTheme();
	};
}
