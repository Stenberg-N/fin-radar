<script lang="ts">
  import AddTransactionForm from "../components/AddTransactionForm.svelte";
</script>

<div id="home-main-container" class="horizontal-flex-container">
  <div class="form-wrapper">
    <AddTransactionForm />
  </div>
</div>

<style>
  #home-main-container {
    width: 100%;
    height: 100%;
    align-items: unset;
    justify-content: flex-start;
    padding: 20px;
  }

  .form-wrapper {
    max-width: 40%;
  }
</style>