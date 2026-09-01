import { Store, load } from "@tauri-apps/plugin-store";
import { get, writable } from "svelte/store";

import { user } from "./user";
import { sendAlert } from "./alert";

type NotePrefs = {
  noteColumns: number;
  noteHeight: "50%" | "100%";
  noteBgColor: "dark" | "light";
  mainBgColor: "dark" | "light";
};

type MainPrefs = {
  isNavBarCollapsed: boolean;
};

type UserPrefsStore = {
  notePrefs: NotePrefs;
  mainPrefs: MainPrefs;
};

const DEFAULT_PREFS: UserPrefsStore = {
  mainPrefs: {
    isNavBarCollapsed: false,
  },
  notePrefs: {
    noteColumns: 4,
    noteHeight: "100%",
    noteBgColor: "dark",
    mainBgColor: "dark",
  }
};

let store: Store;
export const userPrefs = writable<UserPrefsStore>(DEFAULT_PREFS);

export const loadUserPrefs = async () => {
  const _user = get(user);
  if (!_user) return;

  try {
    store = await load('user-preferences.json', { defaults: { autoSave: false } });
    let prefs = await store.get<UserPrefsStore>(`${_user.id}`);

    if (!prefs || !prefs.mainPrefs || !prefs.notePrefs) {
      prefs = DEFAULT_PREFS;
      await store.set(`${_user.id}`, prefs);
      await store.save();
    }

    userPrefs.set(prefs);
  } catch (error) {
    sendAlert({
      message: "alert.user-prefs.get-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};

export const updateUserPrefs = async <P extends keyof UserPrefsStore, K extends keyof UserPrefsStore[P]>(
  prefType: P,
  key: K,
  value: UserPrefsStore[P][K]
) => {
  const _user = get(user);
  if (!store || !_user) return;

  const current = get(userPrefs);
  if (!current) return;

  const updated = { ...current, [prefType]: { ...current[prefType], [key]: value } };

  try {
    await store.set(`${_user.id}`, updated);
    await store.save();
    userPrefs.set(updated);
  } catch (error) {
    sendAlert({
      message: "alert.user-prefs.set-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};