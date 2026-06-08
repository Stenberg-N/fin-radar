import { writable } from "svelte/store";

type ViewportState = {
  width: number;
  height: number;
  cursorX: number;
  cursorY: number;
}

export const viewport = writable<ViewportState>({
  width: 0,
  height: 0,
  cursorX: 0,
  cursorY: 0,
});

let raf: number | null = null;
let nextX = 0;
let nextY = 0;

export const handleCursorPositionUpdate = (e: MouseEvent) => {
  nextX = e.clientX;
  nextY = e.clientY;

  if (!raf) {
    requestAnimationFrame(() => {
      viewport.update((current) => ({ ...current, cursorX: nextX }));
      viewport.update((current) => ({ ...current, cursorY: nextY }));
      raf = null;
    });
  }
};