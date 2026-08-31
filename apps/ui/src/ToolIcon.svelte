<script lang="ts">
	let { name }: { name: string } = $props();
</script>

<svg
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.75"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#if name === 'select'}
		<path
			fill="currentColor"
			stroke="none"
			d="M6 3.8v16.2l4.4-3.8 3.2 6.1 2.7-1.3-3.3-6.2H19z"
		/>
	{:else if name === 'line'}
		<line x1="5" y1="18" x2="19" y2="6" />
		<circle cx="5" cy="18" r="1.55" fill="currentColor" stroke="none" />
		<circle cx="19" cy="6" r="1.55" fill="currentColor" stroke="none" />
	{:else if name === 'rect'}
		<rect x="5" y="6" width="14" height="12" rx="1.2" />
	{:else if name === 'ellipse'}
		<ellipse cx="12" cy="12" rx="8" ry="5.6" />
	{:else if name === 'poly'}
		<polyline points="4.5,17 9,6.5 14.2,13.8 19.5,7" />
	{:else if name === 'bezier'}
		<path d="M4.5 17c3.6 0 3.6-10.5 7.5-10.5S16 17 19.5 7" />
	{:else if name === 'text'}
		<path d="M7.4 18 12 6.4 16.6 18" />
		<path d="M9.1 13.4h5.8" />
	{:else if name === 'connection'}
		<line x1="4" y1="12" x2="20" y2="12" />
		<circle cx="12" cy="12" r="2.2" fill="currentColor" stroke="none" />
	{:else if name === 'pcb-track'}
		<path d="M5 17.5V12h7V6.5h7" stroke-width="2.35" />
	{:else if name === 'pcb-pad'}
		<circle cx="12" cy="12" r="7" />
		<circle cx="12" cy="12" r="2.7" />
	{:else if name === 'macro'}
		<rect x="7.4" y="5.4" width="9.2" height="13.2" rx="1" />
		<path d="M4.2 9h3.2M4.2 12h3.2M4.2 15h3.2M16.6 9h3.2M16.6 12h3.2M16.6 15h3.2" />
	{:else if name === 'pan'}
		<polyline points="9 7.2 12 4 15 7.2" />
		<polyline points="9 16.8 12 20 15 16.8" />
		<polyline points="7.2 9 4 12 7.2 15" />
		<polyline points="16.8 9 20 12 16.8 15" />
		<line x1="12" y1="4" x2="12" y2="20" />
		<line x1="4" y1="12" x2="20" y2="12" />
	{:else if name === 'zoom'}
		<circle cx="10.5" cy="10.5" r="5.6" />
		<line x1="14.6" y1="14.6" x2="19.4" y2="19.4" />
	{:else}
		<rect x="6" y="6" width="12" height="12" rx="1.2" />
	{/if}
</svg>

<style>
	svg {
		width: 20px;
		height: 20px;
		display: block;
	}
</style>
