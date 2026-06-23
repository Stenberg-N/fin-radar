<script lang="ts">
  import { onMount } from "svelte";

  import { getTransactionsByYear, expenseCategories, incomeCategories, isTransactionsFeedSubtext } from "$lib/transactions";
  import { type Transaction } from "$lib/types";
  import { t } from "$lib/i18n";

  const current = new Date();
  const combinedCategories = [...expenseCategories, ...incomeCategories];

  let transactionsFeedArray = $state<Transaction[]>([]);
  let thisMonthMap = $derived.by<Map<string, number>>(() => {
    let map = new Map<string, number>();

    transactionsFeedArray.filter(t => t.date.split("-")[1] === String(current.getMonth() + 1).padStart(2, '0')).forEach(t => {
      const currentSum = map.get(t.category) || 0;
      map.set(t.category, currentSum + t.amount)
    });

    return map;
  });
  let lastMonthMap = $derived.by<Map<string, number>>(() => {
    let map = new Map<string, number>();

    transactionsFeedArray.filter(t => t.date.split("-")[1] === String(current.getMonth()).padStart(2, '0')).forEach(t => {
      const currentSum = map.get(t.category) || 0;
      map.set(t.category, currentSum + t.amount)
    });

    return map;
  });
  let monthDifferencesMap = $derived.by<Map<string, string>>(() => {
    let map = new Map<string, string>();

    thisMonthMap.entries().forEach(latestTransaction => {
      lastMonthMap.entries().forEach(lastMonthTransaction => {
        if (latestTransaction[0] === lastMonthTransaction[0]) {
          const transactionDifference = ((latestTransaction[1] - lastMonthTransaction[1]) / lastMonthTransaction[1]) * 100;
          map.set(latestTransaction[0], transactionDifference.toFixed(2));
        }
      });
    });

    return map;
  });
  
  onMount(() => {
    (async () => {
      const result = await getTransactionsByYear(String(current.getFullYear()));
      if (result.success) transactionsFeedArray = result.data;
    })();
  });
</script>

<div id="transactions-feed-container" class="vertical-flex-container">
  {#if $isTransactionsFeedSubtext}
    <div id="transactions-feed-subtext-container" class="horizontal-flex-container">
      <p id="transactions-feed-subtext">{$t["transactions-feed.subtext"]}</p>
      <button class="transparent-button-highlight" onclick={() => isTransactionsFeedSubtext.set(false)}><img src="close-x.svg" alt="Close" /></button>
    </div>
  {/if}
  <h2>{$t["transactions-feed.header"]}</h2>
  {#each monthDifferencesMap as [ category, value ], i (i)}
    <p>
      <span>
        {(() => {
          const item = combinedCategories.find(cat => cat.value === category);
          return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
        })()}
      </span>:

      {`${
        (Number(value) > 0 && ["salary", "freelance", "investments"].includes(category))
        ? `${$t["transactions-feed.texts"][1]} ${value}% ${$t["transactions-feed.texts"][3]}`
        : (Number(value) < 0 && ["salary", "freelance", "investments"].includes(category))
          ? `${$t["transactions-feed.texts"][1]} ${value}% ${$t["transactions-feed.texts"][2]}`
          : Number(value) > 0
            ? `${$t["transactions-feed.texts"][0]} ${value}% ${$t["transactions-feed.texts"][3]}`
            : `${$t["transactions-feed.texts"][0]} ${value}% ${$t["transactions-feed.texts"][2]}`
      }`}
    </p>
  {/each}
</div>

<style>
  #transactions-feed-container {
    position: relative;
    justify-content: flex-start;
    height: 100%;
    padding: 82px 32px 32px;
    gap: 16px;
    border-radius: 8px;
    background-color: #181818;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    > p {
      align-self: flex-start;
      margin: 0;
      font-size: clamp(14px, 1.2cqw, 1rem);

      > span {
        font-size: 1rem;
        font-weight: bold;
        color: rgb(255, 70, 70);
      }
    }

    > p:first-of-type {
      padding-top: 16px;
    }
  }

  #transactions-feed-container h2 {
    width: 100%;
    margin: 0;
    padding-bottom: 16px;
    border-bottom: 1px solid #333;
    text-align: center;
  }

  #transactions-feed-subtext-container {
    position: absolute;
    inset: 8px;
    bottom: unset;
    padding: 8px 24px;
    border-radius: 8px;
    background-color: #222;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    user-select: none;
  }

  #transactions-feed-subtext-container p {
    margin: 0;
    font-size: 12px;
  }

  #transactions-feed-subtext-container button {
    position: absolute;
    right: 2px;
    top: 2px;
    height: 18px;
    width: 18px;
  }

  #transactions-feed-subtext-container button img {
    height: 10px;
    width: 10px;
  }
</style>