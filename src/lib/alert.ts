import { writable } from "svelte/store";
import { type Alert } from "./types";

/*

* Message can take any variable + strings. Strings are first checked if they are in translations. If not, the string will act as the message as is.
* IsTimer determines if the alert should remain or die after 2.5 seconds.
* Buttons determines if buttons for confirming and canceling should be visible.
* OnConfirm and onCancel attach a function to their respective buttons.
* AdditionalText when given as an array will place each item as their own <span>, each on their own row.
* PlaceTextOnNewRow creates a <br> and a <span> that is a border, creating a divider between the message and additionalText.

*/

export const alerts = writable<Alert[]>([]);

let id = 0;

export const sendAlert = (
  message: string,
  isTimer: boolean,
  buttons: boolean,
  onConfirm?: () => void,
  onCancel?: () => void,
  additionalText?: string | string[],
  placeTextOnNewRow?: boolean  
) => {
  const alert: Alert = {
    id: ++id,
    message,
    isTimer: isTimer,
    buttons: buttons,
    onConfirm: onConfirm || (() => {}),
    onCancel: onCancel || (() => {}),
    additionalText: additionalText || '',
    placeTextOnNewRow: placeTextOnNewRow || false
  };
  alerts.update((alerts) => [ ...alerts, alert ]);
};

export const close = (id: number) => alerts.update((alerts) => alerts.filter((alert) => alert.id !== id));

export const closeAll = () => {
  alerts.set([]);
}