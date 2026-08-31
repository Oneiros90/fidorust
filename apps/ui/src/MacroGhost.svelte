<script lang="ts">
	let {
		svg,
		x,
		y,
		ox,
		oy,
		w,
		h,
		scale
	}: {
		svg: string;
		x: number;
		y: number;
		ox: number;
		oy: number;
		w: number;
		h: number;
		scale: number;
	} = $props();

	const attachSvg = $derived.by(() => {
		const raw = svg;
		return (node: HTMLElement) => {
			if (!raw) return;
			const parsed = new DOMParser().parseFromString(raw, 'image/svg+xml');
			const el = parsed.documentElement;
			if (el.tagName.toLowerCase() === 'svg') {
				node.replaceChildren(document.importNode(el, true));
			}
		};
	});
</script>

<div
	class="ghost"
	style:left="{x - ox * scale}px"
	style:top="{y - oy * scale}px"
	style:width="{w * scale}px"
	style:height="{h * scale}px"
	{@attach attachSvg}
></div>

<style>
	.ghost {
		position: fixed;
		pointer-events: none;
		z-index: 40;
		overflow: visible;
		opacity: 0.92;
	}
	.ghost :global(svg) {
		display: block;
		width: 100%;
		height: 100%;
		overflow: visible;
	}
</style>
