<script lang="ts">
	import { untrack } from 'svelte';
	import type { ProjectSettingsValues } from '../app/appSession.svelte';
	import type { Dict } from '../i18n';
	import { formatLuAsMm } from '../lib/constants';
	import { clampInt } from '../lib/num';
	import Modal from './Modal.svelte';

	type SectionId = 'grid' | 'snap' | 'defaults';

	let {
		t,
		values,
		onApply,
		onCancel
	}: {
		t: Dict;
		values: ProjectSettingsValues;
		onApply: (values: ProjectSettingsValues) => void;
		onCancel: () => void;
	} = $props();

	let section = $state<SectionId>('grid');
	let edit = $state(
		untrack(() => ({
			...values,
			strokeLu: values.strokeHundredths / 100
		}))
	);

	const sections = $derived([
		{ id: 'grid' as const, label: t.sectionGrid },
		{ id: 'snap' as const, label: t.sectionSnap },
		{ id: 'defaults' as const, label: t.sectionDefaults }
	]);

	function apply() {
		onApply({
			gridX: clampInt(edit.gridX, 1, 40),
			gridY: clampInt(edit.gridY, 1, 40),
			snapX: clampInt(edit.snapX, 1, 20),
			snapY: clampInt(edit.snapY, 1, 20),
			showGrid: edit.showGrid,
			snapEnable: edit.snapEnable,
			hideComponentOrigin: edit.hideComponentOrigin,
			strokeHundredths: clampInt(edit.strokeLu * 100, 1, 2000),
			defaultFilled: edit.defaultFilled
		});
	}
</script>

<Modal
	title={t.projectSettings}
	titleId="project-settings-title"
	maxWidth="720px"
	maxHeight="520px"
	overflow="hidden"
	onClose={onCancel}
	onSubmit={apply}
>
	<div class="settings">
		<nav class="toc" aria-label={t.projectSettings}>
			{#each sections as s (s.id)}
				<button
					type="button"
					class={['toc-item', { active: section === s.id }]}
					aria-current={section === s.id ? 'true' : undefined}
					onclick={() => (section = s.id)}
				>
					{s.label}
				</button>
			{/each}
		</nav>
		<div class="pane">
			{#if section === 'grid'}
				<h3>{t.sectionGrid}</h3>
				<label>
					{t.gridX}
					<span class="value">
						<input type="number" min="1" max="40" step="1" bind:value={edit.gridX} />
						<span class="mm">{formatLuAsMm(edit.gridX)}</span>
					</span>
				</label>
				<label>
					{t.gridY}
					<span class="value">
						<input type="number" min="1" max="40" step="1" bind:value={edit.gridY} />
						<span class="mm">{formatLuAsMm(edit.gridY)}</span>
					</span>
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={edit.showGrid} />
					{t.showGrid}
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={edit.hideComponentOrigin} />
					{t.hideComponentOrigin}
				</label>
			{:else if section === 'snap'}
				<h3>{t.sectionSnap}</h3>
				<label>
					{t.snapX}
					<span class="value">
						<input type="number" min="1" max="20" step="1" bind:value={edit.snapX} />
						<span class="mm">{formatLuAsMm(edit.snapX)}</span>
					</span>
				</label>
				<label>
					{t.snapY}
					<span class="value">
						<input type="number" min="1" max="20" step="1" bind:value={edit.snapY} />
						<span class="mm">{formatLuAsMm(edit.snapY)}</span>
					</span>
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={edit.snapEnable} />
					{t.enableSnap}
				</label>
			{:else}
				<h3>{t.sectionDefaults}</h3>
				<label>
					{t.defaultLineWidth}
					<input type="number" min="0.01" max="20" step="0.05" bind:value={edit.strokeLu} />
				</label>
				<label class="chk">
					<input type="checkbox" bind:checked={edit.defaultFilled} />
					{t.defaultFill}
				</label>
			{/if}
		</div>
	</div>
	<div class="dialog-actions">
		<button type="button" class="primary" onclick={apply}>{t.ok}</button>
		<button type="button" onclick={onCancel}>{t.cancel}</button>
	</div>
</Modal>

<style>
	.settings {
		display: grid;
		grid-template-columns: 168px 1fr;
		min-height: 280px;
		max-height: 360px;
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
		margin-bottom: 16px;
	}
	.toc {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 6px;
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
	}
	.toc-item {
		text-align: left;
		padding: 7px 10px;
		border: 0;
		border-radius: 4px;
		background: transparent;
		color: var(--fg);
		font: inherit;
		font-size: 13px;
		cursor: pointer;
	}
	.toc-item:hover {
		background: color-mix(in srgb, var(--fg) 8%, transparent);
	}
	.toc-item.active {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
		font-weight: 600;
	}
	.pane {
		padding: 16px 20px;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.pane h3 {
		margin: 0 0 4px;
		font-size: 14px;
		font-weight: 600;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 13px;
		max-width: 280px;
	}
	.chk {
		flex-direction: row;
		align-items: center;
		gap: 8px;
		max-width: none;
	}
	.value {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.value input[type='number'] {
		width: 88px;
		flex: 0 0 auto;
	}
	input[type='number'] {
		width: 100%;
	}
	.mm {
		font-family: var(--mono);
		font-size: 12px;
		color: var(--fg-muted);
		cursor: default;
		pointer-events: none;
		user-select: none;
	}
</style>
