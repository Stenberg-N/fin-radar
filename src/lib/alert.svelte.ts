import { SvelteMap } from "svelte/reactivity";

import type { Alert } from "./types";

export const alerts: SvelteMap<number, Alert> = new SvelteMap();
let id = 0;

/**
 * - `message` can take any string. The string is first checked if it is a key for translations. If not, the string will act as the message as is.
 *   - You can also provide the message from the translations store itself by using `$t` or `get(t)`,
 *   but keep in mind that the alert message will not get updated if the localization is changed while the alert is still visible.
 * - `isTimer` determines if the alert should remain or die after 2.5 seconds.
 * - `buttons` determines if buttons for confirming and canceling should be visible.
 * - `onConfirm` and `onCancel` attach a function to their respective buttons.
 * - `onlyConfirmButton` determines if only the confirm button is visible.
 * - i18nKeys for cancel and confirm buttons are used to optionally change the translation used for the button.
 * - `additionalText` when given as an array will place each item as their own span element, each on their own row.
 * - `placeTextOnNewRow` creates a br element and a span element that is a border, creating a divider between the message and additionalText.
 * - `selector` is an object used for passing data to an ItemSelect component for the alert.
 *   - `selectorData` is used for the select options.
 *   - `selectorOverrideData` is used to override the selectorData items. It is visual-only.
 */
export const sendAlert = (options: {
  isTimer: boolean;
  buttons: boolean;
  message: string;
  onConfirm?: () => void;
  onCancel?: () => void;
  onlyConfirmButton?: boolean;
  confirmButtonI18nKey?: string;
  cancelButtonI18nKey?: string;
  additionalText?: string | string[];
  placeTextOnNewRow?: boolean;
  isConfirmButtonWhite?: boolean;
  selector?: {
    selectorData: readonly string[];
    selectorOverrideData?: string[];
  }
}) => {
  const alert: Alert = {
    id: ++id,
    message: options.message,
    isTimer: options.isTimer,
    buttons: options.buttons,
    onConfirm: options.onConfirm ?? (() => {}),
    onCancel: options.onCancel ?? (() => {}),
    onlyConfirmButton: options.onlyConfirmButton ?? false,
    confirmButtonI18nKey: options.confirmButtonI18nKey ?? "confirm.button",
    cancelButtonI18nKey: options.cancelButtonI18nKey ?? "cancel.button",
    additionalText: options.additionalText,
    placeTextOnNewRow: options.placeTextOnNewRow ?? false,
    isConfirmButtonWhite: options.isConfirmButtonWhite ?? false,
    selector: options.selector
  };
  alerts.set(alert.id, alert);
};

export const close = (id: number) => alerts.delete(id);

export const closeAll = () => alerts.clear();