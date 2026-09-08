import { RIGHT_PANEL_COLLAPSE_QUERY } from '../lib/constants';

export class UiState {
	menu = $state<string | null>(null);
	ctxMenu = $state<
		| { kind: 'edit'; x: number; y: number }
		| { kind: 'layer'; x: number; y: number; index: number }
		| { kind: 'libraryItem'; x: number; y: number; stem: string; key: string }
		| null
	>(null);
	rightTab = $state<'layers' | 'library'>('layers');
	rightCollapsed = $state(
		typeof matchMedia === 'function' && matchMedia(RIGHT_PANEL_COLLAPSE_QUERY).matches
	);
	editingLayerName = $state<number | null>(null);
	editingLibraryField = $state<{ stem: string; key: string; field: 'name' | 'description' } | null>(
		null
	);
	expandedUserLibs = $state<Record<string, boolean>>({ project: true, local: false });
	libraryFocus = $state<{ stem: string; key: string } | null>(null);

	toggleMenu = (id: string) => {
		this.menu = this.menu === id ? null : id;
	};

	closeMenu = () => {
		this.menu = null;
	};
}
