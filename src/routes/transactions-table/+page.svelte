<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import { user } from "$lib/user";
  import type { Transaction } from "$lib/types";

  let transactions = $state<Transaction[]>([]);

  onMount(() => {
    getTransactions();
  });

  const getTransactions = async () => {
    if (!$user) return;

    try {
      transactions = await invoke('get_transactions', { userId: $user.id, name: $user.name });
    } catch (error) {
      console.error(error);
    }
  };

</script>

<div id="transactions-table-main-container" class="vertical-flex-container">
  <table>
    <thead>
      <tr>
        <td>ID</td>
        <td>Date</td>
        <td>Amount</td>
        <td>Category</td>
        <td>Description</td>
        <td>Type</td>
      </tr>
    </thead>
    <tbody>
      {#each transactions as transaction (transaction.id)}
        <tr>
          <td>{transaction.id}</td>
          <td>{transaction.date}</td>
          <td>{transaction.amount}</td>
          <td>{transaction.category}</td>
          <td>{transaction.description}</td>
          <td>{transaction._type}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  #transactions-table-main-container {
    height: 100%;
    width: 100%;
  }
</style>