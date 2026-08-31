<script lang="ts">
	import { onMount } from 'svelte';
	import CanvasHost from './CanvasHost.svelte';
	import ToolIcon from './ToolIcon.svelte';
	import { dict, type Locale } from './i18n';
	import type { App as WasmApp } from './wasm/fidocad_wasm.js';

	let engine = $state<WasmApp | null>(null);
	let locale = $state<Locale>((navigator.language.startsWith('en') ? 'en' : 'it') as Locale);
	let theme = $state<'light' | 'dark'>('light');
	let t = $derived(dict(locale));
	let status = $state({
		tool: 'select',
		layer: 0,
		x: 0,
		y: 0,
		xmm: 0,
		ymm: 0,
		zoom: 4,
		pcb: false,
		n: 0,
		selected: 0,
		can_undo: false,
		can_redo: false,
		title: '',
		snap: 5,
		grid: 5
	});
	let libs = $state<
		{ stem: string; title: string; categories: { name: string; macros: [string, string][] }[] }[]
	>([]);
	let layers = $state<{
		layers: { name: string; color: number[]; show: boolean; print: boolean }[];
	}>({ layers: [] });
	let showLayers = $state(false);
	let showAbout = $state(false);
	let showGrid = $state(true);
	let splitMacros = $state(true);
	let filled = $state(false);
	let menu = $state<string | null>(null);
	let error = $state('');
	let fileHandleName = $state('untitled.fcd');

	const tools = [
		['select', 'select'],
		['line', 'line'],
		['rect', 'rect'],
		['ellipse', 'ellipse'],
		['poly', 'poly'],
		['bezier', 'bezier'],
		['text', 'text'],
		['connection', 'connection'],
		['pcb-track', 'pcbTrack'],
		['pcb-pad', 'pcbPad'],
		['macro', 'macro'],
		['pan', 'pan']
	] as const;

	function refresh() {
		if (!engine) return;
		status = JSON.parse(engine.status_json());
		layers = JSON.parse(engine.layers_json());
	}

	function tool(id: string) {
		engine?.set_tool(id);
		if (id === 'text') {
			const txt = prompt(t.textPrompt, 'TEXT');
			if (txt) engine?.set_pending_text(txt);
		}
		engine?.render();
		refresh();
	}

	async function openSample() {
		if (!engine) return;
		const r = await fetch('/sample.fcd');
		const buf = await r.arrayBuffer();
		engine.load_fcd_bytes(new Uint8Array(buf));
		fileHandleName = 'sample.fcd';
		engine.render();
		refresh();
	}

	async function openFile() {
		const input = document.createElement('input');
		input.type = 'file';
		input.accept = '.fcd,.txt';
		input.onchange = async () => {
			const f = input.files?.[0];
			if (!f || !engine) return;
			const buf = await f.arrayBuffer();
				try {
					engine.load_fcd_bytes(new Uint8Array(buf));
				fileHandleName = f.name;
				engine.render();
				refresh();
			} catch (e) {
				error = String(e);
			}
		};
		input.click();
	}

	function download(name: string, content: string, mime: string) {
		const blob = new Blob([content], { type: mime });
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = name;
		a.click();
		URL.revokeObjectURL(a.href);
	}

	function saveFile() {
		if (!engine) return;
		download(fileHandleName.endsWith('.fcd') ? fileHandleName : 'drawing.fcd', engine.save_fcd(), 'text/plain');
	}

	function exportSvg() {
		if (!engine) return;
		download(fileHandleName.replace(/\.fcd$/i, '') + '.svg', engine.export_svg(), 'image/svg+xml');
	}

	function exportPng() {
		const canvas = document.querySelector('canvas');
		if (!canvas) return;
		canvas.toBlob((b) => {
			if (!b) return;
			const a = document.createElement('a');
			a.href = URL.createObjectURL(b);
			a.download = fileHandleName.replace(/\.fcd$/i, '') + '.png';
			a.click();
		});
	}

	function printDoc() {
		const canvas = document.querySelector('canvas');
		if (!canvas) return;
		const w = window.open('');
		if (!w) return;
		w.document.write(`<img src="${canvas.toDataURL('image/png')}" style="max-width:100%">`);
		w.document.close();
		w.focus();
		w.print();
	}

	async function copyFcd() {
		if (!engine) return;
		await navigator.clipboard.writeText(engine.clipboard_fcd());
	}

	async function pasteFcd() {
		if (!engine) return;
		const text = await navigator.clipboard.readText();
		if (text.includes('FIDOCAD') || text.includes('LI ') || text.includes('MC ')) {
			engine.load_fcd(text);
			engine.render();
			refresh();
		}
	}

	function pickMacro(stem: string, key: string) {
		const name = stem === 'stdlib' ? key : `${stem}.${key}`;
		engine?.set_pending_macro(name);
		engine?.render();
		refresh();
	}

	function applyTheme() {
		document.documentElement.dataset.theme = theme;
		engine?.set_theme(theme);
		engine?.render();
	}

	onMount(async () => {
		applyTheme();
		const init = (await import('./wasm/fidocad_wasm.js')).default;
		const { App } = await import('./wasm/fidocad_wasm.js');
		await init();
		engine = new App();
		libs = JSON.parse(engine.library_json());
		engine.set_locale(locale);
		engine.set_theme(theme);
		refresh();
		const onKey = (e: KeyboardEvent) => {
			const meta = e.metaKey || e.ctrlKey;
			if (engine?.key(e.key, meta)) {
				e.preventDefault();
				engine.render();
				refresh();
			}
			if (meta && e.key.toLowerCase() === 'o') {
				e.preventDefault();
				openFile();
			}
			if (meta && e.key.toLowerCase() === 's') {
				e.preventDefault();
				saveFile();
			}
			if (e.altKey && e.key === 'Enter') {
				const props = engine ? JSON.parse(engine.selection_props_json()) : null;
				if (props?.Text) {
					const n = prompt(t.textPrompt, props.Text.text);
					if (n) {
						engine?.set_pending_text(n);
						engine?.render();
					}
				}
			}
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	$effect(() => {
		document.documentElement.lang = locale;
		engine?.set_locale(locale);
	});
</script>

<div class="shell">
	<header class="menubar">
		<img class="brand-mark" src="/favicon.svg" width="22" height="22" alt="" aria-hidden="true" />
		<strong class="brand">{t.app}</strong>
		{#each [['file', t.file], ['edit', t.edit], ['view', t.view], ['options', t.options], ['help', t.help]] as [id, label]}
			<div class="menu">
				<button
					class="menu-btn"
					onclick={() => (menu = menu === id ? null : id)}
					aria-expanded={menu === id}>{label}</button
				>
				{#if menu === id}
					<div class="dropdown">
						{#if id === 'file'}
							<button onclick={() => { engine?.new_doc(); engine?.render(); refresh(); menu = null; }}>{t.new}</button>
							<button onclick={() => { openSample(); menu = null; }}>Esempio / Sample</button>
							<button onclick={() => { saveFile(); menu = null; }}>{t.save}</button>
							<button onclick={() => { exportSvg(); menu = null; }}>{t.exportSvg}</button>
							<button onclick={() => { exportPng(); menu = null; }}>{t.exportPng}</button>
							<button onclick={() => { printDoc(); menu = null; }}>{t.print}</button>
							<button onclick={() => { copyFcd(); menu = null; }}>{t.copyFcd}</button>
							<button onclick={() => { pasteFcd(); menu = null; }}>{t.pasteFcd}</button>
						{:else if id === 'edit'}
							<button disabled={!status.can_undo} onclick={() => { engine?.undo(); engine?.render(); refresh(); }}>{t.undo}</button>
							<button disabled={!status.can_redo} onclick={() => { engine?.redo(); engine?.render(); refresh(); }}>{t.redo}</button>
							<button onclick={() => { engine?.key('Delete', false); engine?.render(); refresh(); }}>{t.delete}</button>
							<button onclick={() => { engine?.key('a', true); engine?.render(); refresh(); }}>{t.selectAll}</button>
							<button onclick={() => { engine?.rotate(); engine?.render(); refresh(); }}>{t.rotate}</button>
							<button onclick={() => { engine?.mirror(); engine?.render(); refresh(); }}>{t.mirror}</button>
						{:else if id === 'view'}
							<button
								onclick={() => {
									showGrid = !showGrid;
									engine?.set_show_grid(showGrid);
									engine?.render();
								}}>{t.grid}</button
							>
							<button onclick={() => { engine?.fit(); engine?.render(); refresh(); }}>{t.fit}</button>
							<button
								onclick={() => {
									engine?.set_pcb_mode(!status.pcb);
									refresh();
								}}>{t.pcbMode}</button
							>
							<button onclick={() => { showLayers = true; menu = null; }}>{t.layers}</button>
						{:else if id === 'options'}
							<button
								onclick={() => {
									splitMacros = !splitMacros;
									engine?.set_split_macros(splitMacros);
								}}>{t.splitMacros}</button
							>
							<button
								onclick={() => {
									locale = locale === 'it' ? 'en' : 'it';
								}}>{t.language}: {locale.toUpperCase()}</button
							>
							<button
								onclick={() => {
									theme = theme === 'light' ? 'dark' : 'light';
									applyTheme();
								}}>{t.theme}: {theme === 'light' ? t.light : t.dark}</button
							>
						{:else}
							<button onclick={() => { showAbout = true; menu = null; }}>{t.about}</button>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
		<div class="grow"></div>
		<label>
			{t.layer}
			<select
				value={status.layer}
				onchange={(e) => {
					engine?.set_layer(Number(e.currentTarget.value));
					engine?.render();
					refresh();
				}}
			>
				{#each layers.layers as l, i}
					<option value={i}>{i}: {l.name}</option>
				{/each}
			</select>
		</label>
	</header>

	<div class="body">
		<nav class="tools" aria-label={t.tools}>
			{#each tools as [id, key] (id)}
				<button
					class={['icon-btn', { active: status.tool === id }]}
					title={t[key]}
					aria-label={t[key]}
					aria-pressed={status.tool === id}
					onclick={() => tool(id)}
				>
					<ToolIcon name={id} />
				</button>
			{/each}
			<label class="chk"
				><input
					type="checkbox"
					checked={filled}
					onchange={(e) => {
						filled = e.currentTarget.checked;
						engine?.set_filled(filled);
					}}
				/>
				{t.filled}</label
			>
			<label class="chk"
				>{t.trackWidth}
				<input
					type="number"
					min="1"
					max="100"
					value="4"
					onchange={(e) => engine?.set_track_width(Number(e.currentTarget.value))}
				/>
			</label>
		</nav>

		<CanvasHost bind:engine onStatus={refresh} />

		<aside class="libs">
			<h3>{t.libraries}</h3>
			<div class="tree">
				{#each libs as lib}
					<details open={lib.stem === 'stdlib'}>
						<summary>{lib.title}</summary>
						{#each lib.categories as cat}
							<details>
								<summary>{cat.name}</summary>
								{#each cat.macros as [key, name]}
									<button class="mac" onclick={() => pickMacro(lib.stem, key)}
										>{key} {name}</button
									>
								{/each}
							</details>
						{/each}
					</details>
				{/each}
			</div>
		</aside>
	</div>

	<footer class="status">
		<span>{status.title || t.statusReady}</span>
		<span>{t.layer} {status.layer}</span>
		<span>{status.x}, {status.y} LU</span>
		{#if status.pcb}
			<span>{status.xmm.toFixed(2)} × {status.ymm.toFixed(2)} mm</span>
		{/if}
		<span>{Math.round(status.zoom * 100)}%</span>
		<span>{status.n} obj</span>
		{#if status.selected}<span>sel {status.selected}</span>{/if}
		<span>snap {status.snap} / grid {status.grid}</span>
	</footer>
</div>

{#if menu}
	<button class="scrim" onclick={() => (menu = null)} aria-label="close menu"></button>
{/if}

{#if showLayers}
	<div class="modal" role="dialog">
		<div class="card">
			<h2>{t.layers}</h2>
			<table>
				<thead>
					<tr>
						<th>#</th>
						<th>{t.layer}</th>
						<th>{t.showScreen}</th>
						<th>{t.showPrint}</th>
					</tr>
				</thead>
				<tbody>
					{#each layers.layers as l, i}
						<tr>
							<td
								><input
									type="color"
									value={'#' + l.color.map((c) => c.toString(16).padStart(2, '0')).join('')}
									oninput={(e) => {
										const hex = e.currentTarget.value;
										const r = parseInt(hex.slice(1, 3), 16);
										const g = parseInt(hex.slice(3, 5), 16);
										const b = parseInt(hex.slice(5, 7), 16);
										engine?.set_layer_color(i, r, g, b);
										engine?.render();
									}}
								/></td
							>
							<td
								><input
									value={l.name}
									onchange={(e) => engine?.set_layer_name(i, e.currentTarget.value)}
								/></td
							>
							<td
								><input
									type="checkbox"
									checked={l.show}
									onchange={(e) => {
										engine?.set_layer_show(i, e.currentTarget.checked);
										engine?.render();
										refresh();
									}}
								/></td
							>
							<td
								><input
									type="checkbox"
									checked={l.print}
									onchange={(e) => engine?.set_layer_print(i, e.currentTarget.checked)}
								/></td
							>
						</tr>
					{/each}
				</tbody>
			</table>
			<button onclick={() => (showLayers = false)}>{t.close}</button>
		</div>
	</div>
{/if}

{#if showAbout}
	<div class="modal">
		<div class="card about">
			<img src="/favicon.svg" width="56" height="56" alt="" />
			<h2>{t.about}</h2>
			<p>{t.aboutBody}</p>
			<button onclick={() => (showAbout = false)}>{t.close}</button>
		</div>
	</div>
{/if}

{#if error}
	<div class="modal">
		<div class="card">
			<p>{error}</p>
			<button onclick={() => (error = '')}>{t.close}</button>
		</div>
	</div>
{/if}

<style>
	.shell {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	.menubar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		background: var(--bg-menu);
		border-bottom: 1px solid var(--border);
		z-index: 3;
	}
	.brand-mark {
		width: 22px;
		height: 22px;
		flex-shrink: 0;
		margin-right: 2px;
	}
	.brand {
		margin-right: 12px;
		letter-spacing: 0.04em;
		color: var(--accent);
	}
	.menu {
		position: relative;
	}
	.menu-btn {
		border: none;
		background: transparent;
		padding: 6px 10px;
	}
	.dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		min-width: 220px;
		background: var(--bg-menu);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		box-shadow: var(--shadow);
		padding: 6px;
		display: flex;
		flex-direction: column;
		z-index: 5;
	}
	.dropdown button {
		text-align: left;
		border: none;
		background: transparent;
		padding: 6px 8px;
		border-radius: 4px;
	}
	.dropdown button:hover {
		background: var(--bg-panel);
	}
	.grow {
		flex: 1;
	}
	.body {
		flex: 1;
		display: flex;
		min-height: 0;
	}
	.tools {
		width: 56px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 6px;
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
		overflow: auto;
	}
	.chk {
		font-size: 10px;
		color: var(--fg-muted);
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 8px;
	}
	.chk input[type='number'] {
		width: 100%;
	}
	.libs {
		width: 260px;
		border-left: 1px solid var(--border);
		background: var(--bg-panel);
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
	.libs h3 {
		margin: 0;
		padding: 10px 12px;
		font-size: 13px;
		border-bottom: 1px solid var(--border);
	}
	.tree {
		overflow: auto;
		padding: 8px;
		font-size: 12px;
	}
	.mac {
		display: block;
		width: 100%;
		text-align: left;
		border: none;
		background: transparent;
		padding: 3px 6px;
		border-radius: 4px;
	}
	.mac:hover {
		background: var(--bg-menu);
	}
	.status {
		display: flex;
		gap: 16px;
		padding: 4px 12px;
		font-size: 12px;
		font-family: var(--mono);
		background: var(--bg-menu);
		border-top: 1px solid var(--border);
		color: var(--fg-muted);
	}
	.scrim {
		position: fixed;
		inset: 0;
		background: transparent;
		border: none;
		z-index: 2;
	}
	.modal {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.35);
		display: grid;
		place-items: center;
		z-index: 10;
	}
	.card {
		background: var(--bg-menu);
		padding: 20px;
		border-radius: 12px;
		max-width: 720px;
		width: min(720px, 92vw);
		max-height: 80vh;
		overflow: auto;
		box-shadow: var(--shadow);
	}
	.about {
		text-align: center;
	}
	.about img {
		display: block;
		margin: 0 auto 8px;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		margin: 12px 0;
		font-size: 13px;
	}
	td,
	th {
		padding: 4px;
		text-align: left;
	}
</style>
