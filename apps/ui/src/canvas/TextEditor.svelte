<script lang="ts">
	import { untrack } from 'svelte';

	let {
		text,
		x,
		y,
		fontSize,
		charWidth,
		angle,
		italic,
		mirrored,
		onCommit,
		onCancel
	}: {
		text: string;
		x: number;
		y: number;
		fontSize: number;
		charWidth: number;
		angle: number;
		italic: boolean;
		mirrored: boolean;
		onCommit: (value: string) => void;
		onCancel: () => void;
	} = $props();

	let value = $state(untrack(() => text));
	let closed = false;

	function commit() {
		if (closed) return;
		closed = true;
		onCommit(value);
	}

	function cancel() {
		if (closed) return;
		closed = true;
		onCancel();
	}

	const letterSpacing = $derived(charWidth - fontSize * 0.6);
	const width = $derived(Math.max(1, [...value].length + 1) * charWidth + 4);
	const transform = $derived(`rotate(${angle}deg) scaleX(${mirrored ? -1 : 1})`);

	function focusAndSelect(node: HTMLInputElement) {
		node.focus();
		node.select();
	}
</script>

<input
	{@attach focusAndSelect}
	bind:value
	class="scene-text"
	aria-label="Edit text"
	style:left="{x}px"
	style:top="{y}px"
	style:width="{width}px"
	style:font-size="{fontSize}px"
	style:letter-spacing="{letterSpacing}px"
	style:font-style={italic ? 'italic' : 'normal'}
	style:transform
	onkeydown={(e) => {
		e.stopPropagation();
		if (e.key === 'Enter') {
			e.preventDefault();
			commit();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancel();
		}
	}}
	onblur={commit}
/>

<style>
	.scene-text {
		position: absolute;
		z-index: 6;
		margin: 0;
		padding: 0 2px;
		border: 1px dashed var(--accent);
		border-radius: 2px;
		background: var(--canvas-bg);
		color: var(--fg);
		font-family: var(--mono);
		line-height: 1;
		transform-origin: 0 0;
		caret-color: var(--accent);
	}
</style>
