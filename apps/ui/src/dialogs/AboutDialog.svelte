<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal>
	<div class="about">
		<img src={app.assetUrl('favicon.svg')} width="56" height="56" alt="" />
		<h2>{app.t.about}</h2>
		<p>{app.t.aboutBody}</p>
		<button type="button" onclick={() => (app.showAbout = false)}>{app.t.close}</button>
	</div>
</Modal>

<style>
	.about {
		text-align: center;
	}
	img {
		display: block;
		margin: 0 auto 8px;
	}
</style>
