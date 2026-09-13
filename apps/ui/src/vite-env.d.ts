/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module 'virtual:fidorust-pro' {
	export function register(): Promise<{
		mount: (target: HTMLElement, host: import('./lib/proHost').ProHost) => () => void;
	} | null>;
}
