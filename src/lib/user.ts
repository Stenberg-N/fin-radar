import { writable, get } from "svelte/store";
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

export const resetPassword = async(isRecovery: boolean, id: number | undefined, name: string | undefined, newPassword: string, confirmNewPassword: string, currentPassword?: string) => {
  try {
    const result = await invoke<boolean>('change_password', { id, name, ...(isRecovery ? { newPassword, confirmNewPassword } : { currentPassword, newPassword, confirmNewPassword }) });
    const currentUserData = get(user);
    user.set(result && currentUserData ? { ...currentUserData, requires_password_reset: result } : null);

    return { success: true};
  } catch (error) {
    return { success: false};
  }
};

export const recoverPassword = async(name: string, recoveryKey: string) => {
  try {
    const result = await invoke<User>('recover_password', { name, recoveryKey });
    user.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
}

export const logout = () => {
  user.set(null);
  closeAll();
};