/// <reference types="svelte" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
	readonly VITE_PRO_LICENSE_HASH?: string;
	readonly VITE_PRO_MODULE_BASE?: string;
}

declare module 'virtual:fidorust-pro' {
	export function register(): Promise<import('./lib/proHost').ProPack | null>;
	export const embeddedPro: boolean;
}
