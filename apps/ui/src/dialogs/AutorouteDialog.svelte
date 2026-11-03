<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { formatLuAsMm } from '../lib/constants';
	import { clampInt } from '../lib/num';
	import Modal from './Modal.svelte';

	type Preview = {
		ok?: boolean;
		can?: boolean;
		reason?: string;
		components?: number;
		pads?: number;
		padsLinked?: number;
		padsMissingLk?: number;
		nets?: number;
		widthDefault?: number;
		clearanceDefault?: number;
		routed?: number;
		jumpers?: number;
		failed?: number;
	};

	const app = getAppSession();

	function parsePreview(raw: string): Preview {
		try {
			return JSON.parse(raw) as Preview;
		} catch {
			return { can: false, reason: 'none' };
		}
	}

	const initial = parsePreview(app.engine?.extCommand('router.preview', '{}') ?? '{}');
	const prefs = app.dialogs.autoroutePrefs;
	let width = $state(
		clampInt(prefs.width ?? initial.widthDefault ?? app.status.defaults.track_width ?? 4, 1, 100)
	);
	let clearance = $state(clampInt(prefs.clearance ?? initial.clearanceDefault ?? 2, 0, 40));
	let jumpers = $state(prefs.jumpers ?? true);
	let diagonal = $state(prefs.diagonal ?? false);
	let replace = $state(prefs.replace ?? false);
	let preview = $state<Preview>(initial);

	$effect(() => {
		app.dialogs.autoroutePrefs = {
			width: clampInt(width, 1, 100),
			clearance: clampInt(clearance, 0, 40),
			jumpers,
			diagonal,
			replace
		};
	});

	const reasonText = $derived(
		preview.reason === 'no_links'
			? app.t.autorouteNoLinks
			: preview.reason === 'no_nets'
				? app.t.autorouteNoNets
				: preview.can
					? ''
					: app.t.autorouteNone
	);

	const summary = $derived(
		app.t.autorouteSummary
			.replace('{components}', String(preview.components ?? 0))
			.replace('{pads}', String(preview.pads ?? 0))
			.replace('{nets}', String(preview.nets ?? 0))
	);
	const missingText = $derived(
		(preview.padsMissingLk ?? 0) > 0
			? app.t.autorouteMissing.replace('{missing}', String(preview.padsMissingLk ?? 0))
			: ''
	);

	function run() {
		const raw = app.engine?.extCommand(
			'router.apply',
			JSON.stringify({
				width: clampInt(width, 1, 100),
				clearance: clampInt(clearance, 0, 40),
				jumpers,
				diagonal,
				replace
			})
		);
		const out = parsePreview(raw ?? '{}');
		if (!out.can) {
			app.error =
				out.reason === 'no_links'
					? app.t.autorouteNoLinks
					: out.reason === 'no_nets'
						? app.t.autorouteNoNets
						: app.t.autorouteNone;
			return;
		}
		const routed = out.routed ?? 0;
		const failed = out.failed ?? 0;
		if (failed > 0) {
			app.error = app.t.autoroutePartial
				.replace('{routed}', String(routed))
				.replace('{jumpers}', String(out.jumpers ?? 0))
				.replace('{failed}', String(failed));
			return;
		}
		app.dialogs.close();
	}
</script>

<Modal
	title={app.t.autorouteTitle}
	titleId="autoroute-title"
	maxWidth="420px"
	onClose={() => app.dialogs.close()}
	onSubmit={preview.can ? run : undefined}
>
	<p class="lead">{summary}</p>
	<p class="hint">{app.t.autorouteNetsHint}</p>
	{#if missingText}
		<p class="warn">{missingText}</p>
	{/if}
	{#if reasonText}
		<p class="warn">{reasonText}</p>
	{/if}
	<label>
		{app.t.autorouteWidth}
		<span class="value">
			<input type="number" min="1" max="100" step="1" bind:value={width} />
			<span class="mm">{formatLuAsMm(width)}</span>
		</span>
	</label>
	<label>
		{app.t.autorouteClearance}
		<span class="value">
			<input type="number" min="0" max="40" step="1" bind:value={clearance} />
			<span class="mm">{formatLuAsMm(clearance)}</span>
		</span>
	</label>
	<label class="chk">
		<input type="checkbox" bind:checked={jumpers} />
		<span>
			{app.t.autorouteJumpers}
			<small>{app.t.autorouteJumpersHint}</small>
		</span>
	</label>
	<label class="chk">
		<input type="checkbox" bind:checked={diagonal} />
		<span>
			{app.t.autorouteDiagonal}
			<small>{app.t.autorouteDiagonalHint}</small>
		</span>
	</label>
	<label class="chk">
		<input type="checkbox" bind:checked={replace} />
		<span>
			{app.t.autorouteReplace}
			<small>{app.t.autorouteReplaceHint}</small>
		</span>
	</label>
	{#snippet actions()}
		<div class="dialog-actions">
			<button type="button" class="primary" disabled={!preview.can} onclick={run}>
				{app.t.autorouteRun}
			</button>
			<button type="button" onclick={() => app.dialogs.close()}>{app.t.cancel}</button>
		</div>
	{/snippet}
</Modal>

<style>
	.lead,
	.hint,
	.warn {
		margin: 0 0 12px;
		font-size: 13px;
	}
	.hint {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.warn {
		color: var(--fg-muted);
	}
	label {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin: 0 0 10px;
		font-size: 13px;
	}
	.value {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	input[type='number'] {
		width: 4.5rem;
	}
	.mm {
		color: var(--fg-muted);
		font-size: 12px;
		min-width: 4.5rem;
	}
	.chk {
		align-items: flex-start;
	}
	.chk span {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
	}
	small {
		color: var(--fg-muted);
		font-size: 12px;
	}
</style>
