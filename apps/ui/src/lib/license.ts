const STORAGE_KEY = 'fidorust.license';

export const PRO_PURCHASE_URL = 'https://github.com/Oneiros90/fidorust';

export function readLicense(): string | null {
	try {
		const v = localStorage.getItem(STORAGE_KEY);
		return v && v.length > 0 ? v : null;
	} catch {
		return null;
	}
}

export function isLicensed(): boolean {
	return readLicense() !== null;
}

export function activateLicense(key: string): boolean {
	const trimmed = key.trim();
	if (!trimmed) return false;
	localStorage.setItem(STORAGE_KEY, `key:${trimmed}`);
	return true;
}

export function deactivateLicense(): void {
	localStorage.removeItem(STORAGE_KEY);
}
