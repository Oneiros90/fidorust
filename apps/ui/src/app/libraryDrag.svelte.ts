import type { App as WasmApp } from '../wasm/fidorust_wasm.js';
import { canvasLocal, cssPerLu } from '../lib/canvasCoords';
import { DRAG_THRESHOLD_PX } from '../lib/constants';
import { parseComponentCursor } from '../lib/libraryDrag';
import type { ComponentCursor } from './engineTypes';
import type { AppSession } from './appSession.svelte';

export class LibraryDragSession {
	constructor(private s: AppSession) {}

	arm = (name: string, e: PointerEvent) => {
		const s = this.s;
		const pointerId = e.pointerId;
		const x0 = e.clientX;
		const y0 = e.clientY;
		let active = false;
		let rot = 0;
		const ac = new AbortController();
		const { signal } = ac;

		const finish = () => {
			ac.abort();
			document.body.classList.remove('lib-dragging');
			s.libGhost = null;
			s.engine?.query((app) => {
				app.set_pending_follow(false);
				app.clear_hover();
				app.render();
			});
		};

		const move = (ev: PointerEvent) => {
			if (ev.pointerId !== pointerId || !s.engine) return;
			if (!active) {
				if (Math.hypot(ev.clientX - x0, ev.clientY - y0) < DRAG_THRESHOLD_PX) return;
				active = true;
				document.body.classList.add('lib-dragging');
				s.engine.query((app) => {
					app.set_pending_follow(true);
				});
			}
			const canvas = s.engine.canvas;
			if (canvas) {
				const loc = canvasLocal(canvas, ev.clientX, ev.clientY);
				if (loc.inside) {
					s.engine.query((app) => {
						app.pointer_move(loc.x, loc.y);
						app.render();
					});
					s.libGhost = null;
					return;
				}
				s.engine.query((app) => {
					app.clear_hover();
					app.render();
				});
			}
			const cur = getCursor(s, name);
			if (!cur) {
				s.libGhost = null;
				return;
			}
			s.libGhost = {
				...cur,
				x: ev.clientX,
				y: ev.clientY,
				scale: cssPerLu(s.status.zoom),
				rot
			};
		};

		const stop = (ev: PointerEvent) => {
			if (ev.pointerId !== pointerId || ev.button !== 0) return;
			const wasActive = active;
			finish();
			if (!wasActive || !s.engine) return;
			const canvas = s.engine.canvas;
			if (!canvas) return;
			const loc = canvasLocal(canvas, ev.clientX, ev.clientY);
			if (loc.inside) {
				s.engine.mutate((app) => {
					app.place_component_at(name, loc.x, loc.y);
				});
				s.libraryFocus = null;
			} else {
				s.engine.query((app) => {
					app.clear_hover();
					app.render();
				});
			}
		};

		const onCtx = (ev: MouseEvent) => {
			if (!active) return;
			ev.preventDefault();
			ev.stopPropagation();
			if (!s.engine) return;
			s.engine.query((app) => {
				app.pointer_right(0, 0);
				app.render();
			});
			rot = (rot + 1) % 4;
			if (s.libGhost) s.libGhost = { ...s.libGhost, rot };
		};

		const onEsc = (ke: KeyboardEvent) => {
			if (ke.key !== 'Escape') return;
			ke.preventDefault();
			finish();
			s.engine?.query((app) => {
				app.clear_hover();
				app.render();
			});
		};

		window.addEventListener('pointermove', move, { signal });
		window.addEventListener('pointerup', stop, { signal });
		window.addEventListener('pointercancel', stop, { signal });
		window.addEventListener('keydown', onEsc, { signal });
		window.addEventListener('contextmenu', onCtx, { capture: true, signal });
	};
}

export function getCursor(s: AppSession, name: string): ComponentCursor | null {
	if (!s.engine) return null;
	const key = `${s.theme}:${name}`;
	let c = s.cursorCache.get(key);
	if (!c) {
		const parsed = parseComponentCursor(
			s.engine.query((app: WasmApp) => app.component_cursor_json(name))
		);
		if (!parsed) return null;
		s.cursorCache.set(key, parsed);
		c = parsed;
	}
	return c;
}
