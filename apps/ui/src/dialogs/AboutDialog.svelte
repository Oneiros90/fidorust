<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { appVersion, displayVersion } from '../lib/updates';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal>
	<div class="about">
		<img src={app.assetUrl('favicon.svg')} width="56" height="56" alt="" />
		<h2>FidoRust {displayVersion(appVersion)}</h2>
		{@html app.t.aboutContent}
		<button type="button" onclick={() => (app.showAbout = false)}>{app.t.close}</button>
	</div>
</Modal>

<style>
	.about {
		text-align: center;
	}

	.about :global(p) {
		margin: 0 0 0.75rem;
		line-height: 1.45;
	}

	.about :global(a) {
		color: var(--accent, #2563eb);
	}

	img {
		display: block;
		margin: 0 auto 8px;
	}
</style>
