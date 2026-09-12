/** Immediate focus+select (canvas text overlay, already in the DOM). */
export function focusAndSelect(node: HTMLInputElement) {
	node.focus();
	node.select();
}

/** Deferred focus+select (inline rename inputs attached during render). */
export function focusAndSelectDeferred(node: HTMLInputElement) {
	queueMicrotask(() => {
		node.focus();
		node.select();
	});
}
