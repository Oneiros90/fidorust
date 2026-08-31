import { createContext } from 'svelte';
import type { AppSession } from './appSession.svelte';

export const [getAppSession, setAppSession] = createContext<AppSession>();
