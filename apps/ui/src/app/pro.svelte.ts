import {
	activateLicense,
	deactivateLicense,
	isLicensed,
	PRO_PURCHASE_URL,
	readLicense
} from '../lib/license';
import { clearCachedPayload, readCachedPayload, writeCachedPayload } from '../lib/proCache';
import type { ProHost, ProPack } from '../lib/proHost';
import { instantiateOverlay, instantiateProApp, revokeProBlobUrls } from '../lib/proInstantiate';
import { fetchAndDecryptModule } from '../lib/proModule';
import type { AppSession } from './appSession.svelte';

export class ProSession {
	licensed = $state(isLicensed());
	active = $state(false);
	wasmPro = $state(false);
	loading = $state(false);
	lastResult = $state<string | null>(null);
	mountOverlay = $state.raw<ProPack['mount'] | null>(null);

	constructor(private readonly app: AppSession) {}

	get purchaseUrl() {
		return PRO_PURCHASE_URL;
	}

	host = (): ProHost => ({
		t: {
			proActive: this.app.t.proActive,
			proActiveBody: this.app.t.proActiveBody,
			proDeactivate: this.app.t.proDeactivate,
			close: this.app.t.close
		},
		deactivate: () => void this.deactivate(),
		exit: () => this.exit(),
		extCommand: (name, payload) => this.app.engine?.extCommand(name, payload) ?? '{"ok":false}'
	});

	overlayAttach = (node: HTMLElement) => {
		const mount = this.mountOverlay;
		if (!mount) return;
		return mount(node, this.host());
	};

	load = async () => {
		const caps = this.app.engine?.capabilities() ?? { pro: false, commands: [] };
		this.wasmPro = caps.pro;
		const { register } = await import('virtual:fidorust-pro');
		const pack = await register();
		if (pack) {
			this.mountOverlay = pack.mount;
			return;
		}
		if (this.licensed && !this.wasmPro) {
			await this.ensureModule();
		}
	};

	tryEnter = async () => {
		if (!this.licensed) {
			this.app.dialogs.open({ kind: 'license' });
			return;
		}
		const ok = await this.ensureModule();
		if (!ok) return;
		this.runSmoke();
		this.active = true;
	};

	exit = () => {
		this.active = false;
	};

	activate = async (key: string) => {
		if (!activateLicense(key)) return;
		this.licensed = true;
		this.app.dialogs.close();
		await this.tryEnter();
	};

	deactivate = async () => {
		this.active = false;
		this.mountOverlay = null;
		deactivateLicense();
		this.licensed = false;
		this.lastResult = null;
		this.app.dialogs.close();
		await clearCachedPayload();
		revokeProBlobUrls();
		if (this.wasmPro) {
			await this.app.restoreFreeEngine();
			this.wasmPro = this.app.engine?.capabilities().pro ?? false;
		}
	};

	runSmoke = () => {
		this.lastResult = this.app.engine?.extCommand('ping', '{}') ?? null;
	};

	ensureModule = async (): Promise<boolean> => {
		if (this.wasmPro && this.mountOverlay) return true;
		if (this.loading) return false;
		this.loading = true;
		const wasPro = this.wasmPro;
		try {
			let payload = await readCachedPayload();
			if (!payload) {
				const license = readLicense();
				if (!license) return false;
				payload = await fetchAndDecryptModule(license);
				await writeCachedPayload(payload);
			}
			const wasmApp = await instantiateProApp(
				payload.files['fidorust_wasm.js'],
				payload.files['fidorust_wasm_bg.wasm']
			);
			await this.app.adoptEngine(wasmApp);
			const pack = await instantiateOverlay(payload.files['overlay.js']);
			this.mountOverlay = pack.mount;
			this.wasmPro = this.app.engine?.capabilities().pro ?? false;
			if (!this.wasmPro) throw new Error('pro module did not install');
			return true;
		} catch (err) {
			this.mountOverlay = null;
			if (!wasPro) await this.app.restoreFreeEngine();
			this.wasmPro = this.app.engine?.capabilities().pro ?? false;
			this.app.error = this.app.t.proLoadFailed;
			console.error(err);
			return false;
		} finally {
			this.loading = false;
		}
	};
}
