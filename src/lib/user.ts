import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

import { closeAll, sendAlert } from "./alert";
import { type User } from "./types";
import { resetViewStates } from "./viewStore";
import { clearTransactions } from "./transactions";
import { stopTimerBatchFlush, clearTimers } from "./timers";
import { clearNotes, clearTabs, stopNoteBatchFlush } from "./notes";

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
    const safeUser = { ...result, password: "" };
    user.set(safeUser);
    goto("/");

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const resetPassword = async (isRecovery: boolean, id: number | undefined, name: string | undefined, newPassword: string, confirmNewPassword: string, currentPassword?: string) => {
  try {
    await invoke('change_password', { id, name, ...(isRecovery ? { newPassword, confirmNewPassword } : { currentPassword, newPassword, confirmNewPassword }) });
    const currentUserData = get(user);
    user.set(currentUserData ? { ...currentUserData, requires_password_reset: false } : null);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const recoverPassword = async (name: string, recoveryKey: string) => {
  try {
    const result = await invoke<User>('recover_password', { name, recoveryKey });
    user.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const deleteUser = async (userId: number, username: string, password: string) => {
  if (!password || password.trim() === '') return { success: false };

  try {
    await invoke('delete_user', { id: userId, name: username, password: password });
    await logout(false);
    sendAlert("alert.delete-user.message.success", true, false);

    return { success: true };
  } catch (error) {
    sendAlert("alert.delete-user.message.fail", true, false);
    return { success: false };
  }
};

export const logout = async (save: boolean = true) => {
  const _user = get(user);
  if (!_user) return;
  await stopTimerBatchFlush(_user.id, _user.name, save);
  await stopNoteBatchFlush(_user.id, _user.name, save);
  user.set(null);
  closeAll();
  resetViewStates();
  clearTransactions();
  clearNotes();
  clearTabs();
  clearTimers();
};