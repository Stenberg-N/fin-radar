import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { type Tab, type Note, type TabIdTitle } from "./types";

export const notes = writable<Note[]>([]);
export const tabs = writable<Tab[]>([]);

export const createNote = async (
  userId: number,
  username: string,
  tabId: number,
  title: string,
  content: string
) => {
  try {
    const result = await invoke<Note>('create_note', { userId: userId, username: username, tabId: tabId, title: title, content: content });
    notes.update((notes) => [ ...notes, result ]);
    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const getNotes = async (userId: number, username: string, tabId: number) => {
  const result = await invoke<Note[]>('get_notes', { userId: userId, username: username, tabId: tabId });
  notes.set(result);
};

export const deleteNote = () => {

};

export const updateNote = () => {

};

export const createTab = async (userId: number, username: string, title: string) => {
  try {
    const result = await invoke<Tab>('create_tab', { userId: userId, username: username, title: title });
    tabs.update((tabs) => [ ...tabs, result ]);
    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const getTabs = async (userId: number, username: string,) => {
  const result = await invoke<Tab[]>('get_tabs', { userId: userId, username: username });
  tabs.set(result);
};

export const deleteTab = async (userId: number, username: string, tabId: number) => {
  try {
    const result = await invoke<Tab>('delete_tab', { userId: userId, username: username, tabId: tabId });
    tabs.update((tabs) => [ ...tabs.filter(t => t.id !== result.id) ]);
    notes.update((notes) => [ ...notes.filter(n => n.tab_id !== result.id) ]);
    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const updateTab = async (
  userId: number,
  username: string,
  tabId: number,
  title: string
) => {
  try {
    const result = await invoke<TabIdTitle>('update_tab', { userId: userId, username: username, tabId: tabId, title: title });

    tabs.update((tabs) =>
      tabs.map((tab) =>
        tab.id === result.id
          ? { ...tab, title: result.title }
          : tab
      )
    );

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const updateTabColor = async (
  userId: number,
  username: string,
  tabId: number,
  color: string
) => {
  try {
    const result = await invoke<Tab>('update_tab_color', { userId: userId, username: username, tabId: tabId, color: color });

    tabs.update((tabs) =>
      tabs.map((tab) => 
        tab.id === result.id
          ? { ...tab, color: result.color }
          : tab
      )
    );

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};