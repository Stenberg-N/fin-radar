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
export const toIsoDate = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
export const toYearMonth = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}`;
export const formatMonthYear = (d: Date) => `${(i18n.t["calendar.monthnames"] as string[])[d.getMonth()]}, ${d.getFullYear()}`;