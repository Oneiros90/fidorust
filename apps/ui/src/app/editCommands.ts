import type { AppSession } from './appSession.svelte';

export function doDelete(s: AppSession) {
	s.engine?.mutate((app) => {
		app.key('Delete', false);
	});
}

export function doUndo(s: AppSession) {
	s.engine?.mutate((app) => {
		app.undo();
	});
}

export function doRedo(s: AppSession) {
	s.engine?.mutate((app) => {
		app.redo();
	});
}

export function doRotate(s: AppSession) {
	s.engine?.mutate((app) => {
		app.rotate();
	});
}

export function doMirror(s: AppSession) {
	s.engine?.mutate((app) => {
		app.mirror();
	});
}

export function doSplit(s: AppSession) {
	s.engine?.mutate((app) => {
		app.split_selected_macros();
	});
}

export function doSelectAll(s: AppSession) {
	s.engine?.mutate((app) => {
		app.key('a', true);
	});
}

export function doInvert(s: AppSession) {
	s.engine?.mutate((app) => {
		app.invert_selection();
	});
}

export function fit(s: AppSession) {
	s.engine?.mutate((app) => {
		app.fit();
	});
}
