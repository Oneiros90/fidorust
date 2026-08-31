<script lang="ts">
	import { attachSvgString } from '../lib/attachSvg';

	let {
		svg,
		x,
		y,
		ox,
		oy,
		w,
		h,
		scale,
		rot = 0
	}: {
		svg: string;
		x: number;
		y: number;
		ox: number;
		oy: number;
		w: number;
		h: number;
		scale: number;
		rot?: number;
	} = $props();

	const attachSvg = $derived(attachSvgString(svg));
</script>

<div
	class="ghost"
	style:left="{x - ox * scale}px"
	style:top="{y - oy * scale}px"
	style:width="{w * scale}px"
	style:height="{h * scale}px"
	style:transform="rotate({rot * 90}deg)"
	style:transform-origin="{ox * scale}px {oy * scale}px"
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
