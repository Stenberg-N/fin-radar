import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { SvelteSet } from "svelte/reactivity";

import type { Transaction } from "./types";
import { waitForUser } from "./user";

type TransactionMapKey = "category-instances" | "category-sums" | "type-sums";

const expenseCategoryTags = ["rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare", "insurance", "subscription", "education", "other"];
const expenseCategoryKeys = Array.from({ length: 12 }, (_, i) => `add-transaction.expense.option${i+1}`);
export const expenseCategories = expenseCategoryTags.map((item, i) => ({
  parent: "add-transaction.categories.expenses",
  key: expenseCategoryKeys[i],
  value: item,
  index: i
}));

const incomeCategoryTags = ["salary", "freelance", "investments"];
const incomeCategoryKeys = Array.from({ length: 3 }, (_, i) => `add-transaction.income.option${i+1}`);
export const incomeCategories = incomeCategoryTags.map((item, i) => ({
  parent: "add-transaction.categories.income",
  key: incomeCategoryKeys[i],
  value: item,
  index: i
}));

export const isTransactionsFeedSubtext = writable(true);
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

export const getTransactions = async (yearMonth: string): Promise<{ success: boolean }> => {
  try {
    const result = await invoke<Transaction[]>('get_transactions', { yearMonth: yearMonth });
    transactions.set(result);

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const getTransactionsByYear = async (year: string): Promise<{ success: boolean, data: Transaction[] }> => {
  if (year.length !== 4) return { success: false, data: [] };

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
): Promise<{ success: boolean }> => {
  try {
    const newTransaction = await invoke<Transaction>('add_transaction', {
      category: category,
      date: date,
      description: description,
      amount: amount,
      type: categoryType,
    });
    transactions.update((transactions) => [ newTransaction, ...transactions ]);

    await ensureTransactionsFeedInitialized();

    const nowYearMonth = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)(new Date());
    const lastYearMonth = ((d: Date) => d.getMonth() === 0 ? `${String(d.getFullYear() - 1)}-12` : `${String(d.getFullYear())}-${String(d.getMonth()).padStart(2, '0')}`)(new Date());
    const newTransactionDateYearMonth = (() => {
      const dateParts = newTransaction.date.split("-");
      return `${dateParts[0]}-${dateParts[1]}`;
    })();

    if (nowYearMonth === newTransactionDateYearMonth) {
      const currentSum = thisMonthMap.get(newTransaction.category) || 0;
      thisMonthMap.set(newTransaction.category, currentSum + newTransaction.amount);
      recomputeMonthDifferencesMap();
    } else if (lastYearMonth === newTransactionDateYearMonth) {
      const currentSum = lastMonthMap.get(newTransaction.category) || 0;
      lastMonthMap.set(newTransaction.category, currentSum + newTransaction.amount);
      recomputeMonthDifferencesMap();
    }

    return { success: true };
  } catch (error) {
    return { success: false };
  }
};

export const deleteTransaction = async (ids: SvelteSet<number>, yearMonth: string): Promise<{ success: boolean, deleted: number }> => {
  try {
    const result = await invoke<Transaction[]>('delete_transaction', { ids: Array.from(ids), yearMonth: yearMonth });
    const deletedIds = result.map(t => t.id);
    transactions.update((transactions) => [ ...transactions.filter(t => !deletedIds.includes(t.id)) ]);
    
    return { success: true, deleted: deletedIds.length };
  } catch (error) {
    return { success: false, deleted: 0 };
  }
};

export const updateTransaction = async (transactionArray: Transaction[], yearMonth: string): Promise<{ success: boolean, amount: number }> => {
  try {
    const result = await invoke<Transaction[]>('update_transaction', { transactions: transactionArray, yearMonth: yearMonth });
    const ids = result.map(t => t.id);
    transactions.update((transactions) => [ ...result, ...transactions.filter(t => !ids.includes(t.id)) ]);

    return { success: true, amount: result.length };
  } catch (error) {
    return { success: false, amount: 0 };
  }
};

export const clearTransactions = (): void => transactions.set([]);

// TRANSACTIONS FEED

const computeThisMonthMap = (transactionsFeedArray: Transaction[], now: Date): Map<string, number> => {
  let map = new Map<string, number>();

  transactionsFeedArray.filter(t => t.date.split("-")[1] === String(now.getMonth() + 1).padStart(2, '0')).forEach(t => {
    const currentSum = map.get(t.category) || 0;
    map.set(t.category, currentSum + t.amount);
  });

  return map;
};

const computeLastMonthMap = (transactionsFeedArray: Transaction[], now: Date): Map<string, number> => {
  let map = new Map<string, number>();

  transactionsFeedArray.filter(t => t.date.split("-")[1] === String(now.getMonth()).padStart(2, '0')).forEach(t => {
    const currentSum = map.get(t.category) || 0;
    map.set(t.category, currentSum + t.amount);
  });

  return map;
};

const computeMonthDifferencesMap = (): Map<string, number> => {
  let map = new Map<string, number>();

  thisMonthMap.entries().forEach(latestTransaction => {
    lastMonthMap.entries().forEach(lastMonthTransaction => {
      if (latestTransaction[0] === lastMonthTransaction[0]) {
        const transactionDifference = ((latestTransaction[1] - lastMonthTransaction[1]) / lastMonthTransaction[1]) * 100;
        map.set(latestTransaction[0], Number(transactionDifference.toFixed(2)));
      }
    });
    if (!map.has(latestTransaction[0])) map.set(latestTransaction[0].concat("-new"), latestTransaction[1]);
  });

  return map;
};

const recomputeMonthDifferencesMap = (): void => {
  monthDifferencesMap.set(computeMonthDifferencesMap());
};

let thisMonthMap: Map<string, number>;
let lastMonthMap: Map<string, number>;
export const monthDifferencesMap = writable<Map<string, number>>(new Map());

let readyResolve: () => void;
const readyPromise = new Promise<void>((resolve) => {
  readyResolve = resolve;
});

export const initTransactionsFeed = async (): Promise<void> => {
  const _user = await waitForUser();
  if (_user.requires_password_reset) return;

  const now = new Date();
  const result = await getTransactionsByYear(String(now.getFullYear()));
  const transactionsFeedArray = result.success ? result.data : [];

  thisMonthMap = computeThisMonthMap(transactionsFeedArray, now);
  lastMonthMap = computeLastMonthMap(transactionsFeedArray, now);
  recomputeMonthDifferencesMap();

  readyResolve();
};

export const ensureTransactionsFeedInitialized = (): Promise<void> => {
  return readyPromise;
};