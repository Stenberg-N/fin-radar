import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { closeAll } from "./alert";
import { type User } from "./types";

const savedUser = localStorage.getItem('user');
const initialUser = savedUser ? JSON.parse(savedUser) : null;

export const user = writable<User | null>(initialUser);

user.subscribe((value) => {
  if (value) {
    const { password: _, ...safeUser } = value;
    localStorage.setItem('user', JSON.stringify(safeUser));
  } else {
    localStorage.removeItem('user');
  }
});

export const login = async (username: string, password: string) => {
  try {
    const result = await invoke<User>('login_user', { name: username, password: password });
    user.set(result);
    return { success: true, user: result };
  } catch (error) {
    return { success: false };
  }
};

export const logout = () => {
  user.set(null);
  closeAll();
};