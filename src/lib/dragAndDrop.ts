import { get, type Writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { type Timer, type Note, type Tab } from "./types";
import { checkTimerRuntimes } from "./timers";
import { user } from "./user";
import { sendAlert } from "./alert";

let ghostEl: HTMLElement | null = null;
let isDragging: boolean = false;
let lastMoveTime = 0;

const handleArraySave = async <T extends Timer | Note | Tab>(array: Writable<T[]>, arrayType: "timers" | "notes" | "tabs") => {
  const _user = get(user);
  const arrayIds = get(array).map(item => item.id);
  if (!_user || !arrayIds.length || !arrayType) return;

  try {
    await invoke('reorder_array', { userId: _user.id, username: _user.name, array: arrayIds, arrayType: arrayType });
  } catch (error) {
    switch (arrayType) {
      case "timers": sendAlert("alert.timer-reorder.fail", true, false); break;
    }
  }
};

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
  arrayType: "timers" | "notes" | "tabs",
  idx: number,
  dragIndex: number | null
) => {
  if (idx === null || dragIndex === null || idx === dragIndex) return { dragIndex: null };

  const reordered = [...get(array)];
  const [movedItem] = reordered.splice(idx, 1);
  reordered.splice(dragIndex, 0, movedItem);
  array.update(() => reordered.map((item, index) => ({ ...item, order_id: index + 1 })) as T[]);
  handleArraySave(array, arrayType);

  return { dragIndex: null };
};

const showGhost = (card: HTMLElement) => {
  ghostEl = card.cloneNode(true) as HTMLElement;
  ghostEl.style.cssText = `
    position: fixed;
    pointer-events: none;
    z-index: 1000;
    top: -9999px;
    left: -9999px;
    width: ${card.offsetWidth}px;
    opacity: 0;
    transform: rotate(0deg) scale(1);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    transition: transform 0.15s, box-shadow 0.15s, opacity 0.15s;
  `;
  document.body.appendChild(ghostEl);

  requestAnimationFrame(() => {
    if (!ghostEl) return;
    ghostEl.style.transform = 'rotate(3deg) scale(0.9)';
    ghostEl.style.boxShadow = '0 8px 16px rgba(0, 0, 0, 0.8)';
    ghostEl.style.opacity = '1';
  });
};

const moveGhost = (e: PointerEvent) => {
  if (!ghostEl) return;

  ghostEl.style.left = `${e.clientX - ghostEl.offsetWidth / 2}px`;
  ghostEl.style.top = `${e.clientY - 20}px`;
};

const removeGhost = () => {
  if (!ghostEl) return;
  ghostEl.style.transform = 'rotate(0deg) scale(1)';
  ghostEl.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.8)';
  ghostEl.style.opacity = '0';
  ghostEl.addEventListener('transitionend', () => { ghostEl?.remove(); ghostEl = null; }, { once: true });
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
  arrayType: "timers" | "notes" | "tabs",
  idx: number,
  dragIndex: number | null
): { dragIndex: number | null } | void => {
  if (!isDragging) return;

  isDragging = false;
  removeGhost();
  const { dragIndex: newDragIndex } = handleDragEnd(array, arrayType, idx, dragIndex);
  return { dragIndex: newDragIndex };
};