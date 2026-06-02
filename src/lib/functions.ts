import { sendAlert } from "./alert";
import { t } from "./i18n";
import { get, type Writable } from "svelte/store";
import { type Timer, type Note, type Tab } from "./types";
import { checkTimerRuntimes } from "./timers";

export const validatePassword = (pw: string) => {
  const hasMinLength = pw.length >= 10;
  const noSpaces = !/\s/.test(pw);
  const hasNumbers = /\d/.test(pw);
  const hasUpperCase = /\p{Lu}+/gu.test(pw);
  const hasLowerCase = /\p{Ll}+/gu.test(pw);
  const hasSpecialChar = /[=\]!@#$€£¤%^&*(){}\[,.?+<>~§'":|/\\-]/.test(pw);

  return {
    isValid: hasMinLength && noSpaces && hasNumbers && hasUpperCase && hasLowerCase && hasSpecialChar,
  };
};

export const handleClickOutside = (
  node: HTMLElement,
  options: {
    getIgnoredElements: () => (HTMLButtonElement | HTMLDivElement | null)[];
    onOutsideClick: () => void;
    additionalElements?: (HTMLButtonElement | HTMLDivElement | null)[];
  }
) => {
  const { getIgnoredElements, onOutsideClick, additionalElements } = options;
  const handleClick = (e: MouseEvent) => {
    const target = e.target as Node;
    const ignored = getIgnoredElements();
    additionalElements?.forEach(el => { ignored.push(el); });

    if (node.contains(target)) return;

    for (const el of ignored) {
      if (el?.contains(target)) return;
    }

    onOutsideClick();
  };

  document.addEventListener('click', handleClick, true);

  return { destroy() { document.removeEventListener('click', handleClick, true); } };
};

export const togglePasswordVisibility = (button: EventTarget | null) => {
  if (!button) return;

  const node = button as HTMLButtonElement;
  const passwordInput = node.previousElementSibling as HTMLInputElement | null;
  const img = node.firstChild as HTMLImageElement | null;

  if (passwordInput && img) {
    const isPassword = passwordInput.type === "password";
    passwordInput.type = isPassword ? "text" : "password";
    img.src = isPassword ? "/eye-hidden.svg" : "/eye-visible.svg";
  }
};

export const handleKeyDownOnInput = (command: string, event: KeyboardEvent) => {
  const allowedKeys = ["Escape", "Enter", "Backspace", "Delete", "ArrowLeft", "ArrowRight", "Tab", "Home", "End", "Control"];
  const regex = /^[0-9\-]+$/g;

  switch (command) {
    case "amount": {
      if (event.key === ",") {
        event.preventDefault();
        sendAlert("alert.add-transaction.amount.comma", true, false);
      }
      if (event.key === "-") {
        event.preventDefault();
        sendAlert("alert.add-transaction.amount.minus", true, false);
      }
      break;
    }
    case "date": {
      if (allowedKeys.includes(event.key)) return;
      if (event.ctrlKey && (event.key.toLowerCase() === 'z' || event.key.toLowerCase() === 'a')) return;

      if (!regex.test(event.key)) {
        event.preventDefault();
        sendAlert("alert.add-transaction.date.input", true, false);
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
  const monthNames = get(t)["calendar.monthnames"] as string[];
  month = monthNames[idx];
  return month ? `${year} ${month}` : `${year}`;
};

export const handleHorizontalScroll = (node: HTMLElement, options?: { scrollMultiplier: number }) => {
  let scrollMult: number | null = null;

  if (options) {
   const { scrollMultiplier } = options;
   scrollMult = scrollMultiplier;
  }

  const handleScroll = (e: WheelEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const delta = scrollMult !== null ? e.deltaY * scrollMult : e.deltaY;
    node.scrollLeft += delta
  };

  node.addEventListener('wheel', handleScroll, { passive: false });
  return { destroy: () => node.removeEventListener('wheel', handleScroll)};
};

//
// DND (DRAG AND DROP)
//

const handleDragStart = (idx: number) => ({ dragIndex: idx });

const handleDragOver = (
  e: PointerEvent,
  idx: number,
  dragIndex: number | null
) => {
  e.preventDefault();
  if (dragIndex === null || dragIndex === idx) return { dragIndex };
  return { dragIndex: idx };
};

const handleDragEnd = <T extends Timer | Note | Tab>(
  array: Writable<T[]>,
  idx: number,
  dragIndex: number | null
) => {
  if (idx === null || dragIndex === null) return { dragIndex: dragIndex };

  const reordered = [...get(array)];
  const [movedItem] = reordered.splice(idx, 1);
  reordered.splice(dragIndex, 0, movedItem);
  array.update(() => reordered as T[]);
  return { dragIndex: null };
};

let ghostEl: HTMLElement | null = null;
let isDragging: boolean = false;
let lastMoveTime = 0;

const showGhost = (card: HTMLElement) => {
    ghostEl = card.cloneNode(true) as HTMLElement;
    ghostEl.style.cssText = `
      position: fixed;
      pointer-events: none;
      z-index: 1000;
      top: -9999px;
      left: -9999px;
      width: ${card.offsetWidth}px;
      opacity: 1;
      transform: rotate(2deg);
      box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
    `;
    document.body.appendChild(ghostEl);
  };

  const moveGhost = (e: PointerEvent) => {
    if (!ghostEl) return;

    ghostEl.style.left = `${e.clientX - ghostEl.offsetWidth / 2}px`;
    ghostEl.style.top = `${e.clientY - 20}px`;
  };

  const removeGhost = () => {
    ghostEl?.remove();
    ghostEl = null;
  };

  export const handlePointerDown = (e: PointerEvent, idx: number): { dragIndex: number | null } | void => {
    if (checkTimerRuntimes()) return;
    const target = e.currentTarget as HTMLElement;
    if (!target) return;

    target.setPointerCapture(e.pointerId);
    isDragging = true;
    const { dragIndex: newDragIndex } = handleDragStart(idx);
    showGhost(target.parentElement!);
    return { dragIndex: newDragIndex };
  };

  export const handlePointerMove = (e: PointerEvent, dragIndex: number | null, view: "timers" | "notes" | "tabs"): { dragIndex: number | null } | void => {
    if (!isDragging) return;

    moveGhost(e);
    const now = Date.now();
    if (now - lastMoveTime < 25) return;
    lastMoveTime = now;

    const els = document.elementsFromPoint(e.clientX, e.clientY);
    const card = els.find(el => el.classList.contains(view === "timers"
      ? "timer-container"
      : view === "notes"
        ? "notes-container"
        : "notes-tab-outer-container"
      ));
    if (!card) return;

    const index = Number((card as HTMLElement).dataset.index);
    if (index === dragIndex) return;
    const { dragIndex: newDragIndex } = handleDragOver(e, index, dragIndex);
    return { dragIndex: newDragIndex };
  };

  export const handlePointerUp = <T extends Timer | Note | Tab>(
    array: Writable<T[]>,
    idx: number,
    dragIndex: number | null
  ): { dragIndex: number | null } | void => {
    if (!isDragging) return;

    isDragging = false;
    removeGhost();
    const { dragIndex: newDragIndex } = handleDragEnd(array, idx, dragIndex);
    return { dragIndex: newDragIndex };
  };