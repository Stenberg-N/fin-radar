import { I18n } from '../classes/i18n.svelte'
import { fi } from "./translations/fi";
import { en } from "./translations/en";

export type Language = 'en' | 'fi';
export type Translation = Record<string, string | string[] | Array<Record<string, string>> | Record<string, string>>;

const translations: Record<Language, Translation> = {
  'en': en,
  'fi': fi,
};

export const i18n = new I18n(translations);