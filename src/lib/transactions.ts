import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { Transaction } from "./types";

const savedTransactions = localStorage.getItem('transactions');
const initialTransactions: Transaction[] = savedTransactions ? JSON.parse(savedTransactions) : [];

export const transactions = writable<Transaction[]>(initialTransactions);

transactions.subscribe((value) => {
  if (value) localStorage.setItem("transactions", JSON.stringify(value));
  else localStorage.removeItem("transactions");
});

export const getTransactions = async (userId: number, username: string) => {
  try {
    const result = await invoke<Transaction[]>('get_transactions', { userId: userId, name: username });
    transactions.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const addTransaction = async (
  userId: number,
  category: string,
  date: string,
  description: string,
  amount: number,
  categoryType: string,
  username: string
) => {
  try {
    const newTransaction = await invoke<Transaction>('add_transaction', {
      userId: userId,
      category: category,
      date: date,
      description: description,
      amount: amount,
      type: categoryType,
      name: username
    });
    transactions.update((transactions) => [ ...transactions, newTransaction ]);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const clearTransactions = () => transactions.set([]);