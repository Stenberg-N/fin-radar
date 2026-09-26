<script lang="ts">
  import { transactionsMap, transactionCategoryTags } from "$lib/transactions";
  import { t } from "$lib/i18n/i18n";
  import { handleClickOutside } from "$lib/actions";

  let {
    setVisibility,
    ignorableEls,
  }: {
    setVisibility: (state: boolean) => void;
    ignorableEls: (HTMLElement | null)[];
  } = $props();

  const allExpenses = transactionsMap.get('type-sums')?.get('expense')?.toFixed(2) || 0;
  const allIncome = transactionsMap.get('type-sums')?.get('income')?.toFixed(2) || 0;
  const allTransactions = [...(transactionsMap.get('category-instances')?.values() || [])].reduce((acc, curr) => acc + curr, 0);
  const allExpenseInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(0, 12).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);
  const allIncomeInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(12, 15).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);

  const statisticsInfo = [
    {
      get label() { return $t["main.layout.view-title"] as string[]; },
      data: allTransactions,
    },
    {
      get label() { return $t["expenses.header"] as string; },
      data: allExpenseInstances,
    },
    {
      get label() { return $t["income.header"] as string; },
      data: allIncomeInstances,
    },
    {
      get label() { return $t["transactions-table.statistics.all-expenses"] as string; },
      data: -allExpenses,
    },
    {
      get label() { return $t["transactions-table.statistics.all-income"] as string; },
      data: allIncome,
    },
    {
      get label() { return $t["transactions-table.statistics.net-income"] as string; },
      data: String((Number(allIncome) - Number(allExpenses)).toFixed(2)),
    },
  ];
</script>

<div id="transactions-table-statistics-overlay" class="flex column"
  use:handleClickOutside={{ onOutsideClick: () => setVisibility(false), additionalElements: ignorableEls }}
>
  <div id="transactions-table-statistics-top-container" class="flex row">
    <h2 style="margin: 0;">{$t["transactions-table.statistics.header"]}</h2>
    <button aria-label="Close modal" class="button-primary transparent highlight static" onclick={() => setVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
    </button>
  </div>
  <div id="transactions-table-statistics-content" class="flex column">
    {#each statisticsInfo as statistic, i (i)}
      <p>{i === 0 ? statistic.label[1] : statistic.label}: <span>{statistic.data}</span></p>
    {/each}
    {#each Array.from(transactionsMap).slice(0, 2) as [ key, map ], i (i)}
      <h3 style="border-bottom: 2px solid var(--outline-color1);">{$t[`transactions-table.statistics.${key}.header`]}</h3>
      {#each map as [ key, content ], idx (idx)}
        <p>
          {($t["add-transaction.categories"] as Record<string, string>)[key]}:
          <span>{(i === 1 && transactionCategoryTags.slice(0, 12).includes(key)) ? -content : content}</span>
        </p>
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
    padding: 1rem 2rem 2rem;
    border-radius: 0.5rem;
    background-color: var(--color-secondary1);
  }

  #transactions-table-statistics-top-container {
    width: 100%;
    justify-content: space-between;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--outline-color1);
  }

  #transactions-table-statistics-content {
    justify-content: flex-start;
    align-items: flex-start;
    height: 100%;
    width: 100%;
    gap: 0.25rem;
    padding: 1rem;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

    p {
      font-size: clamp(14px, 1.1cqw, 1rem);
      margin: 0;

      span {
        font-weight: bold;
      }
    }
  }
</style>