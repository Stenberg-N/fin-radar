<script lang="ts">
  import { expenseCategories, incomeCategories, isTransactionsFeedSubtext, monthDifferencesMap } from "$lib/transactions";
  import { t } from "$lib/i18n/i18n";

  const combinedCategories = [...expenseCategories, ...incomeCategories];

</script>

<div id="transactions-feed-container" class="flex column" class:removed-padding={!$isTransactionsFeedSubtext}>
  {#if $isTransactionsFeedSubtext}
    <div id="transactions-feed-subtext-container" class="flex row">
      <p id="transactions-feed-subtext">{$t["transactions-feed.subtext"]}</p>
      <button aria-label="Close message" class="button-primary transparent highlight" onclick={() => isTransactionsFeedSubtext.set(false)}>
        <span class="span-icon" style="mask-image: url('/close-x.svg');"></span>
      </button>
    </div>
  {/if}
  <h2>{$t["transactions-feed.header"]}</h2>
  <div id="transactions-feed-content" class="flex column">
    {#if $monthDifferencesMap.size > 0}
      {#each $monthDifferencesMap as [ category, value ], i (i)}
        <p>
          <span>
            {(() => {
              const item = combinedCategories.find(cat => cat.value === category.split("-")[0]);
              return item
                ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key]
                : 'Unknown';
            })()}
          </span>:

          {`${
            category.endsWith("-new")
            ? $t["transactions-feed.texts"][2]
            : value > 0
              ? `${Math.abs(value)}% ${$t["transactions-feed.texts"][1]}`
              : `${Math.abs(value)}% ${$t["transactions-feed.texts"][0]}`
          }`}
        </p>
      {/each}
    {:else}
      <p style="align-self: center; margin-top: 40%; user-select: none; font-weight: bold;">{$t["transactions-feed.nothing-to-report"]}</p>
    {/if}
  </div>
</div>

<style>
  #transactions-feed-container {
    position: relative;
    justify-content: flex-start;
    height: 100%;
    width: 500px;
    padding: 82px 2rem 2rem;
    border-radius: 8px;
    background-color: var(--color-secondary1);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    &.removed-padding {
      padding: 1rem 2rem 2rem;
    }

    #transactions-feed-content {
      width: 100%;
      gap: 1rem;
      padding: 10px;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
    }

    #transactions-feed-content > p {
      align-self: flex-start;
      margin: 0;
      font-size: clamp(14px, 1.2cqw, 1rem);

      > span {
        font-size: 1rem;
        font-weight: bold;
        color: var(--color-highlight1);
      }
    }

    #transactions-feed-content > p:first-of-type {
      margin-top: 1rem;
    }
  }

  #transactions-feed-container h2 {
    width: 100%;
    margin: 0;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--outline-color1);
    text-align: center;
  }

  #transactions-feed-subtext-container {
    position: absolute;
    inset: 0.5rem;
    bottom: unset;
    padding: 0.5rem 1.5rem;
    border-radius: 0.25rem;
    background-color: var(--color-secondary2);
    user-select: none;
  }

  #transactions-feed-subtext-container p {
    margin: 0;
    font-size: 0.75rem;
  }

  #transactions-feed-subtext-container button {
    position: absolute;
    right: 2px;
    top: 2px;
    height: 18px;
    width: 18px;
  }

  #transactions-feed-subtext-container button span {
    height: 10px;
    width: 10px;
  }
</style>