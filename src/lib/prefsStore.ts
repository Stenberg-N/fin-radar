import { Store, load } from "@tauri-apps/plugin-store";
import { get, writable } from "svelte/store";

import { user } from "./user";
import { sendAlert } from "./alert";
import { lang, type Language } from "./i18n/i18n";

type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

type NotePrefs = {
  noteColumns: number;
  noteHeight: "50%" | "100%";
  noteBgColor: "dark" | "light";
  mainBgColor: "dark" | "light";
};

type MainPrefs = {
  navBarWidth: number;
  lang: Language;
};

type UserPrefsStore = {
  mainPrefs: MainPrefs;
  notePrefs: NotePrefs;
  settingsOverlayPrefs: SettingsOverlayPrefs;
};

type SettingsOverlayPrefs = {
  sideBarWidth: number;
};

const DEFAULT_PREFS: UserPrefsStore = {
  mainPrefs: {
    navBarWidth: 150,
    lang: "en",
  },
  notePrefs: {
    noteColumns: 4,
    noteHeight: "100%",
    noteBgColor: "dark",
    mainBgColor: "dark",
  },
  settingsOverlayPrefs: {
    sideBarWidth: 300,
  },
};

let store: Store;
export const userPrefs = writable<UserPrefsStore>(DEFAULT_PREFS);
let loadPrefsPromise: Promise<void> | null = null;
let loadedForUserId: number | null = null;
let saveTimeout: ReturnType<typeof setTimeout>;

const isObject = (val: unknown): val is Record<string, unknown> => typeof val === "object" && val !== null && !Array.isArray(val);

const mergeDefaults = <T extends Record<string, unknown>>(
  defaults: T,
  stored: DeepPartial<T> | undefined
): T => {
  const result = {} as T;

  for (const key of Object.keys(defaults) as (keyof T)[]) {
    const defaultValue = defaults[key];
    const storedValue = stored?.[key];

    if (isObject(defaultValue) && isObject(storedValue)) {
      result[key] = mergeDefaults(
        defaultValue as Record<string, unknown>,
        storedValue as DeepPartial<Record<string, unknown>>
      ) as T[keyof T];
    } else {
      result[key] = (storedValue !== undefined ? storedValue : defaultValue) as T[keyof T];
    }
  }

  return result;
};

const normalizeUserPrefs = (stored: DeepPartial<UserPrefsStore> | undefined): UserPrefsStore => mergeDefaults(DEFAULT_PREFS, stored);

const loadUserPrefs = async (overrides?: Partial<MainPrefs>) => {
  const _user = get(user);
  if (!_user) return;

  try {
    store = await load('user-preferences.json', { autoSave: false });
    const doesUserExist = await store.has(`${_user.id}`);
    const prefs = await store.get<DeepPartial<UserPrefsStore>>(`${_user.id}`);
    let normalized = normalizeUserPrefs(prefs);
    lang.set(normalized.mainPrefs.lang);

    if (!doesUserExist) {
      normalized.mainPrefs = { ...normalized.mainPrefs, ...overrides };
      overrides?.lang && lang.set(overrides.lang);
    }

    if (JSON.stringify(normalized) !== JSON.stringify(prefs)) {
      await store.set(`${_user.id}`, normalized);
      await store.save();
    }

    userPrefs.set(normalized);
    loadedForUserId = _user.id;
  } catch (error) {
    sendAlert({
      message: "alert.user-prefs.get-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};

export const ensureUserPrefsLoaded = (overrides?: Partial<MainPrefs>) => {
  const _user = get(user);
  if (!_user) return Promise.resolve();
  if (loadPrefsPromise && loadedForUserId === _user.id) return loadPrefsPromise;
  
  loadPrefsPromise = loadUserPrefs(overrides);
  return loadPrefsPromise;
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
  userPrefs.set(updated);

  try {
    await store.set(`${_user.id}`, updated);
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await store.save();
    }, 300);
  } catch (error) {
    sendAlert({
      message: "alert.user-prefs.set-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};

export const clearUserPrefs = async () => {
  const _user = get(user);
  if (!store || !_user) return;

  try {
    await store.delete(`${_user.id}`);
    await store.save();
  } catch (error) {
    sendAlert({
      message: "alert.user-prefs.delete-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};