<script lang="ts">
	import { formatLuAsMm } from '../lib/constants';
	import { clampInt } from '../lib/num';

	let {
		label,
		x,
		y,
		enabled,
		enabledLabel,
		offLabel,
		min,
		max,
		onX,
		onY,
		onEnabled
	}: {
		label: string;
		x: number;
		y: number;
		enabled: boolean;
		enabledLabel: string;
		offLabel: string;
		min: number;
		max: number;
		onX: (v: number) => void;
		onY: (v: number) => void;
		onEnabled: (v: boolean) => void;
	} = $props();

	const text = $derived(`${label} ${x}×${y}${enabled ? '' : ` ${offLabel}`}`);

	function commit(raw: string, apply: (v: number) => void) {
		const n = Number(raw);
		if (raw === '' || !Number.isFinite(n)) return;
		apply(clampInt(n, min, max));
	}
</script>

<div class="wrap">
	<button type="button" class={['pill', { off: !enabled }]} aria-haspopup="true" aria-label={text}>
		{text}
	</button>
	<div class="flyout">
		<div class="balloon" role="group" aria-label={label}>
			<div class="row">
				<label>
					<span class="axis">X</span>
					<input
						type="number"
						{min}
						{max}
						step="1"
						value={x}
						oninput={(e) => commit(e.currentTarget.value, onX)}
					/>
				</label>
				<span class="mm">{formatLuAsMm(x)}</span>
			</div>
			<div class="row">
				<label>
					<span class="axis">Y</span>
					<input
						type="number"
						{min}
						{max}
						step="1"
						value={y}
						oninput={(e) => commit(e.currentTarget.value, onY)}
					/>
				</label>
				<span class="mm">{formatLuAsMm(y)}</span>
			</div>
			<label class="chk">
				<input
					type="checkbox"
					checked={enabled}
					onchange={(e) => onEnabled(e.currentTarget.checked)}
				/>
				{enabledLabel}
			</label>
		</div>
	</div>
</div>

<style>
	.wrap {
		position: relative;
	}
	.wrap:hover,
	.wrap:focus-within {
		z-index: var(--z-flyout);
	}
	.pill {
		font: inherit;
		font-size: inherit;
		line-height: 1.4;
		padding: 0 7px;
		border: 1px solid var(--border);
		border-radius: 999px;
		background: transparent;
		color: inherit;
		white-space: nowrap;
	}
	.pill.off {
		opacity: 0.65;
	}
	.wrap:hover .pill,
	.wrap:focus-within .pill {
		border-color: var(--accent);
		color: var(--fg);
		opacity: 1;
	}
	.pill:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
	}
	.flyout {
		display: none;
		position: absolute;
		bottom: 100%;
		left: 50%;
		transform: translateX(-50%);
		padding-bottom: 6px;
	}
	.wrap:hover .flyout,
	.wrap:focus-within .flyout {
		display: block;
	}
	.balloon {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: 5px 6px;
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: 6px;
		box-shadow: var(--shadow);
		color: var(--fg);
		font-family: var(--font);
		font-size: 11px;
		line-height: 1.2;
		white-space: nowrap;
		position: relative;
	}
	.balloon::after {
		content: '';
		position: absolute;
		left: 50%;
		bottom: -4px;
		width: 7px;
		height: 7px;
		background: var(--bg-menu);
		border-right: 1px solid var(--border);
		border-bottom: 1px solid var(--border);
		transform: translateX(-50%) rotate(45deg);
	}
	.row,
	.chk {
		display: flex;
		align-items: center;
		gap: 5px;
		margin: 0;
	}
	.row label {
		display: flex;
		align-items: center;
		gap: 5px;
		margin: 0;
		cursor: pointer;
	}
	.axis {
		width: 8px;
		font-family: var(--mono);
		font-size: 11px;
		color: var(--fg-muted);
		cursor: default;
	}
	.balloon input[type='number'] {
		width: 56px;
		padding: 1px 2px 1px 4px;
		font-family: var(--mono);
		font-size: 11px;
		line-height: 1.3;
		border-radius: 4px;
	}
	.balloon input[type='number']::-webkit-inner-spin-button,
	.balloon input[type='number']::-webkit-outer-spin-button {
		opacity: 1;
		display: block;
		height: 1.35em;
		cursor: pointer;
	}
	.mm {
		min-width: 4.6em;
		font-family: var(--mono);
		font-size: 10px;
		color: var(--fg-muted);
		cursor: default;
		pointer-events: none;
		user-select: none;
	}
	.chk {
		padding-top: 1px;
		color: var(--fg-muted);
		cursor: pointer;
	}
	.balloon input[type='checkbox'] {
		margin: 0;
		accent-color: var(--accent);
	}
</style>
