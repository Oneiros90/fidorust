<script lang="ts">
	let {
		title,
		titleId,
		onClose,
		closeLabel
	}: {
		title: string;
		titleId: string;
		onClose: () => void;
		closeLabel: string;
	} = $props();
</script>

<div class="card-head">
	<h2 id={titleId}>{title}</h2>
	<button type="button" class="card-close" onclick={onClose} aria-label={closeLabel}>
		<span aria-hidden="true">×</span>
	</button>
</div>

<style>
	.card-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	h2 {
		margin: 0;
	}
	.card-close {
		width: 28px;
		height: 28px;
		padding: 0;
		display: grid;
		place-items: center;
		font-size: 18px;
		line-height: 1;
		flex-shrink: 0;
	}
</style>
