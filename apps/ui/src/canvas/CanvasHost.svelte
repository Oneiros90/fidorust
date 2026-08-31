<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { getAppSession } from '../app/appContext';
	import { canvasLocal, dpr } from '../lib/canvasCoords';
	import { parseEdit, textOverlayLayout, type TextEdit } from '../lib/textEdit';
	import TextEditor from './TextEditor.svelte';

	const app = getAppSession();
	let engine = $derived(app.engine);

	let canvas: HTMLCanvasElement | undefined;
	let wrap: HTMLDivElement | undefined;
	let space = $state(false);
	let textEdit = $state.raw<TextEdit | null>(null);

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
		return canvasLocal(canvas, e.clientX, e.clientY);
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
		app.refresh();
	}

	function cancelEdit() {
		if (!engine) return;
		engine.cancel_text_edit();
		textEdit = null;
		engine.render();
		app.refresh();
	}

	function down(e: PointerEvent) {
		if (!engine || !canvas || textEdit) return;
		canvas.setPointerCapture(e.pointerId);
		const p = local(e);
		engine.pointer_down(p.x, p.y, e.shiftKey, space || e.button === 1);
		if (e.detail >= 2) {
			engine.pointer_up(p.x, p.y);
		}
		app.refresh();
		engine.render();
	}

	function move(e: PointerEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		engine.pointer_move(p.x, p.y);
		app.refresh();
		engine.render();
	}

	function up(e: PointerEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		engine.pointer_up(p.x, p.y);
		app.refresh();
		engine.render();
	}

	function wheel(e: WheelEvent) {
		if (!engine || !canvas) return;
		e.preventDefault();
		const p = canvasLocal(canvas, e.clientX, e.clientY);
		engine.wheel(p.x, p.y, e.deltaY);
		engine.render();
		syncEditPos();
		app.refresh();
	}

	function dblclick(e: MouseEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		openEdit(engine.dblclick(p.x, p.y));
		engine.render();
		app.refresh();
	}

	function onCtx(e: MouseEvent) {
		e.preventDefault();
		if (textEdit) return;
		app.openContextMenu(e.clientX, e.clientY);
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

	const overlay = $derived.by(() => (textEdit ? textOverlayLayout(textEdit, dpr()) : null));
</script>

<svelte:window
	onkeydown={(e) => {
		if (textEdit) return;
		if (e.code === 'Space') space = true;
		if (e.altKey && e.key === 'Enter' && engine) {
			e.preventDefault();
			openEdit(engine.begin_selected_text_edit());
			engine.render();
			app.refresh();
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
