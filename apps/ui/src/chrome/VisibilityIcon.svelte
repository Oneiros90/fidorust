<script lang="ts">
	let { show }: { show: boolean } = $props();
</script>

{#if show}
	<svg viewBox="0 0 24 24" aria-hidden="true">
		<path
			d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7z"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
		/>
		<circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2" />
	</svg>
{:else}
	<svg viewBox="0 0 24 24" aria-hidden="true">
		<path
			d="M3 3l18 18M10.6 10.6A3 3 0 0 0 12 15a3 3 0 0 0 2.4-1.2M9.9 5.1A11 11 0 0 1 12 5c6 0 10 7 10 7a18 18 0 0 1-3.2 3.8M6.1 6.1C3.7 8 2 12 2 12s4 7 10 7a10 10 0 0 0 4-.8"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
		/>
	</svg>
{/if}

<style>
	svg {
		width: 14px;
		height: 14px;
		display: block;
	}
</style>
