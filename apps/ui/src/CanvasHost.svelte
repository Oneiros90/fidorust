<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import type { App as WasmApp } from './wasm/fidocad_wasm.js';
	import TextEditor from './TextEditor.svelte';

	type TextEdit = {
		text: string;
		wx: number;
		wy: number;
		sx: number;
		sy: number;
		angle: number;
		style: number;
		screenX: number;
		screenY: number;
		zoom: number;
	};

	let {
		engine = $bindable(),
		onStatus,
		onContextMenu
	}: {
		engine: WasmApp | null;
		onStatus: () => void;
		onContextMenu?: (clientX: number, clientY: number) => void;
	} = $props();

	let canvas: HTMLCanvasElement | undefined;
	let wrap: HTMLDivElement | undefined;
	let space = $state(false);
	let textEdit = $state.raw<TextEdit | null>(null);

	function dpr() {
		return Math.min(window.devicePixelRatio || 1, 2);
	}

	function parseEdit(raw: string): TextEdit | null {
		if (!raw || raw === 'null') return null;
		try {
			const o = JSON.parse(raw) as {
				text?: string;
				wx: number;
				wy: number;
				sx: number;
				sy: number;
				angle: number;
				style: number;
				screen_x: number;
				screen_y: number;
				zoom: number;
			};
			if (typeof o?.text !== 'string') return null;
			return {
				text: o.text,
				wx: o.wx,
				wy: o.wy,
				sx: o.sx,
				sy: o.sy,
				angle: o.angle,
				style: o.style,
				screenX: o.screen_x,
				screenY: o.screen_y,
				zoom: o.zoom
			};
		} catch {
			return null;
		}
	}

	function resizeCanvas() {
		if (!canvas || !wrap || !engine) return;
		const r = wrap.getBoundingClientRect();
		const scale = dpr();
		canvas.width = Math.max(1, Math.floor(r.width * scale));
		canvas.height = Math.max(1, Math.floor(r.height * scale));
		canvas.style.width = `${r.width}px`;
		canvas.style.height = `${r.height}px`;
		engine.resize(canvas.width, canvas.height);
		engine.render();
	}

	function local(e: { clientX: number; clientY: number }) {
		if (!canvas) return { x: 0, y: 0 };
		const r = canvas.getBoundingClientRect();
		const scale = dpr();
		return { x: (e.clientX - r.left) * scale, y: (e.clientY - r.top) * scale };
	}

	function syncEditPos() {
		const current = untrack(() => textEdit);
		if (!current || !engine) return;
		try {
			const p = JSON.parse(engine.world_to_screen_json(current.wx, current.wy)) as {
				x: number;
				y: number;
				zoom: number;
			};
			if (current.screenX === p.x && current.screenY === p.y && current.zoom === p.zoom) return;
			textEdit = { ...current, screenX: p.x, screenY: p.y, zoom: p.zoom };
		} catch {
			/* keep last layout */
		}
	}

	function openEdit(raw: string) {
		const next = parseEdit(raw);
		if (!next) return;
		textEdit = next;
	}

	function commitEdit(value: string) {
		if (!engine) return;
		engine.commit_text_edit(value);
		textEdit = null;
		engine.render();
		onStatus();
	}

	function cancelEdit() {
		if (!engine) return;
		engine.cancel_text_edit();
		textEdit = null;
		engine.render();
		onStatus();
	}

	function down(e: PointerEvent) {
		if (!engine || !canvas || textEdit) return;
		canvas.setPointerCapture(e.pointerId);
		const p = local(e);
		engine.pointer_down(p.x, p.y, e.shiftKey, space || e.button === 1);
		if (e.detail >= 2) {
			engine.pointer_up(p.x, p.y);
		}
		onStatus();
		engine.render();
	}
	function move(e: PointerEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		engine.pointer_move(p.x, p.y);
		onStatus();
		engine.render();
	}
	function up(e: PointerEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		engine.pointer_up(p.x, p.y);
		onStatus();
		engine.render();
	}
	function wheel(e: WheelEvent) {
		if (!engine || !canvas) return;
		e.preventDefault();
		const r = canvas.getBoundingClientRect();
		const scale = dpr();
		engine.wheel((e.clientX - r.left) * scale, (e.clientY - r.top) * scale, e.deltaY);
		engine.render();
		syncEditPos();
		onStatus();
	}

	function dblclick(e: MouseEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		openEdit(engine.dblclick(p.x, p.y));
		engine.render();
		onStatus();
	}

	function onCtx(e: MouseEvent) {
		e.preventDefault();
		if (textEdit) return;
		onContextMenu?.(e.clientX, e.clientY);
	}

	onMount(() => {
		if (!wrap) return;
		const ro = new ResizeObserver(() => {
			resizeCanvas();
			syncEditPos();
		});
		ro.observe(wrap);
		return () => ro.disconnect();
	});

	$effect(() => {
		const eng = engine;
		const el = canvas;
		if (!eng || !el) return;
		eng.attach_canvas(el);
		untrack(() => resizeCanvas());
	});

	const overlay = $derived.by(() => {
		if (!textEdit) return null;
		const scale = dpr();
		return {
			x: textEdit.screenX / scale,
			y: textEdit.screenY / scale,
			fontSize: Math.max(8, (textEdit.sy * textEdit.zoom) / scale),
			charWidth: Math.max(4, (textEdit.sx * textEdit.zoom) / scale),
			angle: textEdit.angle,
			italic: (textEdit.style & 2) !== 0,
			mirrored: (textEdit.style & 4) !== 0,
			text: textEdit.text
		};
	});
</script>

<svelte:window
	onkeydown={(e) => {
		if (textEdit) return;
		if (e.code === 'Space') space = true;
		if (e.altKey && e.key === 'Enter' && engine) {
			e.preventDefault();
			openEdit(engine.begin_selected_text_edit());
			engine.render();
			onStatus();
		}
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
		ondblclick={dblclick}
		onwheel={wheel}
		oncontextmenu={onCtx}
	></canvas>
	{#if overlay}
		<TextEditor
			text={overlay.text}
			x={overlay.x}
			y={overlay.y}
			fontSize={overlay.fontSize}
			charWidth={overlay.charWidth}
			angle={overlay.angle}
			italic={overlay.italic}
			mirrored={overlay.mirrored}
			onCommit={commitEdit}
			onCancel={cancelEdit}
		/>
	{/if}
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
