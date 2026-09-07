<script lang="ts">
	let {
		z,
		label,
		onclick
	}: {
		z: string | number;
		label: string;
		onclick: () => void;
	} = $props();
</script>

<button type="button" class="scrim" style:z-index={z} aria-label={label} {onclick}></button>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: transparent;
		border: none;
	}
</style>
