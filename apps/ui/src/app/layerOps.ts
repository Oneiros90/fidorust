import type { Layer } from './engineTypes';

export function otherLayers(layers: Layer[], index: number) {
	return layers.map((l, i) => ({ name: l.name, i })).filter((x) => x.i !== index);
}
