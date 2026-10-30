import { get } from "svelte/store";
import { getContext } from "svelte";

import { isDragging } from "./dragAndDrop";
import { viewport } from "./viewport";

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
    if (node.scrollWidth <= node.clientWidth) return;

    let el = e.target as HTMLElement | null;
    while (el && el !== node) {
      if (el.scrollHeight > el.clientHeight && /(auto)/.test(getComputedStyle(el).overflowY)) return;
      el = el.parentElement;
    }

    e.preventDefault();
    node.scrollLeft += e.deltaY * scrollMultiplier;
  };

  node.addEventListener('wheel', handleScroll, { passive: false });
  return { destroy: () => node.removeEventListener('wheel', handleScroll)};
};

export const handleAutoScroll = (node: HTMLElement) => {
  let rect: DOMRect | null = null;
  let isCursorInNode: boolean = false;
  let pointerPos: number = 0;
  let inLeftZone: boolean = false;
  let inRightZone: boolean = false;
  let raf: number | null = null;
  let mult: number = 1;

  const start = () => {
    if (raf) return;
    raf = requestAnimationFrame(scroll);
  };

  const stop = () => {
    if (raf) {
      cancelAnimationFrame(raf);
      raf = null;
    }
    mult = 1;
  };

  const enter = () => {
    isCursorInNode = true;
    rect = node.getBoundingClientRect();
    node.addEventListener('mousemove', move);
    node.addEventListener('mouseleave', leave);
  };

  const leave = () => {
    isCursorInNode = false;
    stop();
    node.removeEventListener('mouseleave', leave);
    node.removeEventListener('mousemove', move);
  };

  const move = (e: MouseEvent) => {
    if (!isCursorInNode || !get(isDragging)) return;

    pointerPos = e.clientX - (rect?.left ?? 0);
    inLeftZone = pointerPos >= 0 && pointerPos <= 50;
    inRightZone = pointerPos >= node.clientWidth - 50 && pointerPos <= node.clientWidth;

    if (get(isDragging) && (inLeftZone || inRightZone)) start();
    else stop();
  };

  const scroll = () => {
    if (!isCursorInNode || !get(isDragging)) {
      raf = null;
      return;
    }

    if (inLeftZone && node.scrollLeft > 0) node.scrollLeft -= 4 * mult;
    else if (inRightZone && node.scrollLeft < node.scrollWidth - node.clientWidth) node.scrollLeft += 4 * mult;

    mult += 0.016;
    raf = requestAnimationFrame(scroll);
  };

  node.addEventListener('mouseenter', enter);

  return {
    destroy: () => {
      node.removeEventListener('mouseenter', enter);
      node.removeEventListener('mouseleave', leave);
      node.removeEventListener('mousemove', move);
      if (raf) cancelAnimationFrame(raf);
    }
  };
};

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
  let panelLeft = 0;
  let offset = 0;
  let raf: number | null = null;
  let latestClientX = 0;

  const applyResize = () => {
    raf = null;
    const newWidth = Math.min(options.max, Math.max(options.min, latestClientX));
    options.onResize(newWidth);
  };

  const handlePointerDown = (e: PointerEvent) => {
    const panel = node.previousElementSibling as HTMLElement | null;
    const rect = panel?.getBoundingClientRect();
    panelLeft = rect?.left ?? 0;
    offset = (rect?.right ?? e.clientX) - e.clientX;

    node.setPointerCapture(e.pointerId);
    node.addEventListener('pointermove', handlePointerMove);
    node.addEventListener('pointerup', handlePointerUp);
  };

  const handlePointerMove = (e: PointerEvent) => {
    if (threshold) {
      const pos = e.clientX - panelLeft + offset;
      latestClientX = pos < lowerLimit ? threshold.jumpTo : Math.max(pos, threshold.at);
    } else {
      latestClientX = e.clientX - panelLeft + offset;
    }

    if (raf === null) {
      raf = requestAnimationFrame(applyResize);
    }
  };

  const handlePointerUp = (e: PointerEvent) => {
    if (raf !== null) cancelAnimationFrame(raf);

    node.releasePointerCapture(e.pointerId);
    raf = null;

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

export const portalElement = (node: HTMLElement, target = document.body) => {
  target.appendChild(node);

  return {
    destroy: () => {
      node.remove();
    }
  };
};