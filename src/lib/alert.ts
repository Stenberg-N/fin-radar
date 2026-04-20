import { writable } from "svelte/store";
import { type Alert } from "./types";

export const alerts = writable<Alert[]>([]);

let id = 0;

export const sendAlert = (
  message: string,
  isTimer: boolean,
  buttons: boolean,
  onConfirm?: () => void,
  onCancel?: () => void,
  value?: string
) => {
  const alert: Alert = {
    id: ++id,
    message,
    isTimer: isTimer,
    buttons: buttons,
    onConfirm: onConfirm || (() => {}),
    onCancel: onCancel || (() => {}),
    value: value || '',
  };
  alerts.update((alerts) => [ ...alerts, alert ]);
};

export const close = (id: number) => alerts.update((alerts) => alerts.filter((alert) => alert.id !== id));

export const closeAll = () => {
  alerts.set([]);
}