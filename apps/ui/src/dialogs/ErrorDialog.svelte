<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal>
	<p>{app.error}</p>
	<button type="button" onclick={() => (app.error = '')}>{app.t.close}</button>
</Modal>
