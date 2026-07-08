import { writable } from "svelte/store";

import type { ViewStore } from "./types";

export const viewStore = writable<ViewStore>({
  isMenu: false,
  isChangePwOverlay: false,
  isRecoveryView: false,
  isTimersMenu: false,
  isAskPassword: false,
  isNavBarCollapsed: false,
});

export const setViewState = (options: {viewState: keyof ViewStore, state?: boolean, toggle?: boolean}) => {
  viewStore.update((current) => {    
    const newValue = options.toggle ? !current[options.viewState] : (options.state !== undefined ? options.state : current[options.viewState]);
    return { ...current, [options.viewState]: newValue };
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