<script lang="ts">
	import { onMount } from 'svelte';
	import type { App as WasmApp } from './wasm/fidocad_wasm.js';

	let { engine = $bindable(), onStatus }: { engine: WasmApp | null; onStatus: () => void } =
		$props();

	let canvas: HTMLCanvasElement;
	let wrap: HTMLDivElement;
	let space = $state(false);

	function size() {
		if (!canvas || !engine) return;
		const r = wrap.getBoundingClientRect();
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		canvas.width = Math.max(1, Math.floor(r.width * dpr));
		canvas.height = Math.max(1, Math.floor(r.height * dpr));
		canvas.style.width = `${r.width}px`;
		canvas.style.height = `${r.height}px`;
		engine.resize(canvas.width, canvas.height);
		engine.render();
	}

	function local(e: PointerEvent) {
		const r = canvas.getBoundingClientRect();
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		return { x: (e.clientX - r.left) * dpr, y: (e.clientY - r.top) * dpr };
	}

	function down(e: PointerEvent) {
		if (!engine) return;
		canvas.setPointerCapture(e.pointerId);
		const p = local(e);
		engine.pointer_down(p.x, p.y, e.shiftKey, space || e.button === 1);
		onStatus();
		engine.render();
	}
	function move(e: PointerEvent) {
		if (!engine) return;
		const p = local(e);
		engine.pointer_move(p.x, p.y);
		onStatus();
		engine.render();
	}
	function up(e: PointerEvent) {
		if (!engine) return;
		const p = local(e);
		engine.pointer_up(p.x, p.y);
		onStatus();
		engine.render();
	}
	function wheel(e: WheelEvent) {
		if (!engine) return;
		e.preventDefault();
		const r = canvas.getBoundingClientRect();
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		engine.wheel((e.clientX - r.left) * dpr, (e.clientY - r.top) * dpr, e.deltaY);
		engine.render();
		onStatus();
	}

	onMount(() => {
		const ro = new ResizeObserver(size);
		ro.observe(wrap);
		return () => ro.disconnect();
	});

	$effect(() => {
		if (engine && canvas) {
			engine.attach_canvas(canvas);
			size();
		}
	});
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.code === 'Space') space = true;
	}}
	onkeyup={(e) => {
		if (e.code === 'Space') space = false;
	}}
/>

<div class="wrap" bind:this={wrap}>
	<canvas
		id="draw-canvas"
		bind:this={canvas}
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
		ondblclick={() => {
			engine?.dblclick();
			engine?.render();
		}}
		onwheel={wheel}
		oncontextmenu={(e) => e.preventDefault()}
	></canvas>
</div>

<style>
	.wrap {
		flex: 1;
		min-width: 0;
		min-height: 0;
		position: relative;
		background: var(--canvas-bg);
	}
	canvas {
		display: block;
		width: 100%;
		height: 100%;
		touch-action: none;
		cursor: crosshair;
	}
</style>
