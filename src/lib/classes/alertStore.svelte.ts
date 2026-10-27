import { SvelteMap } from "svelte/reactivity";

interface AlertConfig {
  /**
   * Takes any string. Translations are first checked if the string matches any of its keys. If not, the string will act as the message as is.
   * 
   * You can also provide the message from the translations store itself by using `i18n.t`,
   * but keep in mind that the alert message will not be reactive at that point, i.e. get updated if the localization is changed while the alert is still alive.
   */
  message: string,
  /**
   * Determines if the alert should remain alive until closed explicitly or die after 2.5 seconds.
   */
  isTimer: boolean,
  /**
   * Should the buttons for confirming and canceling should be visible.
   */
  buttons: boolean,
  /**
   * Attaches a callback to the confirm button.
   */
  onConfirm?: () => void;
  /**
   * Attaches a callback to the cancel button.
   */
  onCancel?: () => void;
  /**
   * Should only the confirm button be visible.
   */
  onlyConfirmButton?: boolean;
  /**
   * Pass down a translation key to replace the text on the confirm button.
   */
  confirmButtonI18nKey?: string;
  /**
   * Pass down a translation key to replace the text on the cancel button.
   */
  cancelButtonI18nKey?: string;
  /**
   * When given as an array, places each item as their own span element, each on their own row.
   */
  additionalText?: string | string[];
  /**
   * If `true`, creates a br element and a span element that is a border, creating a divider between the message and additionalText. Defaults to `false`.
   */
  addDivider?: boolean;
  /**
   * Should the confirm button be white.
   */
  isConfirmButtonWhite?: boolean;
  /**
   * An object used for passing data to an ItemSelect component for the alert.
   * @field selectorData is used for the select options.
   * @field selectorOverrideData is used to override the selectorData items. It is visual-only.
   */
  selector?: {
    selectorData: readonly string[];
    selectorOverrideData?: string[];
  };
}

export class Alert {
  #timeout?: ReturnType<typeof setTimeout>;
  #raf: number | null = null;
  #DURATION: number = 2500;
  #startedAt: number = 0;
  #onExpire: () => void;

  readonly id: number;
  readonly config: Required<Omit<AlertConfig, "additionalText" | "selector">> & Pick<AlertConfig, "additionalText" | "selector">;
  #duration: number = this.#DURATION;
  #progress = $state(0);

  constructor(id: number, config: AlertConfig, onExpire: () => void) {
    this.id = id;
    this.config = {
      ...config,
      onConfirm: config?.onConfirm ?? (() => {}),
      onCancel: config?.onCancel ?? (() => {}),
      onlyConfirmButton: config?.onlyConfirmButton ?? false,
      confirmButtonI18nKey: config?.confirmButtonI18nKey ?? "confirm.button",
      cancelButtonI18nKey: config?.cancelButtonI18nKey ?? "cancel.button",
      additionalText: config?.additionalText,
      addDivider: config?.addDivider ?? false,
      isConfirmButtonWhite: config?.isConfirmButtonWhite ?? false,
      selector: config?.selector,
    };
    this.#onExpire = onExpire;
    if (this.config.isTimer) this.resume();
  }

  get duration() { return this.#duration; };
  get progress() { return this.#progress; };

  #tick = () => {
    this.#progress = Math.max(0, ((this.#duration - (performance.now() - this.#startedAt)) / this.#DURATION) * 100);
    this.#raf = this.#progress > 0 !== null ? requestAnimationFrame(this.#tick) : null;
  };

  resume = () => {
    if (!this.config.isTimer) return;

    this.#startedAt = performance.now();
    this.#timeout = setTimeout(() => this.#onExpire(), this.#duration);
    this.#raf = requestAnimationFrame(this.#tick);
  };

  pause = () => {
    if (!this.config.isTimer) return;

    clearTimeout(this.#timeout);
    this.#duration = Math.max(0, this.#duration - (performance.now() - this.#startedAt));
    if (this.#raf) {
      cancelAnimationFrame(this.#raf);
      this.#raf = null;
    }
  };

  destroy = () => {
    clearTimeout(this.#timeout);
    if (this.#raf) cancelAnimationFrame(this.#raf);
    this.#onExpire();
  };
}

let id = 0;
export const alerts = new SvelteMap<number, Alert>();

export const sendAlert = (config: AlertConfig) => {
  const _id = ++id;
  alerts.set(_id, new Alert(_id, config, () => alerts.delete(_id)));
};

export const clearAlerts = () => {
  alerts.forEach((a) => a?.destroy());
  alerts.clear();
};