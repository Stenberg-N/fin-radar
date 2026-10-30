<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { monthDifferencesMap, lastMonthMap, thisMonthMap, transactionCategoryTags } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  type HoverTarget = { element: "header" | "new-tag" | "filter" };

  interface Props {
    /** Used to hide the component header. Defaults to `true`. */
    showHeader?: boolean;
    /** Used to pass a reference to an element. This element's clientHeight is subtracted from the components 100% height. */
    topBar?: HTMLElement | null;
  }

  let {
    showHeader = true,
    topBar,
  }: Props = $props();

  const topBarHeight = $derived(topBar?.clientHeight ?? 0);

  const hover = new HoverTitle<HoverTarget>();
  const displayedFeed = $derived([...$monthDifferencesMap].filter(([category, _]) => matches(category)));
  let selectedFilters: SvelteSet<"new" | "expense" | "income"> = new SvelteSet();
  const expenseTags = transactionCategoryTags.slice(0, -3);
  const incomeTags = transactionCategoryTags.slice(-3);
  let isFiltersVisible = $state<boolean>(false);
  let toggleFiltersButton = $state<HTMLButtonElement | null>(null);

  const filters = $derived([
    {
      get title() { return (i18n.t["transactions-feed.texts"] as string[])[0].slice(0, -1); },
      id: "new" as const,
    },
    {
      get title() { return i18n.t["income.header"] as string; },
      id: "income" as const,
    },
    {
      get title() { return i18n.t["expenses.header"] as string; },
      id: "expense" as const,
    }
  ]);

  $effect(() => {
    return () => { hover.destroy(); };
  });

  const matches = (category: string) => {
    const tag = category.split("-")[0];
    const wantsIncome = selectedFilters.has("income");
    const wantsExpense = selectedFilters.has("expense");
    const wantsNew = selectedFilters.has("new");

    if (wantsIncome || wantsExpense) {
      const match =
        (wantsIncome && incomeTags.includes(tag)) ||
        (wantsExpense && expenseTags.includes(tag));
      if (!match) return false;
    }

    if (wantsNew && !category.endsWith("-new")) return false;

    return true;
  };

  const toggleFilter = (filter: "new" | "expense" | "income") => {
    if (selectedFilters.has(filter)) {
      selectedFilters.delete(filter);
      return;
    }

    if (filter === "income") selectedFilters.delete("expense");
    if (filter === "expense") selectedFilters.delete("income");
    selectedFilters.add(filter);
  };
</script>

{#if isFiltersVisible}
  <ModalWrapper
    transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
    onOutsideClick={() => { isFiltersVisible = false; }}
    ignorableEls={[toggleFiltersButton]}
    borderRadius={8}
    outline={{ width: 1, color: 'var(--outline-color2)'}}
  >
    <div id="home-filter-options-container" class="flex column">
      {#each filters as filter (filter.id)}
        <label class="flex row">
          <input type="checkbox" checked={selectedFilters.has(filter.id)} onchange={() => { toggleFilter(filter.id); }} />
          <span>{filter.title}</span>
        </label>
      {/each}
    </div>
  </ModalWrapper>
{/if}

{#if hover.isHovering}
  {@const content = (() => {
    switch (hover.target?.element) {
      case "filter": return i18n.t["filter-list-header"] as string;
      case "header": return (i18n.t["transactions-feed.texts"] as string[])[1];
      case "new-tag": return (i18n.t["transactions-feed.texts"] as string[])[2];
    }
  })()}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    transition={{ type: "fade", easing: "cubic-in-out", duration: 200 }}
    position={{ moveTop: 24, centerElement: true }}
    outline={{ width: 1, color: 'var(--outline-color3)'}}
    borderRadius={8}
  >
    <p id="home-hover-modal-content">
      {content}
    </p>
  </ModalWrapper>
{/if}

<div id="transactions-feed-container" class="flex column"
  style:--topbar={`${topBarHeight}px`}
  style:--border={showHeader ? "2px solid var(--outline-color2)" : "none"}
  style:--padding={showHeader ? "0 0.5rem" : "0"}
  style:--padding-toolbar={showHeader ? "0.5rem" : "0.25rem 0.5rem"}
>
  <div id="transactions-feed-toolbar" class="flex row">
    {#if showHeader}
      <h2 onmouseenter={(e) => hover.enter(e, { element: "header" })} onmouseleave={(e) => hover.leave(e)}>
        {i18n.t["transactions-feed.header"]}
      </h2>
    {/if}
    <button
      aria-label="Toggle filters"
      bind:this={toggleFiltersButton}
      class="button-primary transparent highlight default-corners lower-padding"
      class:toggled={isFiltersVisible}
      onclick={() => { isFiltersVisible = !isFiltersVisible; }}
      onmouseenter={(e) => hover.enter(e, { element: "filter" })}
      onmouseleave={(e) => hover.leave(e)}
    >
      <span class="span-icon img-small" style="mask-image: url('/filter.svg');"></span>
    </button>
  </div>
  <div id="transactions-feed-content">
    {#if displayedFeed.length > 0}
      {#each displayedFeed as [ category, value ] (category)}
        {@const isPositive = value > 0 && transactionCategoryTags.slice(-3).includes(category.split("-")[0]) || value < 0 && !transactionCategoryTags.slice(-3).includes(category.split("-")[0])}
        <div class="feed-item flex column" in:fly={{ x: -120, duration: 300, easing: cubicInOut }} out:fly={{ x: 120, duration: 300, easing: cubicInOut }}>
          <div class="feed-item-topbar flex row">
            <p>
              {(() => {
                const item = transactionCategoryTags.find(k => k === category.split("-")[0]);
                return item
                  ? (i18n.t["add-transaction.categories"] as Record<string, string>)[item]
                  : 'Unknown';
              })()}
            </p>
            {#if category.endsWith("-new")}
              <p onmouseenter={(e) => hover.enter(e, { element: "new-tag" })} onmouseleave={(e) => hover.leave(e)}>
                {(i18n.t["transactions-feed.texts"] as string[])[0]}
              </p>
            {/if}
          </div>
          <div class="feed-item-content flex column">
            <div class="flex">
              <span class="span-icon img-small-medium" style="mask-image: url('/sack.svg');"></span>
              {thisMonthMap.get(category.split('-')[0]) + $userPrefs.mainPrefs.currency}
            </div>
            <div class="flex">
              <span
                class="span-icon img-medium-large"
                style="
                  mask-image: url('/arrow-up.svg');
                  transform: rotate({value > 0 ? '' : '180deg'});
                  background-color: var(--color-{isPositive ? 'positive' : 'negative'});
                "
              ></span>
              {(value > 0 ? "+" : "") + value + "%"}
              {
                `${(() => {
                  const res = (thisMonthMap.get(category.split('-')[0]) ?? 0) - (lastMonthMap.get(category) ?? 0);
                  return `(${(res < 0 ? "-" : "+") + Math.abs(res) + $userPrefs.mainPrefs.currency})`;
                })()}`
              }
            </div>
          </div>
        </div>
      {/each}
    {:else}
      <p style="align-self: center; margin-top: 40%; user-select: none; font-weight: bold;">
        {i18n.t["transactions-feed.nothing-to-report"]}
      </p>
    {/if}
  </div>
</div>

<style>
  #home-hover-modal-content {
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary2);
    margin: 0;
    text-align: center;
  }
  
  #home-filter-options-container {
    align-items: flex-start;
    gap: 0.25rem;
    padding: 0.5rem;
    background-color: var(--color-secondary2);
    
    label {
      justify-content: flex-start;
      width: 100%;
      gap: 0.5rem;
      padding: 0.25rem 0.5rem;
      border-radius: 0.25rem;
      background-color: var(--color-secondary4);

      &:hover {
        cursor: pointer;
      }

      input {
        margin: 0;
        height: 14px;
        width: 14px;
      }

      span {
        user-select: none;
      }
    }
  }

  #transactions-feed-container {
    justify-content: flex-start;
    align-items: flex-start;
    flex: 1 1 auto;
    height: calc(100% - var(--topbar));
    padding: var(--padding);

    h2 {
      margin: 0;
    }

    #transactions-feed-toolbar {
      justify-content: space-between;
      width: 100%;
      padding: var(--padding-toolbar);
      border-bottom: var(--border);

      button {
        margin-left: auto;
      }
    }

    #transactions-feed-content {
      display: grid;
      grid-template-columns: 1fr;
      grid-auto-rows: max-content;
      width: 100%;
      height: 100%;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
      text-align: center;

      .feed-item {
        align-self: flex-start;
        align-items: flex-start;
        width: 100%;
        min-width: fit-content;
        padding: 0.75rem;
        gap: 1rem;
        border-bottom: 1px solid var(--outline-color2);
        font-size: clamp(14px, 1.2cqw, 1rem);

        .feed-item-content {
          align-items: flex-start;
          gap: 0.25rem;

          > div {
            gap: 0.25rem;

            &:first-of-type {
              span {
                margin: 0.25rem;
              }
            }
          }
        }

        .feed-item-topbar {
          justify-content: space-between;
          width: 100%;

          > p {
            align-self: flex-start;
            margin: 0;

            &:first-of-type {
              font-weight: bold;
              color: var(--color-highlight1);
            }

            &:nth-of-type(2) {
              padding: 0.25rem;
              border-radius: 0.25rem;
              outline: 1px solid var(--outline-color2);
              color: var(--color-highlight3);
              font-size: 14px;
              line-height: 16px;
              user-select: none;
            }
          }
        }
      }
    }
  }
</style>