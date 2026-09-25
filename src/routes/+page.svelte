<script lang="ts">
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import AddTransactionForm from "../components/AddTransactionForm.svelte";
  import TransactionsFeed from "../components/home/TransactionsFeed.svelte";

  let homeTools = $state([
    { label: "add-transaction", state: false, icon: "/credit-card.svg" },
  ]);
  
  let transactionFormToggleButton = $state<HTMLButtonElement | null>(null);
  let homeToolsRefs = $state<HTMLElement[]>([]);

  $effect(() => {
    if (homeToolsRefs[0]) transactionFormToggleButton = homeToolsRefs[0] as HTMLButtonElement;
  });

</script>

<div id="home-main-container" class="flex row">
  <TransactionsFeed />
  <div id="home-tools-container" class="flex column">
    <div id="home-tools" class="flex row">
      {#each homeTools as tool, i (tool.label)}
        <button aria-label={tool.label} bind:this={homeToolsRefs[i]} class="button-primary transparent highlight" class:toggled={tool.state} onclick={() => tool.state = !tool.state}>
          <span class="span-icon" style="mask-image: url('{tool.icon}');"></span>
        </button>
      {/each}
    </div>
    {#if homeTools[0].state}
      <div id="form-wrapper" transition:slide={{ duration: 300, easing: cubicInOut }}>
        <AddTransactionForm options={{ closeForm: () => homeTools[0].state = false, ignorableEls: [transactionFormToggleButton] }} />
      </div>
    {/if}
  </div>
</div>

<style>
  #home-main-container {
    width: 100%;
    height: 100%;
    align-items: unset;
    justify-content: flex-start;
    padding: 2rem;
    gap: 1rem;

    #form-wrapper {
      z-index: 1;
      min-height: 0;
      height: 100%;
      width: 100%;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    }

    #home-tools-container {
      justify-content: flex-start;
      height: 100%;
      width: 500px;
      gap: 1rem;

      #home-tools {
        justify-content: flex-start;
        flex-shrink: 0;
        width: 100%;
        height: 3.5rem;
        padding: 0.25rem;
        border-radius: 0.5rem;
        background-color: var(--color-secondary1);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
      }

      button {
        height: 3rem;
        width: 3rem;
        padding: 6px;
        border-radius: 0.5rem;
        transition: transform 0.2s;

        &.toggled {
          background-color: var(--hover-color-transparent);
        }

        > span {
          height: 100%;
          width: 100%;
        }
      }
    }
  }
</style>