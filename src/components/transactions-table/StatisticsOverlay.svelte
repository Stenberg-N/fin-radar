<script lang="ts">
  import { expenseCategories, incomeCategories, transactionsMap } from "$lib/transactions";
  import { t } from "$lib/i18n";

  let {
    setVisibility,
  }: {
    setVisibility: (state: boolean) => void;
  } = $props();

  const combinedCategories = [...expenseCategories, ...incomeCategories];
  const allExpenses = transactionsMap.get('type-sums')?.get('expense')?.toFixed(2) || 0;
  const allIncome = transactionsMap.get('type-sums')?.get('income')?.toFixed(2) || 0;
  const allTransactions = [...(transactionsMap.get('category-instances')?.values() || [])].reduce((acc, curr) => acc + curr, 0);
  const allExpenseInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    [...expenseCategories.map(item => item.value)].includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);
  const allIncomeInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    [...incomeCategories.map(item => item.value)].includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);
</script>

<div id="transactions-table-statistics-overlay" class="vertical-flex-container">
  <div id="transactions-table-statistics-top-container" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["transactions-table.statistics.header"]}</h2>
    <button class="transparent-button-highlight" onclick={() => setVisibility(false)}>
      <img src="close-x.svg" alt="Close" class="img-small" />
    </button>
  </div>
  <div id="transactions-table-statistics-content" class="vertical-flex-container">
    <p>{$t["main.layout.view-title"][1]}: <span>{allTransactions}</span></p>
    <p>{$t["expenses.header"]}: <span>{allExpenseInstances}</span></p>
    <p>{$t["income.header"]}: <span>{allIncomeInstances}</span></p>
    <p>{$t["transactions-table.statistics.all-expenses"]}: <span>-{allExpenses}</span></p>
    <p>{$t["transactions-table.statistics.all-income"]}: <span>{allIncome}</span></p>
    <p>{$t["transactions-table.statistics.net-income"]}: <span>{String(Number(allIncome) - Number(allExpenses))}</span></p>
    {#each Array.from(transactionsMap).slice(0, 2) as [ key, map ], i (i)}
      <h3 style="border-bottom: 1px solid #333;">{$t[`transactions-table.statistics.${key}.header`]}</h3>
      {#each map as [ key, content ], idx (idx)}
        {@const category = combinedCategories.find(cat => cat.value === key)}
        <p>{(() => {
          return category ? ($t[category.parent] as Array<Record<string, string>>)[category.index][category.key] : 'Unknown';
        })()}: <span>{(i === 1 && category?.parent.split(".")[2] === "expenses") ? -content : content}</span></p>
      {/each}
    {/each}
  </div>
</div>

<style>
  #transactions-table-statistics-overlay {
    justify-content: flex-start;
    width: 100%;
    min-height: 0;
    height: 100%;
    padding: 16px 32px 32px;
    border-radius: 8px;
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
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
  }

  #transactions-table-statistics-content p {
    font-size: clamp(14px, 1.1cqw, 16px);
    margin: 0;
  }

  #transactions-table-statistics-content p span {
    font-weight: bold;
  }
</style>