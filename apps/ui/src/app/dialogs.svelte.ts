import type { PropFormField } from '../lib/propForm';
import type { ExportFormat } from '../lib/exportOptions';

export type DialogState =
	| { kind: 'deleteLayer'; index: number }
	| { kind: 'deleteComponent'; stem: string; key: string }
	| { kind: 'about' }
	| { kind: 'technologies' }
	| { kind: 'projectSettings' }
	| { kind: 'properties'; fields: PropFormField[] }
	| { kind: 'error'; message: string }
	| { kind: 'discard' }
	| { kind: 'shareLink'; url: string }
	| { kind: 'shareFcd'; text: string }
	| { kind: 'export'; format: ExportFormat };

/**
 * Single dialog slot. Kind `deleteLayer` is listed first: Escape used to clear
 * `pendingDeleteLayer` before any other flag in `onKey`.
 */
export class Dialogs {
	dialog = $state<DialogState | null>(null);
	isOpen = $derived(this.dialog !== null);

	open(d: DialogState) {
		this.dialog = d;
	}

	close() {
		this.dialog = null;
	}
}
