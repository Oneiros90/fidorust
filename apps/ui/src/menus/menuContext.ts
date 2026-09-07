import { createContext } from 'svelte';

export const [getCloseMenu, setCloseMenu] = createContext<() => void>();
