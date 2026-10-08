import { get } from "svelte/store";
import { getContext } from "svelte";

import { sendAlert } from "./alert";
import { i18n } from "./i18n/i18n.svelte";
import { isDragging } from "./dragAndDrop";
import { viewport } from "./viewport";

//
//
//
//  HELPERS ETC.
//
//
//

export class HoverTitle<T = undefined> {
  isHovering = $state(false);
  target = $state<T | undefined>(undefined);
  #timeout: ReturnType<typeof setTimeout> | null = null;
  #delay: number;
  #ignoreSelector: string;
  #watched: Element | null = null;
  static #count = 0;
  readonly id = `hover-${HoverTitle.#count++}`;

  constructor(delay = 500) {
    this.#delay = delay;
    this.#ignoreSelector = `[hover-title-owner="${this.id}"]`;
  }

  #unwatch = () => {
    this.#watched?.removeEventListener('mouseleave', this.#onLeave);
    this.#watched = null;
  };

  #hide = () => {
    if (this.#timeout) clearTimeout(this.#timeout);
    this.#timeout = null;
    this.isHovering = false;
  };

  #onLeave = () => {
    this.#unwatch();
    this.#hide();
  };

  enter = (target?: T) => {
    this.#unwatch();
    if (this.#timeout) clearTimeout(this.#timeout);
    this.target = target;

    this.#timeout = setTimeout(() => this.isHovering = true, this.#delay);
  };

  leave = (e?: MouseEvent) => {
    const next = e?.relatedTarget;
    const trigger = e?.currentTarget;
    const modal = next instanceof Element ? next.closest(this.#ignoreSelector) : null;
    const isOwnModal = modal && !(trigger instanceof Node && modal.contains(trigger));

    if (isOwnModal) {
      this.#unwatch();
      this.#watched = modal;
      modal.addEventListener('mouseleave', this.#onLeave);
      return;
    }
    
    this.#onLeave();
  };

  destroy = () => {
    this.#unwatch();
    if (this.#timeout) clearTimeout(this.#timeout);
  };
}

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

//
//
//
//  ACTIONS
//
//
//

export const handleClickOutside = (
  node: HTMLElement,
  options: {
    onOutsideClick: () => void;
    getAdditionalElements?: () => (HTMLElement | null)[];
  }
) => {
  let opts = options;
  const getIgnoredElements = getContext<() => (HTMLElement | null)[]>('ignoredElements');

  const handleClick = (e: MouseEvent) => {
    const target = e.target as Node;
    if (node.contains(target)) return;

    const ignored = [...getIgnoredElements(), ...(opts.getAdditionalElements?.() ?? [])];
    if (ignored.some((el) => el?.contains(target))) return;

    opts.onOutsideClick();
  };

  document.addEventListener('click', handleClick, true);

  return {
    destroy: () => { document.removeEventListener('click', handleClick, true); },
    update: (newOptions: typeof options) => { opts = newOptions; },
  };
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
  };
};

export const gutter = $state({ isMoving: false });
export const moveGutter = (
  node: HTMLElement,
  options: {
    min: number;
    max: number;
    onResize: (width: number) => void;
    threshold?: { at: number, jumpTo: number };
  }
) => {
  const threshold = options.threshold;
  const lowerLimit = threshold ? threshold.at / 2 : 0;
  let nodeWidth: number;
  let raf: number | null = null;
  let latestClientX = 0;

  const applyResize = () => {
    raf = null;
    const width = latestClientX - nodeWidth;
    const newWidth = Math.min(options.max, Math.max(options.min, width));
    options.onResize(newWidth);
  };

  const handlePointerDown = (e: PointerEvent) => {
    nodeWidth = node.getBoundingClientRect().width;
    node.setPointerCapture(e.pointerId);
    gutter.isMoving = true;

    node.addEventListener('pointermove', handlePointerMove);
    node.addEventListener('pointerup', handlePointerUp);
  };

  const handlePointerMove = (e: PointerEvent) => {
    if (threshold) {
      const pos = e.clientX - 12; // Offset the cursor to center it on the gutter.
      latestClientX = pos < lowerLimit ? threshold.jumpTo : Math.max(pos, threshold.at);
    }

    if (raf === null) {
      raf = requestAnimationFrame(applyResize);
    }
  };

  const handlePointerUp = (e: PointerEvent) => {
    if (raf !== null) cancelAnimationFrame(raf);

    node.releasePointerCapture(e.pointerId);
    raf = null;
    gutter.isMoving = false;

    node.removeEventListener('pointermove', handlePointerMove);
    node.removeEventListener('pointerup', handlePointerUp);
  };

  node.addEventListener('pointerdown', handlePointerDown);

  return {
    destroy: () => {
      if (raf !== null) cancelAnimationFrame(raf);
      node.removeEventListener('pointerup', handlePointerUp);
      node.removeEventListener('pointerdown', handlePointerDown);
      node.removeEventListener('pointermove', handlePointerMove);
    }
  };
};

export const draggedElement = $state({ isDragged: false });
export const dragElement = (
  node: HTMLElement,
  options: {
    elToMove: HTMLElement | null;
    onMove: (top: number, left: number) => void;
  }
) => {
  let opts = options;
  let vp = get(viewport);
  let raf: number | null = null;
  let elRect: DOMRect;
  let positions: { firstX: number, firstY: number, latestX: number, latestY: number } = { firstX: 0, firstY: 0, latestX: 0, latestY: 0 };

  const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

  const applyPosition = () => {
    raf = null;
    const posX = positions.latestX + (elRect.left - positions.firstX);
    const posY = positions.latestY + (elRect.top - positions.firstY);

    opts.onMove(
      clamp(posY, 0, vp.height - elRect.height),
      clamp(posX, 0, vp.width - elRect.width)
    );
  };

  const handleDragStart = (e: PointerEvent) => {
    if (!opts.elToMove) return;

    elRect = opts.elToMove.getBoundingClientRect();
    vp = get(viewport);
    positions.firstX = e.clientX;
    positions.firstY = e.clientY;

    draggedElement.isDragged = true;

    node.setPointerCapture(e.pointerId);
    node.addEventListener('pointermove', handleDragMove);
    node.addEventListener('pointerup', handleDragEnd);
  };

  const handleDragMove = (e: PointerEvent) => {
    positions.latestX = e.clientX;
    positions.latestY = e.clientY;

    if (!raf) raf = requestAnimationFrame(applyPosition);
  };

  const handleDragEnd = (e: PointerEvent) => {
    if (raf) cancelAnimationFrame(raf);
    raf = null;

    draggedElement.isDragged = false;

    node.releasePointerCapture(e.pointerId);
    node.removeEventListener('pointermove', handleDragMove);
    node.removeEventListener('pointerup', handleDragEnd);
  };

  node.addEventListener('pointerdown', handleDragStart);

  return {
    destroy: () => {
      if (raf) cancelAnimationFrame(raf);
      node.removeEventListener('pointerdown', handleDragStart);
      node.removeEventListener('pointermove', handleDragMove);
      node.removeEventListener('pointerup', handleDragEnd);
    },
    update: (newOptions: typeof options) => {
      opts = newOptions;
    }
  };
};