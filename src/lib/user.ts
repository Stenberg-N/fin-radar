import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

import { closeAll, sendAlert } from "./alert";
import { type User } from "./types";
import { resetViewStates } from "./viewStore";
import { clearTransactions } from "./transactions";
import { stopTimerBatchFlush, startTimerBatchFlush, clearTimers, getTimers } from "./timers";
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

export const validatePassword = (pw: string) => {
  const hasMinLength = pw.length >= 10;
  const noSpaces = !/\s/.test(pw);
  const hasNumbers = /\d/.test(pw);
  const hasUpperCase = /\p{Lu}+/gu.test(pw);
  const hasLowerCase = /\p{Ll}+/gu.test(pw);
  const hasSpecialChar = /[=\]!@#$€£¤%^&*(){}\[,.?+<>~§'":|/\\-]/.test(pw);

  return {
    isValid: hasMinLength && noSpaces && hasNumbers && hasUpperCase && hasLowerCase && hasSpecialChar,
  };
};

export const togglePasswordVisibility = (button: EventTarget | null) => {
  if (!button) return;

  const node = button as HTMLButtonElement;
  const passwordInput = node.previousElementSibling as HTMLInputElement | null;
  const img = node.firstChild as HTMLImageElement | null;

  if (passwordInput && img) {
    const isPassword = passwordInput.type === "password";
    passwordInput.type = isPassword ? "text" : "password";
    img.src = isPassword ? "/eye-hidden.svg" : "/eye-visible.svg";
  }
};

export const createUser = async (username: string, password: string, confirmPassword: string) => {
  try {
    const result = await invoke<string>('create_user', { name: username, password: password, confirmPassword: confirmPassword });
    
    return { success: true, result: result };
  } catch (error) {
    return { success: false, result: null }
  }
};

export const login = async (username: string, password: string) => {
  try {
    const result = await invoke<User>('login_user', { name: username, password: password });
    const safeUser = { ...result, password: "" };
    user.set(safeUser);
    await getTimers();
    startTimerBatchFlush();
    goto("/");

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const resetPassword = async (isRecovery: boolean, newPassword: string, confirmNewPassword: string, currentPassword?: string) => {
  try {
    await invoke('change_password', { ...(isRecovery ? { newPassword, confirmNewPassword } : { currentPassword, newPassword, confirmNewPassword }) });
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

export const cancelRecoverPassword = async () => {
  try {
    await invoke('cancel_password_recovery');
    await logout();
    sendAlert("alert.password-recover.cancel.success", true, false);
  } catch (error) {
    sendAlert("alert.password-recover.cancel.fail", true, false);
  }
};

export const deleteUser = async (password: string) => {
  if (!password || password.trim() === '') return { success: false };

  try {
    await invoke('delete_user', { password: password });
    await logout(false);
    sendAlert("alert.delete-user.message.success", true, false);

    return { success: true };
  } catch (error) {
    sendAlert("alert.delete-user.message.fail", true, false);
    return { success: false };
  }
};

export const logout = async (save: boolean = true) => {
  await invoke('logout_user');
  await stopTimerBatchFlush(save);
  await stopNoteBatchFlush(save);
  user.set(null);
  closeAll();
  resetViewStates();
  clearTransactions();
  clearNotes();
  clearTabs();
  clearTimers();
};