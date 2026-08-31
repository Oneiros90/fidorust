<script lang="ts">
	import { getAppSession } from '../app/appContext';

	const app = getAppSession();
</script>

<label>
	{app.t.layer}
	<select
		value={app.status.layer}
		onchange={(e) => app.setLayer(Number(e.currentTarget.value))}
	>
		{#each app.layers.layers as l, i (i)}
			<option value={i}>{i}: {l.name}</option>
		{/each}
	</select>
</label>
