import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { type Tab, type Note, type TabIdTitle } from "./types";
import { sendAlert } from "./classes/alertStore.svelte";
import { i18n } from "./i18n/i18n.svelte";

export const notes = writable<Note[]>([]);
export const tabs = writable<Tab[]>([]);

const noteUpdateBatch: Partial<Note>[] = [];
let flushInterval: ReturnType<typeof setInterval> | null = null;
export const isNoteUpdateBatchOngoing = writable<boolean>(false);

const flushBatch = async () => {
  if (!noteUpdateBatch.length) return;
  const batch = noteUpdateBatch.splice(0);

  await updateNote(batch);
  isNoteUpdateBatchOngoing.update(() => false);
};

export const startNoteBatchFlush = () => {
  if (flushInterval) return;
  flushInterval = setInterval(() => flushBatch(), 2000);
};

export const stopNoteBatchFlush = async (save?: boolean) => {
  if (flushInterval) clearInterval(flushInterval);
  flushInterval = null;
  if (save) await flushBatch();
};

export const queueNoteUpdate = (note: {id: number, title?: string, content?: string, bg_color?: string}) => {
  const defined = Object.fromEntries(Object.entries(note).filter(([_, v]) => v !== undefined)) as Partial<Note>;
  const idx = noteUpdateBatch.findIndex(n => n.id === note.id);
  if (idx !== -1) noteUpdateBatch[idx] = { ...noteUpdateBatch[idx], ...defined };
  else noteUpdateBatch.push(defined);

  notes.update((notes) => notes.map((note) => note.id === defined.id ? { ...note, ...defined } : note));
};

//
// MAIN NOTES
//

export const createNote = async (
  tabId: number,
) => {
  if (tabId === null) return { success: false };

  try {
    const title = i18n.lang === 'en' ? "Title" : "Otsikko";
    const content = i18n.lang === 'en' ? "No content" : "Ei sisältöä";
    const result = await invoke<Note>('create_note', { tabId: tabId, title: `<p><span style="font-size: 16px;">${title}</span></p>`, content: `<p><span style="font-size: 16px;">${content}</span></p>` });

    notes.update((notes) => [ ...notes, result ]);

    return { success: true };
  } catch {
    sendAlert({ message: "alert.add-note.fail", isTimer: true, buttons: false});
    return { success: false };
  }
};

export const getNotes = async () => {
  const result = await invoke<Note[]>('get_notes');
  notes.set(result);
};

export const deleteNote = async (noteId: number) => {
  try {
    const result = await invoke<Note>('delete_note', { noteId: noteId });
    notes.update((notes) => [ ...notes.filter(n => n.id !== result.id) ]);

    sendAlert({ message: "alert.delete-note.success", isTimer: true, buttons: false });
    return { success: true };
  } catch {
    sendAlert({ message: "alert.delete-note.fail", isTimer: true, buttons: false });
    return { success: false };
  }
};

export const updateNote = async (noteArray: Partial<Note>[]) => {
  try {
    await invoke<Note[]>('update_note', { noteArray: noteArray });

    return { success: true };
  } catch {
    sendAlert({ message: "alert.note-update.fail", isTimer: true, buttons: false });
    await getNotes();
    return { success: false };
  }
};

export const clearNotes = () => notes.set([]);

//
// MAIN TABS
//

export const createTab = async () => {
  try {
    const title = i18n.lang === 'en' ? "New tab" : "Uusi välilehti";
    const result = await invoke<Tab>('create_tab', { title: title });
    tabs.update((tabs) => [ ...tabs, result ]);

    return { success: true };
  } catch {
    sendAlert({ message: "alert.add-tab.fail", isTimer: true, buttons: false });
    return { success: false };
  }
};

export const getTabs = async () => {
  const result = await invoke<Tab[]>('get_tabs');
  tabs.set(result);
};

export const deleteTab = async (tabId: number | null) => {
  if (!tabId || !get(tabs).some(t => t.id === tabId)) return { success: false };

  try {
    const result = await invoke<Tab>('delete_tab', { tabId: tabId });
    tabs.update((tabs) => [ ...tabs.filter(t => t.id !== result.id) ]);
    notes.update((notes) => [ ...notes.filter(n => n.tab_id !== result.id) ]);

    sendAlert({ message: "alert.delete-tab.success", isTimer: true, buttons: false });
    return { success: true };
  } catch {
    sendAlert({ message: "alert.delete-tab.fail", isTimer: true, buttons: false });
    return { success: false };
  }
};

export const updateTab = async (tabId: number | null, title: string) => {
  if (!tabId) return;

  if (title.trim() === '') {
    sendAlert({ message: "alert.tab.no-title", isTimer: true, buttons: false });
    return { success: false };
  }

  try {
    const result = await invoke<TabIdTitle>('update_tab', { tabId: tabId, title: title });

    tabs.update((tabs) =>
      tabs.map((tab) =>
        tab.id === result.id
          ? { ...tab, title: result.title }
          : tab
      )
    );

    return { success: true };
  } catch {
    sendAlert({ message: "alert.update-tab.fail", isTimer: true, buttons: false });
    return { success: false };
  }
};

export const updateTabColor = async (tabId: number | null, color: string) => {
  if (!tabId || !get(tabs).some(t => t.id === tabId)) return { success: false };

  try {
    const result = await invoke<Tab>('update_tab_color', { tabId: tabId, color: color });

    tabs.update((tabs) =>
      tabs.map((tab) => 
        tab.id === result.id
          ? { ...tab, color: result.color }
          : tab
      )
    );

    return { success: true };
  } catch {
    sendAlert({ message: "alert.tab-color-update.fail", isTimer: true, buttons: false });
    return { success: false };
  }
};

export const clearTabs = () => tabs.set([]);