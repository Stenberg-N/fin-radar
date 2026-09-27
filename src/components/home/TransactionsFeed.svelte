<script lang="ts">
  import { monthDifferencesMap, transactionCategoryTags } from "$lib/transactions";
  import { t } from "$lib/i18n/i18n";

</script>

<div id="transactions-feed-container" class="flex column">
  <h2 title={$t["transactions-feed.subtext"] as string}>{$t["transactions-feed.header"]}</h2>
  <div id="transactions-feed-content" class="flex column">
    {#if $monthDifferencesMap.size > 0}
      {#each $monthDifferencesMap as [ category, value ], i (i)}
        <p>
          <span>
            {(() => {
              const item = transactionCategoryTags.find(k => k === category.split("-")[0]);
              return item
                ? ($t["add-transaction.categories"] as Record<string, string>)[item]
                : 'Unknown';
            })()}
          </span>:

          {`${
            category.endsWith("-new")
            ? ($t["transactions-feed.texts"] as string[])[2]
            : value > 0
              ? `${Math.abs(value)}% ${($t["transactions-feed.texts"] as string[])[1]}`
              : `${Math.abs(value)}% ${($t["transactions-feed.texts"] as string[])[0]}`
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
    flex: 1 1 auto;
    height: 100%;
    max-width: 450px;
    padding: 1rem 2rem 2rem;
    border-radius: 8px;
    background-color: var(--color-secondary1);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    h2 {
      width: 100%;
      margin: 0;
      padding-bottom: 1rem;
      border-bottom: 2px solid var(--outline-color1);
      text-align: center;
    }

    #transactions-feed-content {
      width: 100%;
      gap: 1rem;
      padding: 10px;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
    
      > p {
        align-self: flex-start;
        margin: 0;
        font-size: clamp(14px, 1.2cqw, 1rem);

        &:first-of-type {
          margin-top: 1rem;
        }

        > span {
          font-size: 1rem;
          font-weight: bold;
          color: var(--color-highlight1);
        }
      }
    }
  }
</style>