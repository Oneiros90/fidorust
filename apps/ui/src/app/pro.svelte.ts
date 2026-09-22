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
import { ensureLegacyPaneApi } from '../lib/wasmCompat';
import type { AppSession } from './appSession.svelte';

function commandOn(raw: string): boolean {
	try {
		return !!(JSON.parse(raw) as { on?: boolean }).on;
	} catch {
		return false;
	}
}

export class ProSession {
	licensed = $state(isLicensed());
	active = $state(false);
	wasmPro = $state(false);
	loading = $state(false);
	circuitOn = $state(false);
	lastResult = $state<string | null>(null);
	mountOverlay = $state.raw<ProPack['mount'] | null>(null);
	private pack: ProPack | null = null;
	private unsubRefresh: (() => void) | null = null;
	private pendingCircuit = false;
	private embeddedPro = false;

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
		extCommand: (name, payload) => this.app.engine?.extCommand(name, payload) ?? '{"ok":false}',
		subscribeRefresh: (cb) => this.app.engine?.subscribeRefresh(cb) ?? (() => {})
	});

	overlayAttach = (node: HTMLElement) => {
		const mount = this.mountOverlay;
		if (!mount) return;
		return mount(node, this.host());
	};

	bindEngine = () => {
		this.unsubRefresh?.();
		this.unsubRefresh = this.app.engine
			? this.app.engine.subscribeRefresh(() => {
					if (this.circuitOn) this.app.engine?.extCommand('circuit.sync', '{}');
				})
			: null;
	};

	adoptPack = (pack: ProPack) => {
		this.pack = pack;
		this.mountOverlay = pack.mount;
		this.bindEngine();
	};

	load = async () => {
		const caps = this.app.engine?.capabilities() ?? { pro: false, commands: [] };
		this.wasmPro = caps.pro;
		const { embeddedPro, register } = await import('virtual:fidorust-pro');
		this.embeddedPro = embeddedPro;
		const pack = await register();
		if (pack) {
			this.adoptPack(pack);
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

	requestCircuit = async () => {
		if (!this.licensed) {
			this.pendingCircuit = true;
			this.app.dialogs.open({ kind: 'license' });
			return;
		}
		if (this.circuitOn) {
			this.app.engine?.extCommand('circuit.set', '{"on":false}');
			this.circuitOn = false;
			this.pack?.setCircuitChrome(false);
			return;
		}
		await this.enableCircuit();
	};

	exit = () => {
		this.active = false;
	};

	activate = async (key: string): Promise<boolean> => {
		if (!activateLicense(key)) return false;
		this.licensed = true;
		this.app.dialogs.close();
		await this.tryEnter();
		if (this.pendingCircuit) {
			this.pendingCircuit = false;
			await this.enableCircuit();
		}
		return true;
	};

	deactivate = async () => {
		this.pack?.setCircuitChrome(false);
		this.circuitOn = false;
		this.pendingCircuit = false;
		this.active = false;
		this.mountOverlay = null;
		this.pack = null;
		this.unsubRefresh?.();
		this.unsubRefresh = null;
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

	private enableCircuit = async () => {
		const ok = await this.ensureModule();
		if (!ok) return;
		this.bindEngine();
		const raw = this.app.engine?.extCommand('circuit.set', '{"on":true}') ?? '{}';
		this.circuitOn = commandOn(raw);
		this.pack?.setCircuitChrome(this.circuitOn);
	};

	ensureModule = async (): Promise<boolean> => {
		this.wasmPro = this.app.engine?.capabilities().pro ?? false;
		if (this.wasmPro) return true;
		// Linked Pro crate (FIDORUST_PRO=1): never replace the live wasm with a packed blob.
		// The published module can lag this UI and drop dual-pane methods.
		if (this.embeddedPro) {
			console.error('FidoRust Pro wasm is not linked; rebuild with FIDORUST_PRO=1');
			return false;
		}
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
			const wasmApp = ensureLegacyPaneApi(
				await instantiateProApp(
					payload.files['fidorust_wasm.js'],
					payload.files['fidorust_wasm_bg.wasm']
				)
			);
			await this.app.adoptEngine(wasmApp);
			const pack = await instantiateOverlay(payload.files['overlay.js']);
			this.adoptPack(pack);
			this.wasmPro = this.app.engine?.capabilities().pro ?? false;
			if (!this.wasmPro) throw new Error('pro module did not install');
			return true;
		} catch (err) {
			this.mountOverlay = null;
			this.pack = null;
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
