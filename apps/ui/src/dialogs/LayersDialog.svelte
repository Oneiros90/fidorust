<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import DialogHeader from './DialogHeader.svelte';
	import Modal from './Modal.svelte';

	const app = getAppSession();
</script>

<Modal labelledBy="layers-title">
	<DialogHeader
		title={app.t.layers}
		titleId="layers-title"
		closeLabel={app.t.close}
		onClose={() => (app.showLayers = false)}
	/>
	<table>
		<thead>
			<tr>
				<th>#</th>
				<th>{app.t.layer}</th>
				<th>{app.t.showScreen}</th>
				<th>{app.t.showPrint}</th>
			</tr>
		</thead>
		<tbody>
			{#each app.layers.layers as l, i (i)}
				<tr>
					<td
						><input
							type="color"
							value={'#' + l.color.map((c) => c.toString(16).padStart(2, '0')).join('')}
							oninput={(e) => {
								const hex = e.currentTarget.value;
								app.setLayerColor(
									i,
									parseInt(hex.slice(1, 3), 16),
									parseInt(hex.slice(3, 5), 16),
									parseInt(hex.slice(5, 7), 16)
								);
							}}
						/></td
					>
					<td
						><input
							value={l.name}
							onchange={(e) => app.setLayerName(i, e.currentTarget.value)}
						/></td
					>
					<td
						><input
							type="checkbox"
							checked={l.show}
							onchange={(e) => app.setLayerShow(i, e.currentTarget.checked)}
						/></td
					>
					<td
						><input
							type="checkbox"
							checked={l.print}
							onchange={(e) => app.setLayerPrint(i, e.currentTarget.checked)}
						/></td
					>
				</tr>
			{/each}
		</tbody>
	</table>
</Modal>

<style>
	table {
		width: 100%;
		border-collapse: collapse;
		margin: 12px 0 0;
		font-size: 13px;
	}
	td,
	th {
		padding: 4px;
		text-align: left;
	}
</style>
