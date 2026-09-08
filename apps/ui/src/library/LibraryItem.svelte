<script lang="ts">
	import type { Engine } from '../app/engine.svelte';
	import { getAppSession } from '../app/appContext';
	import { parseSvgElement } from '../lib/attachSvg';
	import { PREVIEW_ROOT_MARGIN, THUMB_SIZE } from '../lib/constants';
	import { componentFullName } from '../lib/libraryDrag';

	let {
		engine,
		stem,
		componentKey,
		label,
		description,
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
		description: string;
		writable: boolean;
		selected: boolean;
		theme: 'light' | 'dark';
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
	let skipDescCommit = false;

	const attachPreview = $derived.by(() => {
		const eng = engine;
		const n = name;
		const themeKey = theme;
		const rev = eng?.libsRev ?? 0;
		return (node: HTMLElement) => {
			if (!eng) return;
			node.dataset.theme = themeKey;
			node.dataset.rev = String(rev);
			const io = new IntersectionObserver(
				(entries) => {
					if (!entries.some((e) => e.isIntersecting)) return;
					const raw = eng.query((wasm) => wasm.component_preview_svg(n));
					const el = raw ? parseSvgElement(raw) : null;
					if (el) node.replaceChildren(el);
					io.disconnect();
				},
				{ rootMargin: PREVIEW_ROOT_MARGIN }
			);
			io.observe(node);
			return () => io.disconnect();
		};
	});

	function focusAndSelect(node: HTMLInputElement) {
		queueMicrotask(() => {
			node.focus();
			node.select();
		});
	}

	function pick() {
		onPick(stem, componentKey);
		app.libraryFocus = { stem, key: componentKey };
	}

	function onDblClick(e: MouseEvent) {
		if (!writable) return;
		const t = e.target as HTMLElement;
		if (t.closest('.desc')) {
			e.stopPropagation();
			app.beginEditComponentDescription(stem, componentKey);
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

	function commitDescription(value: string) {
		if (skipDescCommit) {
			skipDescCommit = false;
			app.editingLibraryField = null;
			return;
		}
		app.setComponentDescription(stem, componentKey, value);
	}

	function cancelName() {
		skipNameCommit = true;
		app.editingLibraryField = null;
	}

	function cancelDescription() {
		skipDescCommit = true;
		app.editingLibraryField = null;
	}
</script>

<div
	class={['item', { selected }]}
	role="button"
	tabindex="0"
	aria-pressed={selected}
	title={description ? `${label} — ${description}` : label}
	onclick={pick}
	ondblclick={onDblClick}
	onkeydown={onKey}
	onpointerdown={onPointerDown}
	oncontextmenu={onContext}
	style:--thumb-size="{THUMB_SIZE}px"
>
	<div class="thumb" {@attach attachPreview}></div>
	<span class="meta">
		<span class="key">{componentKey}</span>
		{#if editing === 'name'}
			<input
				class="name-input"
				value={label}
				{@attach focusAndSelect}
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
		{#if editing === 'description'}
			<input
				class="desc-input"
				value={description}
				{@attach focusAndSelect}
				onpointerdown={(e) => e.stopPropagation()}
				onclick={(e) => e.stopPropagation()}
				onblur={(e) => commitDescription(e.currentTarget.value)}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						e.preventDefault();
						e.currentTarget.blur();
					}
					if (e.key === 'Escape') {
						e.preventDefault();
						cancelDescription();
					}
				}}
			/>
		{:else if description || writable}
			<span class={['desc', { empty: !description }]}>{description}</span>
		{/if}
	</span>
</div>

<style>
	.item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
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
		width: var(--thumb-size);
		height: var(--thumb-size);
		flex-shrink: 0;
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
		width: var(--thumb-size);
		height: var(--thumb-size);
	}
	.meta {
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
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
	.desc {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 11px;
		color: var(--fg-muted);
		min-height: 1.2em;
	}
	.desc.empty {
		opacity: 0.45;
	}
	.name-input,
	.desc-input {
		width: 100%;
		min-width: 0;
		padding: 1px 4px;
		font: inherit;
		font-size: inherit;
	}
	.desc-input {
		font-size: 11px;
	}
</style>
