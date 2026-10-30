<script lang="ts">
  import { slide, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { writable } from "svelte/store";
  import { onMount, untrack } from "svelte";
  import { SvelteDate, SvelteSet } from "svelte/reactivity";
  import { beforeNavigate, goto, onNavigate } from "$app/navigation";

  import { sendAlert } from "$lib/classes/alertStore.svelte";
  import { transactions, deleteTransaction, updateTransaction, getTransactions } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { Transaction } from "$lib/types";
  import { formatMonthYear, toYearMonth } from "$lib/utils/helpers.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";
  import { calendarDate } from "$lib/calendar";

  import AddTransactionForm from "../../components/transactions/AddTransactionForm.svelte";
  import StatisticsModal from "../../components/transactions/StatisticsModal.svelte";
  import SearchBar from "../../components/elements/SearchBar.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import DateSelect from "../../components/elements/DateSelect.svelte";
  import TransactionRow from "../../components/transactions/TransactionRow.svelte";

  type HoverContent = { content: string };

  let selectedTransactionIds: SvelteSet<number> = new SvelteSet();
  let frozenIds = $state<Set<number> | null>(null);
  let current: SvelteDate = new SvelteDate();
  let isFormVisible = $state<boolean>(false);
  let isStatisticsVisible = $state<boolean>(false);
  const columnsAndTypes = [
    { column: "id", type: "number" },
    { column: "date", type: "datetime" },
    { column: "amount", type: "number" },
    { column: "category", type: "text" },
    { column: "description", type: "text" },
    { column: "_type", type: "text" },
  ];
  let sortData = writable<{ column: string, ascending: boolean }>({ column: '', ascending: true });
  let searchRegex = $state<RegExp | null>(null);
  let appliedRegex = $state<RegExp | null>(null);
  let clearSearch = $state<{ runClearSearch: () => void} | null>(null);
  let inEditMode = $state<boolean>(false);
  let openFormButton = $state<HTMLButtonElement | null>(null);
  let openStatisticsButton = $state<HTMLButtonElement | null>(null);
  const hover = new HoverTitle<HoverContent>();

  const NAVBAR_WIDTH = $derived($userPrefs.mainPrefs.gutterAt);
  let CONTAINER = $state<HTMLDivElement | null>(null);
  let CONTAINER_HEIGHT = $state<number | null>(null);
  let CONTAINER_WIDTH = $state<number | null>(null);
  const ITEM_HEIGHT = 56;
  const BUFFER = 5;
  const VISIBLE_ITEMS = $derived(Math.ceil((CONTAINER_HEIGHT ?? 0) / ITEM_HEIGHT));
  let scrollTop = $state<number>(0);

  const sortedFilteredTransactions = $derived.by(() => {
    if (inEditMode) {
      const sort = $sortData;
      const t = untrack(() => sortRows(editableTransactions, sort));
      return frozenIds
        ? t.filter((t) => frozenIds!.has(t.id))
        : t;
    }

    let base = $transactions;
    if (appliedRegex) {
      const regex = appliedRegex;
      base = base.filter((t) => matches(t, regex));
    }

    return sortRows(base, $sortData);
  });

  const start = $derived(Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - BUFFER));
  const end = $derived(Math.min(sortedFilteredTransactions.length, start + VISIBLE_ITEMS + BUFFER * 2));

  let originalTransactions = $state<Transaction[]>([]);
  let editableTransactions = $state<Transaction[]>([]);
  const displayTransactions = $derived(sortedFilteredTransactions.slice(start, end));

  const toolBarNavButtons = [
    { label: 'Refresh transactions', img: '/refresh.svg', command: () => goto("/transactions-table") },
    { label: 'Previous month', img: '/arrow.svg', command: () => handleMonthChange(-1) },
    { label: 'Next month', img: '/arrow.svg', command: () => handleMonthChange(1) },
  ];

  let toolBarLowerButtons = $state([
    {
      get text() { return i18n.t[!isStatisticsVisible ? "transactions-table.statistics.show" : "transactions-table.statistics.hide"]; },
      img: null,
      command: () => isStatisticsVisible = !isStatisticsVisible,
      get bind() { return openStatisticsButton; },
      set bind(el) { openStatisticsButton = el; },
      disabled: null,
      get toggled() { return isStatisticsVisible; },
    },
    { 
      get text() { return i18n.t[isFormVisible ? "cancel.button" : "add.button"]; },
      img: '/plus.svg',
      command: () => isFormVisible = !isFormVisible,
      get bind() { return openFormButton; },
      set bind(el) { openFormButton = el; },
      get disabled() { return inEditMode; },
      get toggled() { return isFormVisible; },
    },
    {
      get text() { return i18n.t[inEditMode ? "exit.button": "edit.button"]; },
      img: '/edit-pen.svg',
      command: () => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) }),
      bind: null,
      get disabled() { return sortedFilteredTransactions.length <= 0 || isFormVisible; },
      toggled: null,
    },
    {
      text: i18n.t["commit.button"],
      img: '/disk.svg',
      command: () => sendAlert({ message: "alert.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() }),
      bind: null,
      get disabled() { return !inEditMode; },
      toggled: null,
    },
  ]);

  const editBannerButtons = [
    {
      get text() { return i18n.t[inEditMode ? "exit.button": "edit.button"]; },
      img: '/edit-pen.svg',
      command: () => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) }),
      get disabled() { return isFormVisible; },
      show: true,
    },
    {
      get text() { return i18n.t["delete.button"]; },
      img: '/trash-can.svg',
      command: () => sendAlert({ message: "alert.transactions-table.delete.confirmation", isTimer: false, buttons: true, onConfirm: async () => handleDelete() }),
      get disabled() { return inEditMode; },
      show: true,
    },
    {
      get text() { return i18n.t["commit.button"]; },
      img: '/disk.svg',
      command: () => sendAlert({ message: "alert.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() }),
      disabled: null,
      get show() { return inEditMode; },
    },
  ];

  onMount(() => {
    handleVirtualList();
  });

  onNavigate(({  }) => {
    const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
    statusBar.textContent = null;
  });

  beforeNavigate(({ cancel, to }) => {
    if (!inEditMode) return;

    cancel();
    const pendingNavigation = to?.url.pathname;
    sendAlert({
      message: "alert.transactions-table.in-edit-mode.warning",
      isTimer: false,
      buttons: true,
      additionalText: [(i18n.t["alert.navigate-without-saving"] as string) + " " + (i18n.t["alert.transactions-table.toggle-edit.confirmation"] as string)[1]],
      onConfirm: () => pendingNavigation ? (inEditMode = false, goto(pendingNavigation)) : {}
    });
  });

  $effect(() => {
    if (selectedTransactionIds.size > 0 && !inEditMode) {
      CONTAINER?.style.setProperty('--table-body-outer', "280px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "280px");
    } else if (inEditMode) {
      CONTAINER?.style.setProperty('--table-body-outer', "232px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "232px");
    } else {
      CONTAINER?.style.setProperty('--table-body-outer', "36px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "36px");
    }
  });

  $effect(() => {
    const yearMonth = toYearMonth(current);
    const timer = setTimeout(async () => {
      await refreshTransactions(yearMonth);
    }, 400);

    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (current !== null) {
      const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
      statusBar.textContent = formatMonthYear(current);
    }
  });

  $effect(() => {
    const regex = searchRegex;

    if (!regex) {
      appliedRegex = null;
      frozenIds = null;
      return;
    }

    untrack(() => {
      const source = inEditMode ? editableTransactions : $transactions;

      if (source.some((t) => matches(t, regex))) {
        appliedRegex = regex;
        frozenIds = new SvelteSet(source.filter((t) => matches(t, appliedRegex!)).map((t) => t.id));
      } else {
        appliedRegex = null;
        frozenIds = null;
        sendAlert({
          message: "alert.search.nothing-found",
          isTimer: true,
          buttons: false,
        });
      }
    });
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const emptySortData = () => { sortData.set({ column: '', ascending: true }); };
  const sortRows = (rows: Transaction[], sort: { column: string, ascending: boolean }) => {
    if (!sort.column) return rows;
    const columnInfo = columnsAndTypes.find(c => c.column === sort.column);
    if (!columnInfo) return rows;

    return [...rows].sort((a, b) => {
      const aValue = a[sort.column as keyof Transaction];
      const bValue = b[sort.column as keyof Transaction];

      let order = 0;
      switch (columnInfo.type) {
        case "datetime": order = new Date(aValue).getTime() - new Date(bValue).getTime(); break;
        case "number": order = Number(aValue) - Number(bValue); break;
        case "text": order = String(aValue).localeCompare(String(bValue)); break;
      }

      return sort.ascending ? order : -order;
    });
  };
  const handleVirtualList = () => { if (!CONTAINER) return; scrollTop = CONTAINER.scrollTop; };
  const refreshTransactions = async (yearMonth?: string) => {
    if (!yearMonth) yearMonth = toYearMonth(current);
    await getTransactions(yearMonth);
    selectedTransactionIds.clear();
    emptySortData();
  };
  const matches = (t: Transaction, regex: RegExp) => {
    return Object.values(t).some(val => regex.test(String(val)));
  };

  /***********************************************************************************************************************************/

  const patchTransaction = (id: number, patch: Partial<Transaction>) => {
    const transaction = editableTransactions.find(t => t.id === id);
    if (transaction) Object.assign(transaction, patch);
  };

  const handleSelect = (id: number) => {
    if (selectedTransactionIds.has(id)) selectedTransactionIds.delete(id);
    else selectedTransactionIds.add(id);
  };

  const handleSelectAll = () => {
    if (selectedTransactionIds.size === sortedFilteredTransactions.length) selectedTransactionIds.clear();
    else sortedFilteredTransactions.map(t => t.id).forEach(tid => selectedTransactionIds.add(tid));
  };

  const handleDelete = async () => {
    const yearMonth = toYearMonth(current);
    const result = await deleteTransaction(selectedTransactionIds, yearMonth);

    if (result.success) selectedTransactionIds.clear();
  };

  const enterEditMode = () => {
    originalTransactions = structuredClone($transactions);
    editableTransactions = structuredClone($transactions);
    frozenIds = appliedRegex ? new SvelteSet(editableTransactions.filter((t) => matches(t, appliedRegex!)).map((t) => t.id)) : null;
    inEditMode = true;
    emptySortData();
  };

  const exitEditMode = (clearIds?: boolean) => {
    inEditMode = false;
    emptySortData();
    frozenIds = null;
    if (clearIds !== false || clearIds === undefined) selectedTransactionIds.clear();
  };

  const commitChanges = async () => {
    let needsRefresh = false;
    const originalMap = new Map(originalTransactions.map(t => [t.id, t]));
    const changedTransactions: Transaction[] = [];

    for (const edited of editableTransactions) {
      const original = originalMap.get(edited.id);
      if (!original) continue;

      if (
        edited.date !== original.date ||
        String(edited.amount) !== String(original.amount) ||
        edited.category !== original.category ||
        edited.description !== original.description
      ) {
        switch (true) {
          case edited.date.trim() === '' || !/^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$/.test(edited.date): {
            sendAlert({ message: "alert.invalid-date", isTimer: true, buttons: false });
            return;
          }
          case edited.category.trim() === '': {
            sendAlert({ message: "alert.add-transaction.invalid-category", isTimer: true, buttons: false });
            return;
          }
          case edited.description.trim() === '': {
            sendAlert({ message: "alert.add-transaction.invalid-description", isTimer: true, buttons: false });
            return;
          }
          case edited.amount === null || edited.amount <= 0 || isNaN(edited.amount): {
            sendAlert({ message: "alert.add-transaction.invalid-amount", isTimer: true, buttons: false });
            return;
          }
        }
        if (edited.date.split("-")[1] !== original.date.split("-")[1]) needsRefresh = true;
        changedTransactions.push(edited);
      }
    }

    if (changedTransactions.length === 0) {
      sendAlert({ message: "alert.saving.no-changes", isTimer: true, buttons: false });
      exitEditMode();
      return;
    }

    const yearMonth = toYearMonth(current);
    const result = await updateTransaction(changedTransactions, yearMonth);

    if (result.success && needsRefresh) refreshTransactions();
    exitEditMode();
  };

  const handleMonthChange = (delta: number) => {
    current.setFullYear(current.getFullYear(), current.getMonth() + delta, 1);
    clearSearch?.runClearSearch();
  };

  const orderBy = (column: string) => {
    sortData.update(current => current.column === column
      ? { column, ascending: !current.ascending }
      : { column, ascending: true });
  };
</script>

{#if isFormVisible}
  <ModalWrapper
    position={{ left: NAVBAR_WIDTH + 20, top: 150, isDraggable: true }}
    transition={{ type: "slide", duration: 300, easing: "cubic-in-out", axis: "y" }}
    onOutsideClick={() => { isFormVisible = false; }}
    ignorableEls={[openFormButton]}
    outline={{ width: 1, color: "var(--outline-color2)"}}
    focus={true}
  >
    <AddTransactionForm
      closeForm={(state) => { isFormVisible = state; }}
      calendarStartDate={current}
    />
  </ModalWrapper>
{/if}

{#if isStatisticsVisible}
  <ModalWrapper
    position={{ left: NAVBAR_WIDTH + 20, top: 150, isDraggable: true }}
    transition={{ type: "slide", duration: 300, easing: "cubic-in-out", axis: "y" }}
    onOutsideClick={() => { isStatisticsVisible = false; }}
    ignorableEls={[openStatisticsButton]}
    outline={{ width: 1, color: "var(--outline-color2)"}}
    focus={true}
  >
    <StatisticsModal
      setVisibility={(state) => { isStatisticsVisible = state; }}
      now={current} 
    />
  </ModalWrapper>
{/if}

{#if hover.isHovering}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    position={{ moveTop: -30 }}
    transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
    borderRadius={8}
    outline={{ width: 1, color: 'var(--outline-color2)'}}
  >
    <p id="transactions-table-hover-title-content">
      {hover.target?.content}
    </p>
  </ModalWrapper>
{/if}

<div id="transactions-table-main-container" class="flex column">
  <div id="main-toolbar" class="flex column">
    <div class="transactions-table-toolbar-subbar primary-toolbar flex row">
      <div id="nav-controls" class="flex row">
        {#each toolBarNavButtons as button, i (i)}
          <button aria-label="{button.label}" class="button-primary transparent highlight" onclick={button.command} disabled={[1, 2].includes(i) && inEditMode}>
            <span class="span-icon img-small" style="mask-image: url('{button.img}'); {[1, 2].includes(i) && `transform: rotateZ(${i === 1 ? 90 : -90}deg);`}"></span>
          </button>
        {/each}
      </div>
      <DateSelect
        onDateSelect={(date) => {
          if (date.getTime() !== current.getTime()) current.setTime(date.getTime());
          calendarDate.set(current);
        }}
        disabled={inEditMode}
      />
      <SearchBar
        sendRegexToParent={(regex) => { searchRegex = regex; }}
        getClearSearch={(func) => { clearSearch = func; }}
        addFunctionsToClearSearch={[emptySortData]}
        searchModeIndicator={true}
      />
    </div>
    <div class="transactions-table-toolbar-subbar primary-toolbar flex row">
      {#each toolBarLowerButtons as button, i (i)}
        <button
          bind:this={button.bind}
          class="button-primary {i === 3 ? 'white-bg' : 'transparent highlight outline default-corners'}"
          class:toggled={button.toggled}
          style="{i === 0 && 'min-width: 105px'};"
          onclick={button.command}
          disabled={button.disabled}
        >
          {#if i !== 0}
            <span class="span-icon img-small" style="mask-image: url('{button.img}'); {i === 1 && isFormVisible ? 'transform: rotateZ(45deg)' : ''}; transition: transform 0.1s;"></span>
          {/if}
          {button.text}
        </button>
      {/each}
    </div>
  </div>

  <div id="transactions-table">
    {#if selectedTransactionIds.size > 0 || inEditMode}
      <div id="edit-banner" class="flex column" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
        <div id="edit-banner-top-bar" class="flex row">
          <p style="margin: 0;">
            {i18n.t["transactions-table.edit-banner.header"]}
          </p>
          {#if inEditMode}
            <p class="opacity-breathing" style="position: absolute; right: 50%; transform: translateX(50%);">
              {i18n.t["transactions-table.edit-banner.notification.header.editmode"]}
            </p>
          {/if}
          <button aria-label="Close banner" class="button-primary transparent highlight static"
            onclick={() => inEditMode
              ? sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) })
              : exitEditMode()
            }
          >
            <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
          </button>
        </div>

        {#if !inEditMode}
          <p transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
            {(i18n.t["transactions-table.edit-banner.paragraph"] as string[])[0]} {selectedTransactionIds.size} {(i18n.t["transactions-table.edit-banner.paragraph"] as string[])[1]}
          </p>
        {/if}

        <div id="edit-banner-buttons" class="flex row">
          {#each editBannerButtons as button, i (i)}
            {#if button.show}
              <button class="button-primary {i === 2 ? 'white-bg' : ''}" disabled={button.disabled} onclick={button.command} transition:fly={i === 2 ? { y: 24, duration: 200, easing: cubicInOut } : undefined}>
                <span class="span-icon" style="mask-image: url('{button.img}');"></span>
                {button.text}
              </button>
            {/if}
          {/each}
        </div>

        <div class="flex row" style="gap: 2px;">
          {#each (i18n.t["transactions-table.edit-banner.note"] as string[]) as text, i (text)}
            <p style="font-weight: {i === 0 ? "bold" : ""}; opacity: 0.5; font-size: 0.75rem;">{text}</p>
          {/each}
        </div>
      </div>
    {/if}

    <div id="headers-container" class="gridbox" class:selected-txs={selectedTransactionIds.size > 0 || inEditMode}>
      <input type="checkbox" class="table-checkbox" style="align-self: center;" checked={sortedFilteredTransactions.length > 0 && selectedTransactionIds.size === sortedFilteredTransactions.length && !inEditMode}
        disabled={sortedFilteredTransactions.length <= 0 || inEditMode} onclick={() => inEditMode ? {} : handleSelectAll()}
      />
      {#each (i18n.t["transactions-table.thead.headers"] as string[]) as header, i (header)}
        <button class="table-header button-primary transparent"
          class:currentlyOrderedBy={$sortData.column === columnsAndTypes[i]["column"]}
          style="max-width: {i === 2 ? '380px' : ''};"
          onclick={() => orderBy(columnsAndTypes[i]["column"])}
        >
          {header}
          {#if $sortData.column === columnsAndTypes[i]["column"]}
            <span class="span-icon img-small" style="
              mask-image: url('/arrow.svg');
              {$sortData.ascending ? 'transform: rotateZ(180deg);' : ""};
              transition: {$sortData.column === columnsAndTypes[i]["column"] ? 'transform 0.1s' : ""};"
            ></span>
          {/if}
        </button>
      {/each}
    </div>

    <div id="table-body-wrapper" bind:this={CONTAINER} bind:clientHeight={CONTAINER_HEIGHT} bind:clientWidth={CONTAINER_WIDTH} onscroll={handleVirtualList}>
      <div style="height: {sortedFilteredTransactions.length * ITEM_HEIGHT + 20}px; position: relative;">
        <div id="table-body" class="flex column" style="transform: translateY({start * ITEM_HEIGHT}px);">
          {#if sortedFilteredTransactions.length > 0}
            {#each displayTransactions as transaction (transaction.id)}
              <TransactionRow
                {transaction}
                {selectedTransactionIds}
                handleSelect={(id) => { handleSelect(id); }}
                onChange={patchTransaction}
                {inEditMode}
                {CONTAINER_WIDTH}
                onMouseLeave={(e) => hover.leave(e)}
                onMouseEnter={(e, str) => hover.enter(e, { content: str })}
              />
            {/each}
          {:else}
            <div class="flex column" style="margin-top: 120px;">
              <h3>{i18n.t["transactions-table.no-transactions"]}</h3>
              <span class="span-icon" style="mask-image: url('/credit-card.svg'); width: 240px; height: 180px; mask-position: center;"></span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #transactions-table-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary2);
  }

  #transactions-table-main-container, #transactions-table {
    position: relative;
    height: 100%;
    width: 100%;
    justify-content: flex-start;
    overflow: hidden;
  }

  #transactions-table-main-container {
    .gridbox {
      display: grid;
      grid-template-columns: 46px minmax(40px, 70px) minmax(90px, 120px) minmax(128px, 1fr) minmax(136px, 1fr) minmax(96px, 1fr) 48px;
    }

    #table-body-wrapper {
      position: absolute;
      top: 0;
      bottom: var(--table-body-outer-bottom);
      width: 100%;
      overflow-y: auto;
      transform: translateY(var(--table-body-outer, 36px));
      transition: transform 300ms ease-in-out, bottom 300ms ease-in-out;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));
    }

    #table-body {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      align-items: unset;
      width: 100%;
      overflow: hidden;
      padding: 10px 0.5rem;
    }

    #main-toolbar {
      width: 100%;
      height: 7rem;
    }

    #nav-controls {
      gap: 0.25rem;
    }

    #edit-banner {
      position: relative;
      width: calc(100% - 20px);
      align-items: flex-start;
      margin: 10px;
      padding: 1rem;
      border-radius: 0.5rem;
      background-color: var(--color-secondary1);
      user-select: none;

      > *:not(:nth-last-child(-n + 2)) {
        margin-bottom: 1.5rem;
      }

      > p {
        margin-top: 0;
      }

      #edit-banner-top-bar {
        width: 100%;
        justify-content: space-between;

        p {
          font-weight: bold;
        }
      }

      #edit-banner-buttons {
        justify-content: flex-start;
        gap: 1rem;

        button {
          height: unset;
          justify-content: flex-start;
          padding: 0.5rem 1rem;

          span {
            width: 20px;
            height: 20px;
          }
        }
      }
    }

    .table-checkbox {
      align-self: center;
      width: 14px;
      height: 14px;
      margin: 0.25rem 1rem;
    }

    #headers-container {
      position: sticky;
      top: 0;
      height: 2.25rem;
      padding: 0.25rem 1rem 0.25rem 0.5rem;
      border-bottom: 1px solid var(--outline-color2);
      background-color: var(--color-primary2);

      &.selected-txs {
        border-top: 1px solid var(--outline-color2);
      }

      button:hover {
        color: var(--color-highlight1);
      }

      .table-header {
        justify-content: space-between;
        padding: 0 0.5rem;
        gap: 6px;
        border-radius: 0;
        text-align: left;
        font-weight: bold;
        font-size: clamp(0.75rem, 1.1cqw, 14px);
        user-select: none;
        border-left: 1px solid var(--color-secondary3);
        color: var(--color-white-primary1);

        &.currentlyOrderedBy {
          color: var(--color-highlight1);
        }
      }
    }
  }
</style>