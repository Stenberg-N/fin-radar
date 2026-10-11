import { get } from "svelte/store";

import type { Language, Translation } from "$lib/i18n/i18n.svelte";
import { userPrefs, updateUserPrefs } from "$lib/utils/prefsStore";

const isValidLanguage = (lang: string): lang is Language => ['en', 'fi'].includes(lang);

const getInitialLanguage = () => {
  const saved = get(userPrefs).mainPrefs.lang;
  return saved && isValidLanguage(saved) ? saved : "en";
};

export class I18n {
  #translations: Record<Language, Translation>;
  #lang = $state<Language>(getInitialLanguage());

  constructor(translations: Record<Language, Translation>) {
    this.#translations = translations;
  }

  get lang() {
    return this.#lang;
  };

  set lang(value: Language) {
    this.#lang = value;
    updateUserPrefs("mainPrefs", "lang", value);
  };

  get t() {
    return this.#translations[this.#lang];
  };
}