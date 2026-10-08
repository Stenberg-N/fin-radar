import { get } from "svelte/store";

import { fi } from "./translations/fi";
import { en } from "./translations/en";
import { userPrefs, updateUserPrefs } from "$lib/prefsStore";

export type Language = 'en' | 'fi';
export type Translation = Record<string, string | string[] | Array<Record<string, string>> | Record<string, string>>;

const translations: Record<Language, Translation> = {
  'en': en,
  'fi': fi,
};

const isValidLanguage = (lang: string): lang is Language => ['en', 'fi'].includes(lang);

const getInitialLanguage = () => {
  const saved = get(userPrefs).mainPrefs.lang;
  return saved && isValidLanguage(saved) ? saved : "en";
};

class I18n {
  #lang = $state<Language>(getInitialLanguage());

  get lang() {
    return this.#lang;
  };

  set lang(value: Language) {
    this.#lang = value;
    updateUserPrefs("mainPrefs", "lang", value);
  };

  get t() {
    return translations[this.#lang];
  };
}

export const i18n = new I18n();