import { writable, get } from "svelte/store";

import { fi } from "./translations/fi";
import { en } from "./translations/en";
import { userPrefs, updateUserPrefs } from "$lib/prefsStore";

export type Language = 'en' | 'fi';
export type Translation = Record<string, string | string[] | Array<Record<string, string>>>;

const translations: Record<Language, Translation> = {
  'en': en,
  'fi': fi,
};

const isValidLanguage = (lang: string): lang is Language => ['en', 'fi'].includes(lang);

const getInitialLanguage = async () => {
  const saved = get(userPrefs).mainPrefs.lang;
  return saved && isValidLanguage(saved) ? saved : "en";
};

const createLangStore = async () => {
  const { subscribe, set, update } = writable<Language>(await getInitialLanguage());
  return { subscribe, set: (lang: Language) => { updateUserPrefs("mainPrefs", "lang", lang); set(lang); }, update };
};

export const lang = await createLangStore();

export const t = { subscribe: (run: (value: Translation) => void) => lang.subscribe((lang) => run(translations[lang])) };