import { RIGHT_PANEL_COLLAPSE_QUERY } from '../lib/constants';

export class UiState {
	menu = $state<string | null>(null);
	ctxMenu = $state<
		| { kind: 'edit'; x: number; y: number }
		| { kind: 'layer'; x: number; y: number; index: number }
		| null
	>(null);
	rightTab = $state<'layers' | 'library'>('layers');
	rightCollapsed = $state(
		typeof matchMedia === 'function' && matchMedia(RIGHT_PANEL_COLLAPSE_QUERY).matches
	);
	editingLayerName = $state<number | null>(null);

	toggleMenu = (id: string) => {
		this.menu = this.menu === id ? null : id;
	};

	closeMenu = () => {
		this.menu = null;
	};
}
