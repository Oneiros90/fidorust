<script lang="ts">
	let {
		label,
		shortcut,
		disabled = false,
		onclick
	}: {
		label: string;
		shortcut?: string;
		disabled?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<button type="button" {disabled} {onclick}>
	{label}{#if shortcut}<span class="acc">{shortcut}</span>{/if}
</button>

<style>
	button {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
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
	}
</style>
