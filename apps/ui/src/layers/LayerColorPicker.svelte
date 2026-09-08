<script lang="ts">
	import ColorPicker, { ChromeVariant, type RgbaColor } from 'svelte-awesome-color-picker';
	import { getAppSession } from '../app/appContext';
	import LayerColorInput from './LayerColorInput.svelte';

	let {
		index,
		color,
		label
	}: {
		index: number;
		color: number[];
		label: string;
	} = $props();

	const app = getAppSession();
	const components = { ...ChromeVariant, input: LayerColorInput };

	function toRgba(c: number[]): RgbaColor {
		return {
			r: c[0] ?? 0,
			g: c[1] ?? 0,
			b: c[2] ?? 0,
			a: (c[3] ?? 255) / 255
		};
	}

	let rgb = $state.raw<RgbaColor>({ r: 0, g: 0, b: 0, a: 1 });
	let isOpen = $state(false);

	function setOpen(open: boolean) {
		isOpen = open;
		if (open) app.setLayer(index);
	}

	$effect.pre(() => {
		if (!isOpen) rgb = toRgba(color);
	});

	$effect(() => {
		if (!isOpen) return;
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') e.stopImmediatePropagation();
		};
		window.addEventListener('keydown', onKey, true);
		return () => window.removeEventListener('keydown', onKey, true);
	});

	function commit(next: RgbaColor | null) {
		if (!next) return;
		const r = Math.round(next.r);
		const g = Math.round(next.g);
		const b = Math.round(next.b);
		const a = Math.round(Math.max(0, Math.min(1, next.a ?? 1)) * 255);
		const cur = color;
		if (
			r === (cur[0] ?? 0) &&
			g === (cur[1] ?? 0) &&
			b === (cur[2] ?? 0) &&
			a === (cur[3] ?? 255)
		) {
			return;
		}
		app.setLayerColor(index, r, g, b, a);
	}

	let texts = $derived({
		label: {
			h: app.t.colorHue,
			s: app.t.colorSat,
			v: app.t.colorVal,
			r: app.t.colorRed,
			g: app.t.colorGreen,
			b: app.t.colorBlue,
			a: app.t.layerOpacity,
			hex: app.t.colorHex,
			withoutColor: app.t.layer
		},
		color: {
			rgb: 'RGB',
			hsv: 'HSV',
			hex: 'HEX'
		},
		changeTo: app.t.colorChangeTo
	});
</script>

<div class="picker">
	<ColorPicker
		bind:rgb
		bind:isOpen={() => isOpen, setOpen}
		{components}
		{label}
		{texts}
		isAlpha
		sliderDirection="horizontal"
		position="responsive"
		textInputModes={['hex', 'rgb']}
		onInput={(e) => commit(e.rgb)}
	/>
</div>

<style>
	.picker {
		flex-shrink: 0;
		display: flex;
		--input-size: 22px;
		--picker-width: 228px;
		--picker-height: 128px;
		--slider-width: 12px;
		--picker-indicator-size: 10px;
		--picker-z-index: var(--z-flyout);
		--focus-color: var(--accent);
		--cp-bg-color: var(--bg-menu);
		--cp-border-color: var(--border);
		--cp-text-color: var(--fg);
		--cp-input-color: var(--bg-panel);
		--cp-button-hover-color: color-mix(in srgb, var(--accent) 18%, transparent);
		--alpha-grid-bg:
			linear-gradient(
					45deg,
					color-mix(in srgb, var(--fg) 14%, transparent) 25%,
					transparent 25%,
					transparent 75%,
					color-mix(in srgb, var(--fg) 14%, transparent) 75%
				)
				0 0 / 8px 8px,
			linear-gradient(
					45deg,
					color-mix(in srgb, var(--fg) 14%, transparent) 25%,
					transparent 25%,
					transparent 75%,
					color-mix(in srgb, var(--fg) 14%, transparent) 75%
				)
				4px 4px / 8px 8px,
			var(--bg-menu);
	}
	.picker :global(.color-picker),
	.picker :global(.a) {
		--alpha-grid-bg: inherit;
		display: inline-flex;
	}
	.picker :global(input[type='color']) {
		cursor: pointer;
	}
	.picker :global(.wrapper) {
		box-shadow: var(--shadow);
		font-family: var(--font);
		font-size: 12px;
		margin: 0;
		border-radius: var(--radius);
		--picker-width: 228px;
		--picker-height: 128px;
		--text-input-margin: 6px 8px 8px;
		overflow: hidden;
	}
	.picker :global(.h),
	.picker :global(.a) {
		--thumb-border: 1px solid var(--border);
		--thumb-background: var(--bg-menu);
	}
	.picker :global(.text-input input),
	.picker :global(.text-input button) {
		font-family: var(--mono);
		font-size: 11px;
		border-radius: 6px;
		border: 1px solid var(--border);
		background: var(--bg-panel);
		color: var(--fg);
	}
	.picker :global(.text-input input[type='number']) {
		appearance: textfield;
	}
	.picker :global(.text-input input[type='number']::-webkit-outer-spin-button),
	.picker :global(.text-input input[type='number']::-webkit-inner-spin-button) {
		appearance: none;
	}
	.picker :global(.text-input button:hover) {
		border-color: var(--accent);
		background: var(--cp-button-hover-color);
	}
</style>
