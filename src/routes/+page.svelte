<script lang="ts">
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { userPrefs, updateUserPrefs } from "$lib/utils/prefsStore";
  import { moveGutter } from "$lib/utils/actions.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { Gutter } from "$lib/classes/gutter.svelte";
  import { getTransactions } from "$lib/transactions";
  import { toYearMonth } from "$lib/utils/helpers.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";

  import AddTransactionForm from "../components/transactions/AddTransactionForm.svelte";
  import TransactionsFeed from "../components/transactions/TransactionsFeed.svelte";
  import ModalWrapper from "../components/ModalWrapper.svelte";
  import StatisticsModal from "../components/transactions/StatisticsModal.svelte";

  type ListItem = "feed" | "stats";

  let hasStatsLoaded = false;
  const listWidth = $derived($userPrefs.homePrefs.feedListWidth);
  const hover = new HoverTitle();
  const gutter = new Gutter();

  let homeTools = $state([
    { label: "add-transaction", state: false, icon: "/credit-card.svg" },
  ]);
  const listContent = {
    feed: TransactionsFeed,
    stats: StatisticsModal,
  };
  let selectedListItem = $state<ListItem>("feed");

  let transactionFormToggleButton = $state<HTMLButtonElement | null>(null);
  let sidePanelTopBar = $state<HTMLDivElement | null>(null);
  let homeToolsRefs = $state<HTMLElement[]>([]);

  $effect(() => {
    if (homeToolsRefs[0]) transactionFormToggleButton = homeToolsRefs[0] as HTMLButtonElement;
  });

  $effect(() => {
    if (selectedListItem === "stats" && !hasStatsLoaded) {
      (async () => {
        await getTransactions(toYearMonth(new Date()));
        hasStatsLoaded = true;
      })();
    }
  });

</script>

<div id="home-main-container" style="grid-template-columns: {listWidth}px 0 minmax(480px, 1fr);">
  {#if hover.isHovering}
    <ModalWrapper
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      position={{ centerElement: true, isContinuousUpdate: true, moveTop: -50 }}
      outline={{ width: 1, color: 'var(--outline-color1)'}}
      borderRadius={8}
    >
      <p id="home-hover-title-content">
        {listWidth}px
      </p>
    </ModalWrapper>
  {/if}

    <div id="home-side-panel">
      {#if listWidth >= 250}
        {@const ListItem = listContent[selectedListItem]}
        <div bind:this={sidePanelTopBar} id="view-buttons-wrapper" class="flex"
          style="position: {selectedListItem === "feed" ? 'absolute' : 'static'}; top: 0;"
        >
          {#each Array.from({ length: 2 }, (_, i) => i) as btn (btn)}
            <button
              class="button-primary transparent text-highlight"
              class:current={btn === 0 && selectedListItem === "feed" || btn === 1 && selectedListItem === "stats"}
              onclick={() => selectedListItem = btn === 0 ? "feed" : "stats"}
            >
              {i18n.t[btn === 0 ? "transactions-feed.header" : "transactions-table.statistics.header"]}
            </button>
            {#if btn === 0}
              <div class="divider"></div>
            {/if}
          {/each}
        </div>

        <ListItem
          now={new Date()}
          lowerPadding={true}
          transparentBg={true}
          showHeader={false}
          addGaps={false}
          topBar={selectedListItem === "stats" ? sidePanelTopBar : undefined}
        />
      {/if}
    </div>

  <div role="slider" aria-valuenow={listWidth} tabindex="0" class="resize-gutter-default flex row" class:highlight={gutter.isHovered}
    use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("homePrefs", "feedListWidth", newWidth); },  min: 0, max: 600, threshold: { at: 250 , jumpTo: 0 } }}
    onmouseenter={() => { hover.enter(); gutter.enter(); }}
    onmouseleave={(e) => { hover.leave(e); gutter.leave(); }}
  ></div>

  <div id="home-main-content">
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
          <AddTransactionForm
            closeForm={(state) => { homeTools[0].state = state; }}
            ignorableEls={[transactionFormToggleButton]}
          />
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  #home-main-container {
    display: grid;
    grid-template-columns: minmax(240px, 360px) 0 minmax(480px, 1fr);
    width: 100%;
    height: 100%;
    gap: 0.25rem;
    overflow-y: auto;

    .resize-gutter-default {
      height: unset;
    }

    #home-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary1);
    }

    #home-side-panel {
      min-width: 0;
      border-right: 1px solid var(--outline-color1);
      overflow: hidden;

      #view-buttons-wrapper {
        justify-content: flex-start;
        height: 32px;
        gap: 0.5rem;
        padding: 0.25rem 0.5rem;

        .current {
          color: var(--color-highlight1);
        }

        .divider {
          align-self: stretch;
          width: 0;
          border-left: 1px solid var(--outline-color1);
          transform: translateX(50%);
        }
      }
    }

    #home-main-content {
      padding: 2rem;
      border-left: 1px solid var(--outline-color1);
      overflow: hidden;
    }

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
      width: 100%;
      max-width: 500px;
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