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
let latestX = 0;
let latestY = 0;

const applyCursorPosition = () => {
  raf = null;
  viewport.update((current) => ({ ...current, cursorY: latestY, cursorX: latestX }));
};

export const handleCursorPositionUpdate = (e: MouseEvent) => {
  latestX = e.clientX;
  latestY = e.clientY;

  if (raf === null) {
    raf = requestAnimationFrame(applyCursorPosition);
  }
};