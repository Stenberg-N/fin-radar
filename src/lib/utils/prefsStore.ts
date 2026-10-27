import { Store, load } from "@tauri-apps/plugin-store";
import { get, writable } from "svelte/store";

import { user } from "../user";
import { sendAlert } from "../classes/alertStore.svelte";
import { i18n, type Language } from "../i18n/i18n.svelte";

type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

type MainPrefs = {
  navBarWidth: number;
  lang: Language;
  hourFormat: "12" | "24";
  currency: "£" | "$" | "€";
};

type HomePrefs = {
  feedListWidth: number;
}

type CalendarPrefs = {
  eventListWidth: number;
};

type NotePrefs = {
  noteColumns: number;
  noteHeight: "50%" | "100%";
  mainBgColor: "dark" | "light";
};

type UserPrefsStore = {
  mainPrefs: MainPrefs;
  homePrefs: HomePrefs;
  calendarPrefs: CalendarPrefs;
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
    hourFormat: "12",
    currency: "$",
  },
  homePrefs: {
    feedListWidth: 360,
  },
  calendarPrefs: {
    eventListWidth: 300,
  },
  notePrefs: {
    noteColumns: 4,
    noteHeight: "100%",
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

i18n.onLangChange = (lang) => { updateUserPrefs("mainPrefs", "lang", lang); };

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

const loadUserPrefs = async () => {
  const _user = get(user);
  if (!_user) return;

  const langBeforeLoad = i18n.lang;

  try {
    store = await load('user-preferences.json', { autoSave: false });
    const doesUserExist = await store.has(`${_user.id}`);
    const prefs = await store.get<DeepPartial<UserPrefsStore>>(`${_user.id}`);
    const normalized = normalizeUserPrefs(prefs);

    if (!doesUserExist) {
      normalized.mainPrefs = {
        ...normalized.mainPrefs,
        lang: langBeforeLoad,
        ...(langBeforeLoad === 'en'
          ? { currency: "$", hourFormat: "12" }
          : { currency: "€", hourFormat: "24" }
        )
      };
    }

    i18n.applyLang(normalized.mainPrefs.lang);

    if (JSON.stringify(normalized) !== JSON.stringify(prefs)) {
      await store.set(`${_user.id}`, normalized);
      await store.save();
    }

    userPrefs.set(normalized);
  } catch {
    loadPrefsPromise = null;
    loadedForUserId = null;
    sendAlert({
      message: "alert.user-prefs.get-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};

export const ensureUserPrefsLoaded = () => {
  const _user = get(user);
  if (!_user) return Promise.resolve();
  if (loadPrefsPromise && loadedForUserId === _user.id) return loadPrefsPromise;
  
  loadedForUserId = _user.id;
  loadPrefsPromise = loadUserPrefs();
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
  } catch {
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
  } catch {
    sendAlert({
      message: "alert.user-prefs.delete-store.fail",
      isTimer: true,
      buttons: false,
    });
  }
};

export const resetUserPrefsState = async () => {
  clearTimeout(saveTimeout);
  try {
    await store?.save();
    await store?.close();
  } catch {}
  loadPrefsPromise = null;
  loadedForUserId = null;
  userPrefs.set(DEFAULT_PREFS);
};