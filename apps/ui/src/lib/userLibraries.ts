export const USER_LIBRARIES_KEY = 'fidorust.userLibraries';
export const LOCAL_LIBRARY_KEY = 'fidorust.localLibrary';

export type UserLibraryRecord = {
	stem: string;
	title: string;
	fcl: string;
	aliases?: string[];
};

function readRaw(key: string): string | null {
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

function writeRaw(key: string, value: string | null) {
	try {
		if (value === null) localStorage.removeItem(key);
		else localStorage.setItem(key, value);
	} catch {
		/* quota / private mode */
	}
}

export function loadUserLibraries(): UserLibraryRecord[] {
	const raw = readRaw(USER_LIBRARIES_KEY);
	if (raw) {
		try {
			const parsed = JSON.parse(raw) as unknown;
			if (Array.isArray(parsed)) {
				return parsed.filter(
					(x): x is UserLibraryRecord =>
						!!x &&
						typeof x === 'object' &&
						typeof (x as UserLibraryRecord).stem === 'string' &&
						typeof (x as UserLibraryRecord).fcl === 'string'
				);
			}
		} catch {
			/* ignore */
		}
	}
	const old = readRaw(LOCAL_LIBRARY_KEY);
	if (old) {
		writeRaw(LOCAL_LIBRARY_KEY, null);
		const migrated: UserLibraryRecord[] = [{ stem: 'local', title: 'Local library', fcl: old }];
		saveUserLibraries(migrated);
		return migrated;
	}
	return [];
}

export function saveUserLibraries(libs: UserLibraryRecord[]) {
	writeRaw(USER_LIBRARIES_KEY, JSON.stringify(libs));
}

export function persistUserLibrariesBlob(json: string) {
	writeRaw(USER_LIBRARIES_KEY, json.trim() ? json : '[]');
}
