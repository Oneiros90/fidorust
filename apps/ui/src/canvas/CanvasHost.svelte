<script lang="ts">
	import { untrack } from 'svelte';
	import { getAppSession } from '../app/appContext';
	import { canvasLocal, dpr } from '../lib/canvasCoords';
	import {
		dblClickOpensProperties,
		parseEdit,
		textOverlayLayout,
		type TextEdit
	} from '../lib/textEdit';
	import TextEditor from './TextEditor.svelte';

	const app = getAppSession();
	let engine = $derived(app.engine);

	let canvas: HTMLCanvasElement | undefined;
	let wrap: HTMLDivElement | undefined;
	let space = $state(false);
	let panning = $state(false);
	let textEdit = $state.raw<TextEdit | null>(null);

	const canvasCursor = $derived(
		panning
			? 'grabbing'
			: space || app.status.tool === 'pan'
				? 'grab'
				: app.status.duplicate_drag
					? 'copy'
					: app.status.tool === 'select'
						? app.status.hover_hit
							? 'pointer'
							: 'default'
						: 'crosshair'
	);

	function copyMod(e: { ctrlKey: boolean; metaKey: boolean }) {
		return e.ctrlKey || e.metaKey;
	}

	function syncDuplicate(e: { ctrlKey: boolean; metaKey: boolean }) {
		if (!engine) return;
		engine.mutate(
			(wasm) => {
				wasm.set_move_duplicate(copyMod(e));
			},
			{ refreshFirst: true }
		);
	}

	let lastStampMs = 0;

	function tryStamp(): boolean {
		if (!engine || textEdit) return false;
		const now = performance.now();
		if (now - lastStampMs < 40) return false;
		let stamped = false;
		engine.mutate(
			(wasm) => {
				stamped = wasm.stamp_drag_copy();
			},
			{ refreshFirst: true }
		);
		if (stamped) lastStampMs = now;
		return stamped;
	}

	function onMiddleDown(e: MouseEvent) {
		if (e.button !== 1) return;
		if (tryStamp()) e.preventDefault();
	}

	function resizeCanvas() {
		if (!canvas || !wrap || !engine) return;
		const r = wrap.getBoundingClientRect();
		const scale = dpr();
		canvas.width = Math.max(1, Math.floor(r.width * scale));
		canvas.height = Math.max(1, Math.floor(r.height * scale));
		canvas.style.width = `${r.width}px`;
		canvas.style.height = `${r.height}px`;
		engine.query((wasm) => {
			wasm.resize(canvas!.width, canvas!.height);
			wasm.render();
		});
	}

	function local(e: { clientX: number; clientY: number }) {
		if (!canvas) return { x: 0, y: 0 };
		return canvasLocal(canvas, e.clientX, e.clientY);
	}

	function syncEditPos() {
		const current = untrack(() => textEdit);
		if (!current || !engine) return;
		try {
			const p = JSON.parse(
				engine.query((wasm) => wasm.world_to_screen_json(current.wx, current.wy))
			) as {
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
		engine.query((wasm) => {
			wasm.commit_text_edit(value);
		});
		textEdit = null;
		engine.mutate(() => {});
	}

	function cancelEdit() {
		if (!engine) return;
		engine.query((wasm) => {
			wasm.cancel_text_edit();
		});
		textEdit = null;
		engine.mutate(() => {});
	}

	function down(e: PointerEvent) {
		if (e.button === 2) {
			e.preventDefault();
			return;
		}
		if (!engine || !canvas || textEdit) return;
		canvas.setPointerCapture(e.pointerId);
		if (space || e.button === 1 || app.status.tool === 'pan') panning = true;
		const p = local(e);
		engine.mutate(
			(wasm) => {
				wasm.pointer_down(p.x, p.y, e.shiftKey, space || e.button === 1);
				wasm.set_move_duplicate(copyMod(e));
				if (e.detail >= 2) {
					wasm.pointer_up(p.x, p.y);
				}
			},
			{ refreshFirst: true }
		);
	}

	function move(e: PointerEvent) {
		if (!engine || textEdit) return;
		if (e.button === 1 && (e.buttons & 4) !== 0) {
			e.preventDefault();
			tryStamp();
		}
		const p = local(e);
		engine.mutate(
			(wasm) => {
				wasm.set_move_duplicate(copyMod(e));
				wasm.pointer_move(p.x, p.y);
			},
			{ refreshFirst: true }
		);
	}

	function leave(e: PointerEvent) {
		if (!engine || textEdit || panning || e.buttons !== 0) return;
		engine.mutate(
			(wasm) => {
				wasm.clear_hover();
			},
			{ refreshFirst: true }
		);
	}

	function up(e: PointerEvent) {
		if (e.button === 2 || !engine || textEdit) return;
		if (e.button === 1) {
			e.preventDefault();
			if (!panning) return;
		}
		panning = false;
		const p = local(e);
		engine.mutate(
			(wasm) => {
				wasm.set_move_duplicate(copyMod(e));
				wasm.pointer_up(p.x, p.y);
			},
			{ refreshFirst: true }
		);
	}

	function wheel(e: WheelEvent) {
		if (!engine || !canvas) return;
		e.preventDefault();
		const p = canvasLocal(canvas, e.clientX, e.clientY);
		engine.query((wasm) => {
			wasm.wheel(p.x, p.y, e.deltaY);
			wasm.render();
		});
		syncEditPos();
		engine.refresh();
	}

	function dblclick(e: MouseEvent) {
		if (!engine || textEdit) return;
		const p = local(e);
		let raw = 'null';
		engine.mutate((wasm) => {
			raw = wasm.dblclick(p.x, p.y);
			openEdit(raw);
		});
		if (dblClickOpensProperties(raw)) app.openProperties();
	}

	function onCtx(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		if (!engine || textEdit) return;
		const p = local(e);
		if (engine.query((wasm) => wasm.pointer_right(p.x, p.y))) {
			engine.mutate(() => {});
			return;
		}
		engine.mutate((wasm) => {
			wasm.prepare_context_menu(p.x, p.y);
		});
		app.openContextMenu(e.clientX, e.clientY);
	}

	function attachWrap(node: HTMLDivElement) {
		wrap = node;
		const ro = new ResizeObserver(() => {
			resizeCanvas();
			syncEditPos();
		});
		ro.observe(node);
		return () => {
			ro.disconnect();
			if (wrap === node) wrap = undefined;
		};
	}

	const attachCanvas = $derived.by(() => {
		const eng = engine;
		return (node: HTMLCanvasElement) => {
			canvas = node;
			const cleanup = eng?.attachCanvas(node);
			untrack(() => resizeCanvas());
			return () => {
				cleanup?.();
				if (canvas === node) canvas = undefined;
			};
		};
	});

	const overlay = $derived.by(() => (textEdit ? textOverlayLayout(textEdit, dpr()) : null));
</script>

<svelte:window
	onkeydown={(e) => {
		if (textEdit) return;
		if (e.code === 'Space') space = true;
		if (e.key === 'Control' || e.key === 'Meta') syncDuplicate(e);
		if (e.altKey && e.key === 'Enter' && engine) {
			e.preventDefault();
			engine.mutate((wasm) => {
				openEdit(wasm.begin_selected_text_edit());
			});
		}
	}}
	onkeyup={(e) => {
		if (e.code === 'Space') space = false;
		if (e.key === 'Control' || e.key === 'Meta') syncDuplicate(e);
	}}
	onmousedowncapture={onMiddleDown}
/>

<div class="wrap" {@attach attachWrap}>
	<canvas
		{@attach attachCanvas}
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
		onpointerleave={leave}
		onpointercancel={() => (panning = false)}
		onmousedown={onMiddleDown}
		onauxclick={(e) => {
			if (e.button === 1) e.preventDefault();
		}}
		ondblclick={dblclick}
		onwheel={wheel}
		oncontextmenu={onCtx}
		style:cursor={canvasCursor}
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
	}
</style>
