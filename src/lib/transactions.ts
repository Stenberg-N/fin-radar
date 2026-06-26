import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { SvelteSet } from "svelte/reactivity";

import type { Transaction } from "./types";
import { getTransactionCategories } from "./i18n";

type TransactionMapKey = "category-instances" | "category-sums" | "type-sums"

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

export let isTransactionsFeedSubtext = writable(true);

export const transactions = writable<Transaction[]>([]);

export const transactionsMap = new Map<TransactionMapKey, Map<string, number>>();
transactions.subscribe((currentTransactions) => {
  const transactionInstances = new Map<string, number>();
  const transactionCategorySums = new Map<string, number>();
  const transactionTypeSums = new Map<string, number>();

  currentTransactions.forEach((transaction) => {
    let instances = transactionInstances.get(transaction.category) || 0;
    transactionInstances.set(transaction.category, instances + 1);

    let currentSum = transactionCategorySums.get(transaction.category) || 0;
    transactionCategorySums.set(transaction.category, currentSum + transaction.amount);

    let currentTypeSum = transactionTypeSums.get(transaction._type) || 0;
    transactionTypeSums.set(transaction._type, currentTypeSum + transaction.amount);
  });

  transactionsMap.set("category-instances", transactionInstances);
  transactionsMap.set("category-sums", transactionCategorySums);
  transactionsMap.set("type-sums", transactionTypeSums);
});

export const getTransactions = async (yearMonth: string) => {
  try {
    const result = await invoke<Transaction[]>('get_transactions', { yearMonth: yearMonth });
    transactions.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const getTransactionsByYear = async (year: string) => {
  try {
    const result = await invoke<Transaction[]>('get_year_transactions', { year: year });
    return { success: true, data: result };
  } catch (error) {
    return { success: false, data: [] };
  }
};

export const addTransaction = async (
  category: string,
  date: string,
  description: string,
  amount: number,
  categoryType: string,
) => {
  try {
    const newTransaction = await invoke<Transaction>('add_transaction', {
      category: category,
      date: date,
      description: description,
      amount: amount,
      type: categoryType,
    });
    transactions.update((transactions) => [ newTransaction, ...transactions ]);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const deleteTransaction = async (ids: SvelteSet<number>, yearMonth: string) => {
  try {
    const result = await invoke<Transaction[]>('delete_transaction', { ids: Array.from(ids), yearMonth: yearMonth });
    const deletedIds = result.map(t => t.id);
    transactions.update((transactions) => [ ...transactions.filter(t => !deletedIds.includes(t.id)) ]);
    
    return { success: true, deleted: deletedIds.length };
  } catch (error) {
    return { success: false, deleted: 0 };
  }
};

export const updateTransaction = async (transactionArray: Transaction[], yearMonth: string) => {
  try {
    const result = await invoke<Transaction[]>('update_transaction', { transactions: transactionArray, yearMonth: yearMonth });
    const ids = result.map(t => t.id);
    transactions.update((transactions) => [ ...result, ...transactions.filter(t => !ids.includes(t.id)) ]);

    return { success: true, amount: result.length };
  } catch (error) {
    return { success: false };
  }
};

export const clearTransactions = () => transactions.set([]);