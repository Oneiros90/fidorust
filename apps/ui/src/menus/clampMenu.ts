export const MENU_VIEW_PAD = 8;

export type MenuAnchorSide = 'left' | 'right' | 'below';

export type MenuBox = {
	left: number;
	top: number;
	width: number;
	height: number;
	maxW: number;
	maxH: number;
};

function clamp(n: number, min: number, max: number): number {
	return Math.min(Math.max(min, n), Math.max(min, max));
}

export function viewportSize(): { vw: number; vh: number; maxW: number; maxH: number } {
	const vw = window.innerWidth;
	const vh = window.innerHeight;
	return {
		vw,
		vh,
		maxW: Math.max(0, vw - MENU_VIEW_PAD * 2),
		maxH: Math.max(0, vh - MENU_VIEW_PAD * 2)
	};
}

export function clampMenuOrigin(x: number, y: number, width: number, height: number): MenuBox {
	const { vw, vh, maxW, maxH } = viewportSize();
	const w = Math.min(Math.max(0, width), maxW);
	const h = Math.min(Math.max(0, height), maxH);
	return {
		left: clamp(x, MENU_VIEW_PAD, vw - w - MENU_VIEW_PAD),
		top: clamp(y, MENU_VIEW_PAD, vh - h - MENU_VIEW_PAD),
		width: w,
		height: h,
		maxW,
		maxH
	};
}

export function anchoredMenuBox(
	anchor: Pick<DOMRect, 'left' | 'right' | 'top' | 'bottom'>,
	size: { width: number; height: number },
	side: MenuAnchorSide
): MenuBox {
	const { vw, vh, maxW, maxH } = viewportSize();
	const width = Math.min(Math.max(0, size.width), maxW);
	const height = Math.min(Math.max(0, size.height), maxH);
	const pad = MENU_VIEW_PAD;

	let left: number;
	let top: number;

	if (side === 'below') {
		left = anchor.left;
		const spaceBelow = vh - pad - anchor.bottom;
		const spaceAbove = anchor.top - pad;
		top = height <= spaceBelow || spaceBelow >= spaceAbove ? anchor.bottom : anchor.top - height;
	} else {
		top = anchor.top;
		const spaceRight = vw - pad - anchor.right;
		const spaceLeft = anchor.left - pad;
		const fitsRight = width <= spaceRight;
		const fitsLeft = width <= spaceLeft;
		const preferRight = side === 'right';
		const useRight = preferRight
			? fitsRight || (!fitsLeft && spaceRight >= spaceLeft)
			: !fitsLeft && (fitsRight || spaceRight > spaceLeft);
		left = useRight ? anchor.right : anchor.left - width;
	}

	return {
		left: clamp(left, pad, vw - width - pad),
		top: clamp(top, pad, vh - height - pad),
		width,
		height,
		maxW,
		maxH
	};
}

/** Position a menu with `position: fixed` so it stays fully on-screen. */
export function placeMenu(el: HTMLElement, anchor: DOMRect, side: MenuAnchorSide): void {
	const { maxW, maxH } = viewportSize();
	el.style.maxWidth = `${maxW}px`;
	el.style.maxHeight = `${maxH}px`;

	const box = anchoredMenuBox(anchor, { width: el.offsetWidth, height: el.offsetHeight }, side);
	el.style.position = 'fixed';
	el.style.left = `${box.left}px`;
	el.style.top = `${box.top}px`;
	// Overflow other than visible clips nested position:fixed flyouts.
	const nested = el.querySelector('.sub');
	el.style.overflowY = !nested && el.scrollHeight > box.maxH ? 'auto' : 'visible';
	el.style.visibility = 'visible';
}

export function attachAnchoredMenu(getSide: () => MenuAnchorSide) {
	return (node: HTMLElement) => {
		const host = node.parentElement;
		const getAnchor = () => host?.querySelector<HTMLElement>(':scope > button') ?? undefined;

		const place = () => {
			if (getComputedStyle(node).display === 'none') {
				node.style.visibility = 'hidden';
				return;
			}
			if (node.offsetWidth === 0 && node.offsetHeight === 0) return;
			const anchor = getAnchor();
			if (!anchor) return;
			placeMenu(node, anchor.getBoundingClientRect(), getSide());
		};

		const onShow = () => {
			place();
			requestAnimationFrame(place);
		};

		const ro = new ResizeObserver(onShow);
		ro.observe(node);
		window.addEventListener('resize', onShow);
		document.addEventListener('scroll', onShow, true);
		host?.addEventListener('mouseenter', onShow);
		host?.addEventListener('focusin', onShow);
		queueMicrotask(onShow);

		return () => {
			ro.disconnect();
			window.removeEventListener('resize', onShow);
			document.removeEventListener('scroll', onShow, true);
			host?.removeEventListener('mouseenter', onShow);
			host?.removeEventListener('focusin', onShow);
		};
	};
}
