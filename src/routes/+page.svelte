<script lang="ts">
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import AddTransactionForm from "../components/AddTransactionForm.svelte";
  import TransactionsFeed from "../components/home/TransactionsFeed.svelte";

  const homeTools = [
    { name: "add-transaction", state: false, icon: "/credit-card.svg" },
  ];

</script>

<div id="home-main-container" class="horizontal-flex-container">
  <div id="home-tools-container" class="vertical-flex-container">
    <div id="home-tools" class="horizontal-flex-container">
      {#each homeTools as tool (tool.name)}
        <button class="transparent-button" onclick={() => tool.state = !tool.state}>
          <img src={tool.icon} alt={tool.icon.slice(1, tool.icon.length - 4)} />
        </button>
      {/each}
    </div>
    {#if homeTools[0].state}
      <div class="form-wrapper" transition:slide={{ duration: 300, easing: cubicInOut }}>
        <AddTransactionForm closeForm={() => homeTools[0].state = false} />
      </div>
    {/if}
  </div>
  <TransactionsFeed />
</div>

<style>
  .form-wrapper {
    z-index: 1;
    min-height: 0;
    height: 100%;
    width: 100%;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  #home-main-container {
    width: 100%;
    height: 100%;
    align-items: unset;
    justify-content: flex-start;
    padding: 20px;
    gap: 20px;
  }

  #home-tools-container {
    justify-content: flex-start;
    height: 100%;
    width: 500px;
    gap: 20px;

    > div:not(.form-wrapper) {
      justify-content: flex-start;
      flex-shrink: 0;
      width: 100%;
      height: 5rem;
      padding: 16px;
      border-radius: 8px;
      background-color: #181818;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    }

    button {
      height: 3rem;
      width: 3rem;
      transition: transform 0.2s;

      > img {
        height: 100%;
      }

      &:hover {
        transform: scale(1.08);
      }
    }
  }
</style>