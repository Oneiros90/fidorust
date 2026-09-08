<script lang="ts">
	let {
		labelElement = $bindable(),
		hex,
		label,
		name = undefined,
		dir
	}: {
		labelElement: HTMLLabelElement | undefined;
		hex: string | null;
		label: string;
		name?: string;
		dir: 'ltr' | 'rtl';
	} = $props();

	function preventDefault(e: MouseEvent) {
		e.preventDefault();
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
<label bind:this={labelElement} onclick={preventDefault} onmousedown={preventDefault} {dir}>
	<div class="container">
		<input
			type="color"
			{name}
			value={hex ?? '#000000'}
			onclick={preventDefault}
			onmousedown={preventDefault}
			aria-haspopup="dialog"
			aria-label={label}
		/>
		<div class="alpha"></div>
		<div class="color" style:background={hex}></div>
	</div>
	<span class="sr-only">{label}</span>
</label>

<style>
	label {
		display: inline-flex;
		align-items: center;
		margin: 0;
		height: var(--input-size, 22px);
		cursor: pointer;
		user-select: none;
		border-radius: 6px;
	}
	.container {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		width: var(--input-size, 22px);
		height: var(--input-size, 22px);
		flex-shrink: 0;
		overflow: hidden;
	}
	input {
		position: absolute;
		inset: 0;
		margin: 0;
		padding: 0;
		border: none;
		width: 100%;
		height: 100%;
		opacity: 0;
		appearance: none;
	}
	.alpha,
	.color {
		position: absolute;
		inset: 0;
		border-radius: 6px;
		user-select: none;
		pointer-events: none;
	}
	.alpha {
		background: var(
			--alpha-grid-bg,
			linear-gradient(
					45deg,
					color-mix(in srgb, var(--fg) 14%, transparent) 25%,
					transparent 25%,
					transparent 75%,
					color-mix(in srgb, var(--fg) 14%, transparent) 75%
				)
				0 0 / 8px 8px,
			linear-gradient(
					45deg,
					color-mix(in srgb, var(--fg) 14%, transparent) 25%,
					transparent 25%,
					transparent 75%,
					color-mix(in srgb, var(--fg) 14%, transparent) 75%
				)
				4px 4px / 8px 8px,
			var(--bg-menu)
		);
	}
	.color {
		border: 1px solid var(--border);
	}
	input:focus-visible ~ .color {
		outline: 2px solid var(--focus-color, var(--accent));
		outline-offset: 1px;
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
