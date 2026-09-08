export const TECHNOLOGIES = [
	{
		id: 'rust',
		name: 'Rust',
		url: 'https://www.rust-lang.org/'
	},
	{
		id: 'tauri',
		name: 'Tauri',
		url: 'https://v2.tauri.app/'
	},
	{
		id: 'svelte',
		name: 'Svelte',
		url: 'https://svelte.dev/'
	},
	{
		id: 'colorPicker',
		name: 'svelte-awesome-color-picker',
		url: 'https://github.com/Ennoriel/svelte-awesome-color-picker'
	}
] as const;

export type TechnologyId = (typeof TECHNOLOGIES)[number]['id'];
