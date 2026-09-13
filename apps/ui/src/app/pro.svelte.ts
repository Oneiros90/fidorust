import type { Component } from 'svelte';
import { activateLicense, deactivateLicense, isLicensed, PRO_PURCHASE_URL } from '../lib/license';
import type { AppSession } from './appSession.svelte';

export type ProOverlay = Component;

type ProPack = { Overlay: ProOverlay };

export class ProSession {
	licensed = $state(isLicensed());
	active = $state(false);
	wasmPro = $state(false);
	lastResult = $state<string | null>(null);
	Overlay = $state.raw<ProOverlay | null>(null);

	constructor(private readonly app: AppSession) {}

	get purchaseUrl() {
		return PRO_PURCHASE_URL;
	}

	load = async () => {
		const caps = this.app.engine?.capabilities() ?? { pro: false, commands: [] };
		this.wasmPro = caps.pro;
		const { register } = await import('virtual:fidorust-pro');
		const pack = (await register()) as ProPack | null;
		this.Overlay = pack?.Overlay ?? null;
	};

	tryEnter = () => {
		if (!this.licensed) {
			this.app.dialogs.open({ kind: 'license' });
			return;
		}
		this.runSmoke();
		this.active = true;
	};

	exit = () => {
		this.active = false;
	};

	activate = (key: string) => {
		if (!activateLicense(key)) return;
		this.licensed = true;
		this.app.dialogs.close();
		this.tryEnter();
	};

	deactivate = () => {
		deactivateLicense();
		this.licensed = false;
		this.active = false;
		this.lastResult = null;
		this.app.dialogs.close();
	};

	runSmoke = () => {
		this.lastResult = this.app.engine?.extCommand('ping', '{}') ?? null;
	};
}
