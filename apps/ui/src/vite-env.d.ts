/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module 'virtual:fidorust-pro' {
	export function register(): Promise<{ Overlay: import('svelte').Component } | null>;
}
