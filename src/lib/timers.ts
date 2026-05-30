import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { lang } from "./i18n";
import type { Timer } from "./types";
import { sendAlert } from "./alert";

interface TimerRuntimeState {
  isRunning: boolean;
  currentDuration: number;
}

export const timers = writable<Timer[]>([]);
export const timerRuntimes = writable<Map<number, TimerRuntimeState>>(new Map());

const updateBatch: Timer[] = [];
let flushInterval: ReturnType<typeof setInterval> | null = null;
const timerIntervalMap = new Map<number, ReturnType<typeof setInterval>>();

const flushBatch = async (userId: number, username: string) => {
  if (!updateBatch.length) return;
  const batch = updateBatch.splice(0);
  const result = await updateTimer(userId, username, batch);
  if (!result.success) sendAlert("alert.update-timer.fail", true, false);
};

export const startTimerBatchFlush = (userId: number, username: string) => {
  if (flushInterval) return;
  flushInterval = setInterval(() => flushBatch(userId, username), 2000);
};

export const stopTimerBatchFlush = async (userId: number, username: string) => {
  if (flushInterval) clearInterval(flushInterval);
  flushInterval = null;
  await flushBatch(userId, username);
};

export const queueTimerUpdate = (timer: Timer) => {
  const idx = updateBatch.findIndex(t => t.id === timer.id);
  if (idx !== -1) updateBatch[idx] = timer;
  else updateBatch.push(timer);
};

export const startTimerCountdown = (timerId: number) => {
  stopTimerCountdown(timerId);

  const interval = setInterval(() => {
    const current = get(timerRuntimes).get(timerId);
    if (!current) return;

    if (current.currentDuration <= 0) {
      timerRuntimes.update((map) => map.set(timerId, { isRunning: false, currentDuration: 0 }));
      stopTimerCountdown(timerId);

      const timerData = get(timers).find(t => t.id === timerId);
      if (timerData) queueTimerUpdate({ ...timerData, duration: 0 });

      const timer = get(timers).find(t => t.id === timerId);
      if (timer) sendAlert(timer.title, false, false, undefined, undefined, timer.message, true);
      return;
    }
    timerRuntimes.update((map) => map.set(timerId, { ...current, currentDuration: current.currentDuration - 1 }));
  }, 1000);
  timerIntervalMap.set(timerId, interval);
};

export const stopTimerCountdown = (timerId: number) => {
  clearInterval(timerIntervalMap.get(timerId));
  timerIntervalMap.delete(timerId);
};

export const createTimer = async (userId: number) => {
  const title = get(lang) === 'en' ? "New timer" : "Uusi ajastin";
  const duration = 0;
  try {
    const result = await invoke<Timer>('create_timer', { userId: userId, duration: duration, title: title, message: undefined });
    timers.update((timers) => [ ...timers, result ]);
    timerRuntimes.update((map) => { map.set(result.id, { isRunning: false, currentDuration: result.duration }); return map; });

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
    stopTimerCountdown(timerId);
    timerRuntimes.update((map) => { map.delete(result.id); return map; });

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const reorderTimer = () => {

};