import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { Transaction } from "./types";
import { getTransactionCategories } from "./i18n";

const expenseCategoryTags = ["rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare", "insurance", "subscription", "education", "other"];
export const expenseCategories = (getTransactionCategories("add-transaction.categories.expenses") as Array<Record<string, string>>).map((item, i) => ({
  parent: "add-transaction.categories.expenses",
  key: Object.keys(item)[0],
  value: expenseCategoryTags[i],
  index: i
}));

const incomeCategoryTags = ["salary", "freelance", "investments"];
export const incomeCategories = (getTransactionCategories("add-transaction.categories.income") as Array<Record<string, string>>).map((item, i) => ({
  parent: "add-transaction.categories.income",
  key: Object.keys(item)[0],
  value: incomeCategoryTags[i],
  index: i
}));

const savedTransactions = localStorage.getItem('transactions');
const initialTransactions: Transaction[] = savedTransactions ? JSON.parse(savedTransactions) : [];

export const transactions = writable<Transaction[]>(initialTransactions);

transactions.subscribe((value) => {
  if (value) localStorage.setItem("transactions", JSON.stringify(value));
  else localStorage.removeItem("transactions");
});

export const getTransactions = async (userId: number, yearMonth: string, username: string) => {
  try {
    const result = await invoke<Transaction[]>('get_transactions', { userId: userId, yearMonth: yearMonth, name: username });
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

export const deleteTransaction = async (userId: number, ids: Array<number>, username: string) => {
  try {
    const result = await invoke<Transaction[]>('delete_transaction', { userId: userId, ids: ids, name: username });
    const deletedIds = result.map(t => t.id);
    transactions.update((transactions) => [ ...transactions.filter(t => !deletedIds.includes(t.id))]);
    
    return { success: true, deleted: deletedIds.length };
  } catch (error) {
    return { success: false };
  }
};

export const updateTransaction = async (userId: number, transactionArray: Transaction[], username: string) => {
  try {
    const result = await invoke<Transaction[]>('update_transaction', { userId: userId, transactions: transactionArray, name: username });
    const ids = result.map(t => t.id);
    transactions.update((transactions) => [ ...transactions.filter(t => !ids.includes(t.id)), ...result ]);

    return { success: true, amount: result.length };
  } catch (error) {
    return { success: false };
  }
};

export const clearTransactions = () => transactions.set([]);