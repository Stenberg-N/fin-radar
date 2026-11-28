import { sendAlert } from "../classes/alertStore.svelte";
import { i18n } from "../i18n/i18n.svelte";

export const handleKeyDownOnInput = (command: string, event: KeyboardEvent) => {
  const allowedKeys = ["Escape", "Enter", "Backspace", "Delete", "ArrowLeft", "ArrowRight", "Tab", "Home", "End", "Control"];
  const regex = /^[0-9\-]+$/g;

  switch (command) {
    case "amount": {
      if (event.key === ",") {
        event.preventDefault();
        sendAlert({ message: "alert.add-transaction.amount.comma", isTimer: true, buttons: false });
      }
      if (event.key === "-") {
        event.preventDefault();
        sendAlert({ message: "alert.add-transaction.amount.minus", isTimer: true, buttons: false });
      }
      break;
    }
    case "date": {
      if (allowedKeys.includes(event.key)) return;
      if (event.ctrlKey && (event.key.toLowerCase() === 'z' || event.key.toLowerCase() === 'a')) return;

      if (!regex.test(event.key)) {
        event.preventDefault();
        sendAlert({ message: "alert.date-input.invalid", isTimer: true, buttons: false});
      }
      break;
    }
  }
};

export const handleNumberInput = (target: EventTarget | null) => {
  if (!target) return;

  const node = target as HTMLInputElement;
  const value = Number(node.value);
  if (value < 0) node.value = "0";
};

export const handleDate = (date: string) => {
  let [year, month] = date.split("-");

  const idx = parseInt(month) - 1;
  const monthNames = i18n.t["calendar.monthnames"] as string[];
  month = monthNames[idx];
  return month ? `${year} ${month}` : `${year}`;
};

export const capitalizeString = (string: string) => {
  return string.slice(0, 1).toUpperCase() + string.slice(1);
};

const pad = (n: number) => String(n).padStart(2, '0');

/**
 * Returns an ISO date from the given Date object.
 * 
 * Adds padding to the month and day parts for them to be always two numbers long.
 * 
 * @example
 * const date = new Date(2026, 10, 1);
 * const formatted = toIsoDate(date);
 * console.log(formatted); // => 2026-11-01
 */
export const toIsoDate = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;

/**
 * Returns a year-month in ISO date format from the given Date object.
 * 
 * Adds padding to month to always be 2 numbers long, i.e. 1 becomes 01, 2 becomes 02 and so on.
 * 
 * @example
 * const date = new Date(2026, 10, 1);
 * const formatted = toYearMonth(date);
 * console.log(formatted); // => 2026-11
 */
export const toYearMonth = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}`;

/**
 * Returns a more user-friendly/readable year-month stamp from the given Date object.
 * 
 * @example
 * const date = new Date(2026, 10, 1);
 * const formatted = formatMonthYear(date);
 * console.log(formatted); // => "November, 2026"
 */
export const formatMonthYear = (d: Date) => `${(i18n.t["calendar.monthnames"] as string[])[d.getMonth()]}, ${d.getFullYear()}`;

/**
 * Used to warn the user about unsaved changes before proceeding.
 * 
 * @param condition A boolean variable that determines if an alert is to be sent about unsaved changes.
 * @param action A callback to run when the condition is false or the confirm button is hit on the alert.
 * @param msgs Used to override the default messages for the alert. Pass down a single string or an array of two strings. The first string will be shown first and the second string, if provided, will be on its own line on the next row.
 */
export const alertBeforeProceeding = (condition: boolean, action: () => void, msgs: string[] | string = [(i18n.t["alert.unsaved-changes"] as string[])[0], i18n.t["alert.navigate-without-saving"] as string]) => {
  condition
    ? sendAlert({
      message: Array.isArray(msgs) ? msgs[0] : msgs,
      isTimer: false,
      buttons: true,
      onConfirm: action,
      additionalText: Array.isArray(msgs) ? [msgs[1]] : undefined,
    })
    : action();
};