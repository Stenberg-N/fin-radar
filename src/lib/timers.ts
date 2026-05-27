import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { lang } from "./i18n";
import type { Timer } from "./types";

export const timers = writable<Timer[]>([]);

export const createTimer = async (userId: number) => {
  const title = get(lang) === 'en' ? "New timer" : "Uusi ajastin";
  const duration = 0;
  try {
    const result = await invoke<Timer>('create_timer', { userId: userId, duration: duration, title: title, message: undefined });
    timers.update((timers) => [ ...timers, result ]);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const getTimers = async (userId: number, username: string) => {
  try {
    const result = await invoke<Timer[]>('get_timers', { userId: userId, username: username });
    timers.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const updateTimer = async (userId: number, username: string, timerArray: Timer[]) => {
  try {
    const result = await invoke<Timer[]>('update_timer', { userId: userId, username: username, timerArray: timerArray });
    timers.update((timers) =>
      timers.map((timer) => {
        const updatedTimer = result.find(t => t.id === timer.id);
        return updatedTimer ? { ...timer, duration: updatedTimer.duration, title: updatedTimer.title, message: updatedTimer.message } : timer;
      })
    );

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const deleteTimer = async (userId: number, username: string, timerId: number) => {
  try {
    const result = await invoke<Timer>('delete_timer', { userId: userId, username: username, timerId: timerId });
    timers.update((timers) => [ ...timers.filter(t => t.id !== result.id) ]);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const reorderTimer = () => {

};