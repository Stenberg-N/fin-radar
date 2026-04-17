<script lang="ts">
  import { transactions } from "$lib/transactions";

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
      {#each $transactions as transaction (transaction.id)}
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