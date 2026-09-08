export const LOCAL_LIBRARY_KEY = 'fidorust.localLibrary';

export function loadLocalLibrary(): string {
	try {
		return localStorage.getItem(LOCAL_LIBRARY_KEY) ?? '';
	} catch {
		return '';
	}
}

export function saveLocalLibrary(fcl: string) {
	try {
		if (!fcl) localStorage.removeItem(LOCAL_LIBRARY_KEY);
		else localStorage.setItem(LOCAL_LIBRARY_KEY, fcl);
	} catch {
		/* quota / private mode */
	}
}
