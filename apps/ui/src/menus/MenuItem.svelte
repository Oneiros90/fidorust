<script lang="ts">
	let {
		label,
		shortcut,
		disabled = false,
		checkable = false,
		active = false,
		onclick
	}: {
		label: string;
		shortcut?: string;
		disabled?: boolean;
		/** Reserved checkmark column (language, theme, …). */
		checkable?: boolean;
		active?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<button type="button" {disabled} {onclick} aria-checked={checkable ? active : undefined}>
	{#if checkable}
		<span class="mark" aria-hidden="true">{active ? '✓' : ''}</span>
	{/if}
	<span class="lbl">{label}</span>
	{#if shortcut}<span class="acc">{shortcut}</span>{/if}
</button>

<style>
	button {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		text-align: left;
		border: none;
		background: transparent;
		padding: 6px 8px;
		border-radius: 4px;
		font: inherit;
		font-size: 13px;
		color: inherit;
	}
	.mark {
		width: 14px;
		flex-shrink: 0;
		color: var(--accent);
		font-size: 12px;
		line-height: 1;
	}
	.lbl {
		flex: 1;
		min-width: 0;
	}
	button:hover:not(:disabled) {
		background: var(--bg-panel);
	}
	button:disabled {
		opacity: 0.45;
	}
	.acc {
		color: var(--fg-muted);
		font-size: 11px;
		font-family: var(--mono);
		margin-left: auto;
	}
</style>
