import { get } from "svelte/store";
import { getContext } from "svelte";

import { sendAlert } from "./alert";
import { t } from "./i18n";
import { isDragging } from "./dragAndDrop";

export const handleClickOutside = (
  node: HTMLElement,
  options: {
    onOutsideClick: () => void;
    additionalElements?: (HTMLElement | null)[];
  }
) => {
  const { onOutsideClick, additionalElements } = options;
  const getIgnoredElements = getContext<() => (HTMLElement | null)[]>('ignoredElements');

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
  const monthNames = get(t)["calendar.monthnames"] as string[];
  month = monthNames[idx];
  return month ? `${year} ${month}` : `${year}`;
};

export const handleHorizontalScroll = (node: HTMLElement, options?: { scrollMultiplier: number }) => {
  const scrollMultiplier = options?.scrollMultiplier ?? 1;  

  const handleScroll = (e: WheelEvent) => {
    e.preventDefault();
    e.stopPropagation();
    node.scrollLeft += e.deltaY * scrollMultiplier;
  };

  node.addEventListener('wheel', handleScroll, { passive: false });
  return { destroy: () => node.removeEventListener('wheel', handleScroll)};
};

export const handleAutoScroll = (
  node: HTMLElement,
  options: {
    querySelector: string; // Used to find the scrollable content from the parent using its class.
    scrollSpeedMultiplier?: "slower" | "faster";
  }
) => {
  if (!node) return;

  const parentEl = node.getBoundingClientRect();
  const querySelector = options.querySelector;
  const scrollSpeedMultiplier = options?.scrollSpeedMultiplier ?? "slower";

  const target = node.querySelector(`.${querySelector}`) as HTMLDivElement;
  if (!target) return;

  const MIN_THRESHOLD = 50;
  const TARGET_WIDTH = target.clientWidth;
  const PARENT_WIDTH = node.clientWidth;

  let pointerPosInEl: number | null = null;
  let raf: number | null = null;
  let isCursorInNode = false;
  let speedMultiplier = 1;

  const scrollStep = () => {
    if (!isCursorInNode || pointerPosInEl === null || !get(isDragging)) {
      raf = null;
      return;
    }

    if (pointerPosInEl <= MIN_THRESHOLD) target.scrollLeft -= 4 * speedMultiplier;
    else if (pointerPosInEl >= TARGET_WIDTH && pointerPosInEl <= PARENT_WIDTH) target.scrollLeft += 4 * speedMultiplier;
    speedMultiplier += scrollSpeedMultiplier === "slower" ? .01 : 0.12;
  
    raf = requestAnimationFrame(scrollStep);
  };

  const startScrolling = () => {
    if (raf) return;
    raf = requestAnimationFrame(scrollStep);
  };

  const stopScrolling = () => {
    if (raf !== null) {
      cancelAnimationFrame(raf);
      raf = null;
    }
    speedMultiplier = 1;
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!node.contains(e.target as Node)) {
      isCursorInNode = false;
      stopScrolling();
      return;
    }

    isCursorInNode = true;
    pointerPosInEl = e.clientX - parentEl.left;

    const inLeftZone = pointerPosInEl >= 0 && pointerPosInEl <= MIN_THRESHOLD;
    const inRightZone = pointerPosInEl >= TARGET_WIDTH && pointerPosInEl <= PARENT_WIDTH;

    if (get(isDragging) && ((inLeftZone && target.scrollLeft > 0) || (inRightZone && target.scrollLeft < (target.scrollWidth - TARGET_WIDTH)))) startScrolling();
    else stopScrolling();
  };

  const handleMouseLeave = () => {
    isCursorInNode = false;
    stopScrolling();
  };

  window.addEventListener('mousemove', handleMouseMove, { passive: true });
  window.addEventListener('mouseleave', handleMouseLeave);

  return {
    destroy: () => {
      stopScrolling();
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseleave', handleMouseLeave);
    }
  }
};

export const capitalizeString = (string: string) => {
  return string.slice(0, 1).toUpperCase() + string.slice(1);
};