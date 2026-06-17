<script lang="ts">
  import { transactions, expenseCategories, incomeCategories } from "$lib/transactions";
  import { t } from "$lib/i18n";

  let {
    setVisibility,
  }: {
    setVisibility: (state: boolean) => void;
  } = $props();

  const transactionInstanceMap = new Map<string, number>(new Map());
  const combinedCategories = [...expenseCategories, ...incomeCategories];

  $transactions.forEach((transaction) => {
    let instances = transactionInstanceMap.get(transaction.category) || 0;
    transactionInstanceMap.set(transaction.category, instances + 1);
  });

  const allExpenses = $derived.by(() => {
    let sum = 0;
    $transactions.filter((t) => t._type === 'expense').forEach((t) => sum += t.amount);
    return sum.toFixed(2);
  });
  const allIncome = $derived.by(() => {
    let sum = 0;
    $transactions.filter((t) => t._type === 'income').forEach((t) => sum += t.amount);
    return sum.toFixed(2);
  });
  
</script>

<div id="transactions-table-statistics-overlay" class="vertical-flex-container">
  <div id="transactions-table-statistics-top-container" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["transactions-table.statistics.header"]}</h2>
    <button class="transparent-button-highlight" onclick={() => setVisibility(false)}>
      <img src="close-x.svg" alt="Close" class="img-small" />
    </button>
  </div>
  <div id="transactions-table-statistics-content" class="vertical-flex-container">
    <p>{`${$t["transactions-table.statistics.all-expenses"]}: ${allExpenses}`}</p>
    <p>{`${$t["transactions-table.statistics.all-income"]}: ${allIncome}`}</p>
    <p>{`${$t["transactions-table.statistics.net-income"]}: ${String(Number(allIncome) - Number(allExpenses))}`}</p>
    <h3 style="border-bottom: 1px solid #333;">{$t["transactions-table.statistics.category-instances.header"]}</h3>
    {#each transactionInstanceMap as instance, i (i)}
      <p>{`${(() => {
        const category = combinedCategories.find(cat => cat.value === instance[0]);
        return category ? ($t[category.parent] as Array<Record<string, string>>)[category.index][category.key] : 'Unknown';
      })()}: ${instance[1]}`}</p>
    {/each}
  </div>
</div>

<style>
  #transactions-table-statistics-overlay {
    justify-content: flex-start;
    min-width: 400px;
    width: 100%;
    height: 100%;
    padding: 16px 32px 32px;
    border-radius: 12px;
    background-color: #181818;
  }

  #transactions-table-statistics-top-container {
    position: relative;
    width: 100%;
    padding-bottom: 16px;
    border-bottom: 1px solid #333;
  }

  #transactions-table-statistics-top-container button {
    position: absolute;
    right: 0;
    height: 32px;
    width: 32px;
  }

  #transactions-table-statistics-content {
    justify-content: flex-start;
    align-items: flex-start;
    height: 100%;
    width: 100%;
    gap: 4px;
    padding: 16px;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
  }

  #transactions-table-statistics-content p {
    font-size: clamp(0.75rem, 0.98cqw, 14px);
    margin: 0;
  }
</style>