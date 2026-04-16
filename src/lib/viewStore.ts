import { writable } from "svelte/store";

import type { ViewStore } from "./types";

export const viewStore = writable<ViewStore>({
  isMenu: false,
  isChangePwOverlay: false,
  isRecoveryView: false,
  isCalendar: false,
});

export const setViewState = (viewState: keyof ViewStore, state?: boolean, toggle?: boolean) => {
  viewStore.update((current) => {    
    const newValue = toggle ? !current[viewState] : (state !== undefined ? state : current[viewState]);
    return { ...current, [viewState]: newValue };
  });
};

export const resetViewStates = () => {
  viewStore.update(current => {
    const keys = Object.keys(current) as Array<keyof ViewStore>;
    const resetState = keys.reduce((acc, key) => {
      acc[key] = false;
      return acc;
    }, {} as ViewStore);
    return resetState;
  });
};