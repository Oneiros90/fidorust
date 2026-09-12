import { DRAG_THRESHOLD_PX } from '../lib/constants';
import type { Engine } from '../app/engine.svelte';
import type { AppSession } from '../app/appSession.svelte';

export const STAMP_DEBOUNCE_MS = 40;

export type RightGesture =
	| {
			kind: 'pending';
			startX: number;
			startY: number;
			clientX: number;
			clientY: number;
			shift: boolean;
	  }
	| { kind: 'marquee'; x: number; y: number; clientX: number; clientY: number };

export function copyMod(e: { ctrlKey: boolean; metaKey: boolean }) {
	return e.ctrlKey || e.metaKey;
}

export function syncDuplicate(engine: Engine | null, e: { ctrlKey: boolean; metaKey: boolean }) {
	if (!engine) return;
	engine.mutate(
		(wasm) => {
			wasm.set_move_duplicate(copyMod(e));
		},
		{ refreshFirst: true }
	);
}

export function makeStamp() {
	let lastStampMs = 0;
	return function tryStamp(engine: Engine | null, textEdit: unknown): boolean {
		if (!engine || textEdit) return false;
		const now = performance.now();
		if (now - lastStampMs < STAMP_DEBOUNCE_MS) return false;
		let stamped = false;
		engine.mutate(
			(wasm) => {
				stamped = wasm.stamp_drag_copy();
			},
			{ refreshFirst: true }
		);
		if (stamped) lastStampMs = now;
		return stamped;
	};
}

export function isPanInput(space: boolean, button: number, tool: string): boolean {
	return space || button === 1 || tool === 'pan';
}

export function commitRightMarquee(
	engine: Engine,
	sx: number,
	sy: number,
	clientX: number,
	clientY: number,
	app: AppSession
) {
	engine.mutate(
		(wasm) => {
			wasm.pointer_up(sx, sy);
		},
		{ refreshFirst: true }
	);
	app.openContextMenu(clientX, clientY);
}

export function abortRightMarquee(engine: Engine | null, gesture: RightGesture | null) {
	if (gesture?.kind === 'marquee' && engine) {
		const { x, y } = gesture;
		engine.mutate(
			(wasm) => {
				wasm.pointer_up(x, y);
			},
			{ refreshFirst: true }
		);
	}
}

export function maybeBeginRightMarquee(
	engine: Engine,
	gesture: RightGesture,
	e: PointerEvent,
	p: { x: number; y: number },
	tool: string
): RightGesture | null {
	if (gesture.kind !== 'pending' || (e.buttons & 2) === 0) return null;
	const dx = e.clientX - gesture.clientX;
	const dy = e.clientY - gesture.clientY;
	if (dx * dx + dy * dy < DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) return null;
	if (tool !== 'select') return null;
	engine.mutate(
		(wasm) => {
			wasm.begin_marquee(gesture.startX, gesture.startY, gesture.shift);
			wasm.set_move_duplicate(copyMod(e));
			wasm.pointer_move(p.x, p.y);
		},
		{ refreshFirst: true }
	);
	return {
		kind: 'marquee',
		x: p.x,
		y: p.y,
		clientX: e.clientX,
		clientY: e.clientY
	};
}
