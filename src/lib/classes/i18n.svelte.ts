import type { Language, Translation } from "$lib/i18n/i18n.svelte";

export class I18n {
  #translations: Record<Language, Translation>;
  #lang = $state<Language>("en");
  onLangChange?: (lang: Language) => void;

  constructor(translations: Record<Language, Translation>) {
    this.#translations = translations;
  }

  get lang() {
    return this.#lang;
  };

  set lang(value: Language) {
    if (value === this.#lang) return;
    this.#lang = value;
    this.onLangChange?.(value);
  };

  get t() {
    return this.#translations[this.#lang];
  };

  applyLang = (value: Language) => {
    this.#lang = value;
  }
}