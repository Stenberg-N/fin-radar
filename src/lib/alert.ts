import { writable } from "svelte/store";
import { type Alert } from "./types";
export const alerts = writable<Alert[]>([]);

let id = 0;

/**
  * Message can take any variable + strings. Strings are first checked if they are in translations. If not, the string will act as the message as is.
  * You can also provide the message from the translations store itself by using `$t` or `get(t)`, but keep in mind that the alert message will not get updated if the localization is changed while the alert is still visible.
  *
  * IsTimer determines if the alert should remain or die after 2.5 seconds.
  * 
  * Buttons determines if buttons for confirming and canceling should be visible.
  * 
  * OnConfirm and onCancel attach a function to their respective buttons.
  * 
  * onlyConfirmButton determines if only the confirm button is shown.
  * 
  * i18nKeys for cancel and confirm buttons are used to optionally change the translation used for the button.
  * 
  * AdditionalText when given as an array will place each item as their own span element, each on their own row.
  * 
  * PlaceTextOnNewRow creates a br element and a span element that is a border, creating a divider between the message and additionalText. 
*/
export const sendAlert = (options: {
  message: string,
  isTimer: boolean,
  buttons: boolean,
  onlyConfirmButton?: boolean;
  onConfirm?: () => void,
  confirmButtonI18nKey?: string;
  onCancel?: () => void,
  cancelButtonI18nKey?: string;
  additionalText?: string | string[];
  placeTextOnNewRow?: boolean;
  isConfirmButtonWhite?: boolean;
}) => {
  const alert: Alert = {
    id: ++id,
    message: options.message,
    isTimer: options.isTimer,
    buttons: options.buttons,
    onlyConfirmButton: options.onlyConfirmButton || false,
    onConfirm: options.onConfirm || (() => {}),
    confirmButtonI18nKey: options.confirmButtonI18nKey || "confirm.button",
    onCancel: options.onCancel || (() => {}),
    cancelButtonI18nKey: options.cancelButtonI18nKey || "cancel.button",
    additionalText: options.additionalText || '',
    placeTextOnNewRow: options.placeTextOnNewRow || false,
    isConfirmButtonWhite: options.isConfirmButtonWhite || false,
  };
  alerts.update((alerts) => [ ...alerts, alert ]);
};

export const close = (id: number) => alerts.update((alerts) => alerts.filter((alert) => alert.id !== id));

export const closeAll = () => {
  alerts.set([]);
};