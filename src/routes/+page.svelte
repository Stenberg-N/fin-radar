<script lang="ts">
  import AddTransactionForm from "../components/AddTransactionForm.svelte";
  import TransactionsFeed from "../components/home/TransactionsFeed.svelte";
</script>

<div id="home-main-container" class="horizontal-flex-container">
  <div class="form-wrapper">
    <AddTransactionForm />
  </div>
  <TransactionsFeed />
</div>

<style>
  #home-main-container {
    width: 100%;
    height: 100%;
    align-items: unset;
    justify-content: flex-start;
    padding: 20px;
    gap: 20px;
  }
</style>