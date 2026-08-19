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

export const handleCursorPositionUpdate = (e: MouseEvent) => {
  if (!raf) {
    requestAnimationFrame(() => {
      viewport.update((current) => ({ ...current, cursorX: e.clientX }));
      viewport.update((current) => ({ ...current, cursorY: e.clientY }));
      raf = null;
    });
  }
};