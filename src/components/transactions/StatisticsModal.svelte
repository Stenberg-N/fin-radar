<script lang="ts">
  import { transactionCategoryTags, transactions } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";
  import { formatMonthYear } from "$lib/utils/helpers.svelte";

  type TransactionMapKey = "category-instances" | "category-sums" | "type-sums";

  interface Props {
    /**
     * Returns a boolean false when the close button is clicked.
     * 
     * Assign this value to a $state to conditionally render the component.
     */
    setVisibility?: (state: boolean) => void;
    /** The date to show on the component. */
    now: Date;
    /** Used to set the background to transparent. Defaults to `false`. */
    transparentBg?: boolean;
    /** Used to set lower padding. Defaults to `false`. */
    lowerPadding?: boolean;
    /** Used to hide the component header. Defaults to `true`. */
    showHeader?: boolean;
    /** Used to add a gap between the key, e.g. a category name and its value, e.g. how many instances of it there are. */
    addGaps?: boolean;
    /** Used to pass a reference to an element. This element's clientHeight is subtracted from the components 100% height. */
    topBar?: HTMLElement | null;
  }

  let {
    setVisibility,
    now: rawNow,
    transparentBg = false,
    lowerPadding = false,
    showHeader = true,
    addGaps = true,
    topBar,
  }: Props = $props();

  const now = $derived(formatMonthYear(rawNow));
  const topBarHeight = $derived(topBar?.clientHeight ?? 0);

  const transactionsMap = $derived.by(() => {
    const map = new Map<TransactionMapKey, Map<string, number>>();
    const transactionInstances = new Map<string, number>();
    const transactionCategorySums = new Map<string, number>();
    const transactionTypeSums = new Map<string, number>();

    $transactions.forEach((transaction) => {
      let instances = transactionInstances.get(transaction.category) || 0;
      transactionInstances.set(transaction.category, instances + 1);

      let currentSum = transactionCategorySums.get(transaction.category) || 0;
      transactionCategorySums.set(transaction.category, currentSum + transaction.amount);

      let currentTypeSum = transactionTypeSums.get(transaction._type) || 0;
      transactionTypeSums.set(transaction._type, currentTypeSum + transaction.amount);
    });

    map.set("category-instances", transactionInstances);
    map.set("category-sums", transactionCategorySums);
    map.set("type-sums", transactionTypeSums);

    return map;
  });

  const categorySections = $derived(
    (["category-instances", "category-sums"] as const).map((key) => ({
      key,
      entries: [...(transactionsMap.get(key) ?? [])]
    }))
  );

  const allExpenses = $derived(transactionsMap.get('type-sums')?.get('expense')?.toFixed(2) || 0);
  const allIncome = $derived(transactionsMap.get('type-sums')?.get('income')?.toFixed(2) || 0);
  const allTransactions = $derived([...(transactionsMap.get('category-instances')?.values() || [])].reduce((acc, curr) => acc + curr, 0));
  const allExpenseInstances = $derived([...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(0, 12).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0));
  const allIncomeInstances = $derived([...(transactionsMap.get('category-instances')?.entries().filter(([key, _]) =>
    transactionCategoryTags.slice(-3).includes(key)) || [])].reduce((acc, [_, value]) => acc + value, 0));

  const statisticsInfo = $derived([
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
  ]);
</script>

<div id="transactions-table-statistics-modal" class="flex column"
  style:--bg={transparentBg ? "transparent" : "var(--color-secondary1)"}
  style:--padding={lowerPadding ? "0 0.5rem" : "1rem 2rem 2rem"}
  style:--gap={addGaps ? "5rem" : ""}
  style:--topbar={`${topBarHeight}px`}
>
  <div id="transactions-table-statistics-top-container" class="flex row">
    <div class="flex column">
      {#if showHeader}
        <h2>
          {i18n.t["transactions-table.statistics.header"]}
        </h2>
      {/if}
      <p>{now}</p>
    </div>
    {#if setVisibility}
      <button aria-label="Close modal" class="button-primary transparent highlight static" onclick={() => setVisibility(false)}>
        <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
      </button>
    {/if}
  </div>
  <div id="transactions-table-statistics-content" class="flex column">
    <h3>
      {i18n.t["transactions-table.statistics.quick-look"]}
    </h3>
    {#each statisticsInfo as statistic, i (statistic.label)}
      <p class="flex">
        {statistic.label}:
        <span>{statistic.data + (![0, 1, 2].includes(i) ? $userPrefs.mainPrefs.currency : "")}</span>
      </p>
    {/each}
    {#each categorySections as section, i (section.key)}
      <h3>
        {i18n.t[`transactions-table.statistics.${section.key}.header`]}
      </h3>
      {#each section.entries as [category, content] (category)}
        <p class="flex">
          {(i18n.t["add-transaction.categories"] as Record<string, string>)[category]}:
          <span>{(i === 1 && transactionCategoryTags.slice(0, 12).includes(category) ? "-" : "") + content + (i === 1 ? $userPrefs.mainPrefs.currency : "")}</span>
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
    height: calc(100% - var(--topbar));
    padding: var(--padding);
    background-color: var(--bg);
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
      padding: 0.25rem 0;
      margin: 0 0.5rem;
      gap: var(--gap);
      font-size: clamp(14px, 1.1cqw, 1rem);

      span {
        font-weight: bold;
        text-align: left;
      }
    }
  }
</style>