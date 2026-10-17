<script lang="ts">
  import { transactionsMap, transactionCategoryTags } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";
  import { formatMonthYear } from "$lib/utils/helpers.svelte";

  let {
    options,
  }: {
    options: {
      setVisibility: (state: boolean) => void;
      now: Date;
    }
  } = $props();

  const now = $derived(formatMonthYear(options.now));

  const allExpenses = transactionsMap.get('type-sums')?.get('expense')?.toFixed(2) || 0;
  const allIncome = transactionsMap.get('type-sums')?.get('income')?.toFixed(2) || 0;
  const allTransactions = [...(transactionsMap.get('category-instances')?.values() || [])].reduce((acc, curr) => acc + curr, 0);
  const allExpenseInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(0, 12).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);
  const allIncomeInstances = [...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(-3).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0);

  const statisticsInfo = [
    {
      get label() { return (i18n.t["main.layout.view-title"] as string[])[1]; },
      data: allTransactions,
    },
    {
      get label() { return i18n.t["expenses.header"] as string; },
      data: allExpenseInstances,
    },
    {
      get label() { return i18n.t["income.header"] as string; },
      data: allIncomeInstances,
    },
    {
      get label() { return i18n.t["transactions-table.statistics.all-expenses"] as string; },
      data: -allExpenses,
    },
    {
      get label() { return i18n.t["transactions-table.statistics.all-income"] as string; },
      data: allIncome,
    },
    {
      get label() { return i18n.t["transactions-table.statistics.net-income"] as string; },
      data: String((Number(allIncome) - Number(allExpenses)).toFixed(2)),
    },
  ];
</script>

<div id="transactions-table-statistics-modal" class="flex column">
  <div id="transactions-table-statistics-top-container" class="flex row">
    <div class="flex column">
      <h2>
        {i18n.t["transactions-table.statistics.header"]}
      </h2>
      <p>{now}</p>
    </div>
    <button aria-label="Close modal" class="button-primary transparent highlight static" onclick={() => options.setVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
    </button>
  </div>
  <div id="transactions-table-statistics-content" class="flex column">
    <h3>
      {i18n.t["transactions-table.statistics.quick-look"]}
    </h3>
    {#each statisticsInfo as statistic, i (i)}
      <p class="flex">
        {statistic.label}:
        <span>{statistic.data + (![0, 1, 2].includes(i) ? $userPrefs.mainPrefs.currency : "")}</span>
      </p>
    {/each}
    {#each Array.from(transactionsMap).slice(0, 2) as [ key, map ], i (i)}
      <h3>
        {i18n.t[`transactions-table.statistics.${key}.header`]}
      </h3>
      {#each map as [ key, content ], idx (idx)}
        <p class="flex">
          {(i18n.t["add-transaction.categories"] as Record<string, string>)[key]}:
          <span>{(i === 1 && transactionCategoryTags.slice(0, 12).includes(key) ? "-" : "") + content + (i === 1 ? $userPrefs.mainPrefs.currency : "")}</span>
        </p>
      {/each}
    {/each}
  </div>
</div>

<style>
  #transactions-table-statistics-modal {
    justify-content: flex-start;
    width: 100%;
    min-height: 0;
    height: 100%;
    padding: 1rem 2rem 2rem;
    background-color: var(--color-secondary1);
  }

  #transactions-table-statistics-top-container {
    width: 100%;
    justify-content: space-between;
    padding-bottom: 0.25rem;
    border-bottom: 2px solid var(--outline-color1);

    > div {
      align-items: flex-start;

      h2 {
        margin: 0;
      }

      p {
        margin: 0 0 0 0.5rem;
        font-size: 14px;
        color: var(--color-highlight1);
      }
    }
  }

  #transactions-table-statistics-content {
    justify-content: flex-start;
    align-items: flex-start;
    height: 100%;
    width: 100%;
    padding: 1rem 0;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

    h3 {
      width: 100%;
      padding: 0.25rem 0;

      &:first-of-type {
        margin-top: 0;
      }
    }

    p {
      justify-content: space-between;
      align-self: stretch;
      gap: 5rem;
      padding: 0.25rem 0;
      margin: 0 0.5rem;
      font-size: clamp(14px, 1.1cqw, 1rem);

      span {
        font-weight: bold;
        text-align: left;
      }
    }
  }
</style>