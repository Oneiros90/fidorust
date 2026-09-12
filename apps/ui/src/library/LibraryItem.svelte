<script lang="ts">
	import type { Engine } from '../app/engine.svelte';
	import { getAppSession } from '../app/appContext';
	import type { Theme } from '../app/types';
	import { parseSvgElement } from '../lib/attachSvg';
	import { PREVIEW_ROOT_MARGIN, THUMB_SIZE } from '../lib/constants';
	import { componentFullName } from '../lib/libraryDrag';
	import { focusAndSelectDeferred } from '../lib/focus';

	let {
		engine,
		stem,
		componentKey,
		label,
		origin = '',
		writable,
		selected,
		theme,
		onPick,
		onArmDrag
	}: {
		engine: Engine | null;
		stem: string;
		componentKey: string;
		label: string;
		origin?: string;
		writable: boolean;
		selected: boolean;
		theme: Theme;
		onPick: (stem: string, key: string) => void;
		onArmDrag: (name: string, e: PointerEvent) => void;
	} = $props();

	const app = getAppSession();
	const name = $derived(componentFullName(stem, componentKey));
	const editing = $derived(
		app.editingLibraryField?.stem === stem && app.editingLibraryField?.key === componentKey
			? app.editingLibraryField.field
			: null
	);

	let skipNameCommit = false;
	let skipKeyCommit = false;

	const previewSvg = $derived(app.previewCache.get(theme, name));

	const attachPreview = $derived.by(() => {
		const eng = engine;
		const n = name;
		const themeKey = theme;
		const raw = previewSvg;
		void (eng?.libsRev ?? 0);
		return (node: HTMLElement) => {
			if (raw) {
				const el = parseSvgElement(raw);
				if (el) node.replaceChildren(el);
				return;
			}
			if (!eng) return;
			const io = new IntersectionObserver(
				(entries) => {
					if (!entries.some((e) => e.isIntersecting)) return;
					app.previewCache.request(eng, themeKey, n);
					io.disconnect();
				},
				{ rootMargin: PREVIEW_ROOT_MARGIN }
			);
			io.observe(node);
			return () => io.disconnect();
		};
	});

	function pick() {
		onPick(stem, componentKey);
		app.libraryFocus = { stem, key: componentKey };
	}

	function onDblClick(e: MouseEvent) {
		if (!writable) return;
		const t = e.target as HTMLElement;
		if (t.closest('.key')) {
			e.stopPropagation();
			app.beginEditComponentKey(stem, componentKey);
		} else if (t.closest('.label')) {
			e.stopPropagation();
			app.beginRenameComponent(stem, componentKey);
		}
	}

	function onPointerDown(e: PointerEvent) {
		if (e.button !== 0) return;
		if (e.target instanceof HTMLInputElement) return;
		pick();
		onArmDrag(name, e);
	}

	function onKey(e: KeyboardEvent) {
		if (editing) return;
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			pick();
			return;
		}
		if (e.key === 'Delete' && writable) {
			e.preventDefault();
			e.stopPropagation();
			app.requestDeleteComponent(stem, componentKey);
		}
	}

	function onContext(e: MouseEvent) {
		if (!writable) return;
		e.preventDefault();
		e.stopPropagation();
		pick();
		app.openLibraryItemContextMenu(e.clientX, e.clientY, stem, componentKey);
	}

	function commitName(value: string) {
		if (skipNameCommit) {
			skipNameCommit = false;
			app.editingLibraryField = null;
			return;
		}
		app.renameComponent(stem, componentKey, value);
	}

	function commitKey(value: string) {
		if (skipKeyCommit) {
			skipKeyCommit = false;
			app.editingLibraryField = null;
			return;
		}
		app.renameComponentKey(stem, componentKey, value);
	}

	function cancelName() {
		skipNameCommit = true;
		app.editingLibraryField = null;
	}

	function cancelKey() {
		skipKeyCommit = true;
		app.editingLibraryField = null;
	}
</script>

<div
	class="item"
	class:selected
	role="button"
	tabindex="0"
	aria-pressed={selected}
	title={origin ? `${label} — ${origin}` : label}
	onclick={pick}
	ondblclick={onDblClick}
	onkeydown={onKey}
	onpointerdown={onPointerDown}
	oncontextmenu={onContext}
>
	<div
		class="thumb"
		style:width="{THUMB_SIZE}px"
		style:height="{THUMB_SIZE}px"
		{@attach attachPreview}
	></div>
	<span class="meta">
		{#if editing === 'key'}
			<input
				class="key-input"
				value={componentKey}
				{@attach focusAndSelectDeferred}
				onpointerdown={(e) => e.stopPropagation()}
				onclick={(e) => e.stopPropagation()}
				onblur={(e) => commitKey(e.currentTarget.value)}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						e.preventDefault();
						e.currentTarget.blur();
					}
					if (e.key === 'Escape') {
						e.preventDefault();
						cancelKey();
					}
				}}
			/>
		{:else}
			<span class="key">{componentKey}</span>
		{/if}
		{#if editing === 'name'}
			<input
				class="name-input"
				value={label}
				{@attach focusAndSelectDeferred}
				onpointerdown={(e) => e.stopPropagation()}
				onclick={(e) => e.stopPropagation()}
				onblur={(e) => commitName(e.currentTarget.value)}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						e.preventDefault();
						e.currentTarget.blur();
					}
					if (e.key === 'Escape') {
						e.preventDefault();
						cancelName();
					}
				}}
			/>
		{:else}
			<span class="label">{label}</span>
		{/if}
		{#if origin}
			<span class="origin">{origin}</span>
		{/if}
	</span>
</div>

<style>
	.item {
		display: grid;
		grid-template-columns: 40px minmax(0, 1fr);
		align-items: center;
		column-gap: 8px;
		width: 100%;
		min-width: 0;
		text-align: left;
		border: 1px solid transparent;
		background: transparent;
		padding: 3px 6px;
		border-radius: 6px;
		font: inherit;
		font-size: 12px;
		line-height: 1.3;
		color: inherit;
		cursor: grab;
		user-select: none;
	}
	.item:hover {
		background: var(--bg-menu);
	}
	.item.selected {
		background: color-mix(in srgb, var(--accent) 22%, var(--bg-panel));
		border-color: var(--accent);
		box-shadow: inset 3px 0 0 var(--accent);
	}
	.item:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
	}
	.item:active {
		cursor: grabbing;
	}
	.thumb {
		width: 40px;
		height: 40px;
		min-width: 40px;
		min-height: 40px;
		max-width: 40px;
		max-height: 40px;
		border-radius: 4px;
		background: var(--canvas-bg);
		border: 1px solid var(--border);
		overflow: hidden;
		display: grid;
		place-items: center;
		pointer-events: none;
	}
	.thumb :global(svg) {
		display: block;
		width: 40px;
		height: 40px;
		max-width: 40px;
		max-height: 40px;
	}
	.meta {
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.key {
		font-family: var(--mono);
		font-size: 11px;
		color: var(--fg-muted);
	}
	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.origin {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.name-input,
	.key-input {
		width: 100%;
		min-width: 0;
		padding: 1px 4px;
		font: inherit;
		font-size: inherit;
	}
	.key-input {
		font-family: var(--mono);
		font-size: 11px;
	}
</style>
