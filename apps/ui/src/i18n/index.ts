import { en } from './en';
import { it } from './it';

export { en } from './en';
export type { Dict } from './en';
export { it } from './it';

export type Locale = 'it' | 'en';

export function dict(locale: Locale): typeof en {
  return locale === 'en' ? en : it;
}
