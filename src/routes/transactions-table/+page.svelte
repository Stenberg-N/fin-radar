<script lang="ts">
  import { slide, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { writable } from "svelte/store";
  import { onMount, getContext } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { onNavigate } from "$app/navigation";

  import { sendAlert } from "$lib/alert";
  import { transactions, expenseCategories, incomeCategories, deleteTransaction, updateTransaction, getTransactions } from "$lib/transactions";
  import { t } from "$lib/i18n";
  import type { Transaction } from "$lib/types";
  import { handleClickOutside, handleKeyDownOnInput, handleNumberInput } from "$lib/actions";

  import AddTransactionForm from "../../components/AddTransactionForm.svelte";
  import StatisticsOverlay from "../../components/transactions-table/StatisticsOverlay.svelte";

  const combinedCategories = [...expenseCategories, ...incomeCategories];
  const categoryOptions = $derived(
    combinedCategories.map(option => ({
      value: option.value,
      label: ($t[option.parent] as Array<Record<string, string>>)[option.index][option.key]
    }))
  );
  let selectedTransactionIds = $state<SvelteSet<number>>(new SvelteSet());
  let current = $state(new Date());
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
  let dateToJump = $state<string>('');
  let searchable = $state<string | null>(null);
  let searchRegex = $state<RegExp | null>(null);
  const inSearchMode = $derived(searchRegex !== null ? true : false);
  let inEditMode = $state<boolean>(false);
  let openFormButton = $state<HTMLButtonElement | null>(null);
  let openStatisticsButton = $state<HTMLButtonElement | null>(null);

  let CONTAINER = $state<HTMLDivElement | null>(null);
  const ITEM_HEIGHT = 56;
  const BUFFER = 5;
  const VISIBLE_ITEMS = $derived(Math.ceil((CONTAINER ? CONTAINER.clientHeight : 0) / ITEM_HEIGHT));
  let scrollTop = $state<number>(0);

  let sortedFilteredTransactions = $derived.by(() => {
    const base = inEditMode && inSearchMode && searchRegex !== null
      ? editableTransactions.filter(t => Object.values(t).some(val => (searchRegex as RegExp).test(String(val))))
      : (inSearchMode && searchRegex !== null
        ? $transactions.filter(t => Object.values(t).some(val => (searchRegex as RegExp).test(String(val))))
        : (inEditMode ? editableTransactions : $transactions));

    return sortRows(base, $sortData);
  });

  let start = $derived(Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - BUFFER));
  let end = $derived(Math.min(sortedFilteredTransactions.length, start + VISIBLE_ITEMS + BUFFER * 2));

  let originalTransactions = $state<Transaction[]>([]);
  let editableTransactions = $state<Transaction[]>([]);
  let displayTransactions = $derived(sortedFilteredTransactions.slice(start, end));

  onMount(() => {
    handleVirtualList();
  });

  onNavigate(({  }) => {
    const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
    statusBar.textContent = null;
  });

  $effect(() => {
    const tableBodyOuter = document.getElementById("transactions-table-body-outer");
    if (selectedTransactionIds.size > 0 && !inEditMode) {
      tableBodyOuter?.style.setProperty('--table-body-outer', "302px");
      tableBodyOuter?.style.setProperty('--table-body-outer-bottom', "302px");
    } else if (inEditMode) {
      tableBodyOuter?.style.setProperty('--table-body-outer', "238px");
      tableBodyOuter?.style.setProperty('--table-body-outer-bottom', "238px");
    } else {
      tableBodyOuter?.style.setProperty('--table-body-outer', "36px");
      tableBodyOuter?.style.setProperty('--table-body-outer-bottom', "36px");
    }
  });

  $effect(() => {
    const yearMonth = `${String(current.getFullYear())}-${String(current.getMonth() + 1).padStart(2, '0')}`;
    const timer = setTimeout(async () => {
      await refreshTransactions(yearMonth);
    }, 400);

    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (current !== null) {
      const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
      statusBar.textContent = `${$t["calendar.monthnames"][current.getMonth()]}, ${current.getFullYear()}`;
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
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
    if (!yearMonth) yearMonth = `${String(current.getFullYear())}-${String(current.getMonth() + 1).padStart(2, '0')}`;
    await getTransactions(yearMonth);
    selectedTransactionIds.clear();
    emptySortData();
  };

  /***********************************************************************************************************************************/

  const handleSelect = (id: number) => {
    if (selectedTransactionIds.has(id)) selectedTransactionIds.delete(id);
    else selectedTransactionIds.add(id);
  };

  const handleSelectAll = () => {
    if (selectedTransactionIds.size === sortedFilteredTransactions.length) selectedTransactionIds.clear();
    else sortedFilteredTransactions.map(t => t.id).forEach(tid => selectedTransactionIds.add(tid));
  };

  const handleDelete = async () => {
    if (selectedTransactionIds.size <= 0) { sendAlert({ message: "alert.transactions-table.delete.no-transactions-selected", isTimer: true, buttons: false }); return; }
    const yearMonth = `${String(current.getFullYear())}-${String(current.getMonth() + 1).padStart(2, '0')}`;
    const result = await deleteTransaction(selectedTransactionIds, yearMonth);

    if (result.success) {
      sendAlert({ message: "alert.transactions-table.delete.success", isTimer: true, buttons: false, additionalText: String(result.deleted) });
      selectedTransactionIds.clear();
    } else sendAlert({ message: "alert.transactions-table.delete.fail", isTimer: true, buttons: false });
  };

  const enterEditMode = () => {
    originalTransactions = structuredClone($transactions);
    editableTransactions = structuredClone($transactions);
    inEditMode = true;
    emptySortData();
  };

  const exitEditMode = (clearIds?: boolean) => {
    inEditMode = false;
    emptySortData();
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
        if (
          edited.date.trim() === '' ||
          edited.category.trim() === '' ||
          edited.description.trim() === ''
        ) {
          sendAlert({ message: "alert.input-missing", isTimer: true, buttons: false });
          return;
        } else if (edited.amount === null || isNaN(edited.amount) || edited.amount <= 0) {
          sendAlert({ message: "alert.add-transaction.invalid-amount", isTimer: true, buttons: false });
          return;
        }
        if (edited.date.split("-")[1] !== original.date.split("-")[1]) needsRefresh = true;
        changedTransactions.push(edited);
      }
    }

    if (changedTransactions.length === 0) {
      sendAlert({ message: "alert.transactions-table.no-changes", isTimer: true, buttons: false });
      exitEditMode();
      return;
    }

    const yearMonth = `${String(current.getFullYear())}-${String(current.getMonth() + 1).padStart(2, '0')}`;
    const result = await updateTransaction(changedTransactions, yearMonth);

    if (result.success) {
      if (needsRefresh) refreshTransactions();
      sendAlert({ message: "alert.transactions-table.update.success", isTimer: true, buttons: false, additionalText: String(result.amount) });
    } else {
      sendAlert({ message: "alert.transactions-table.update.fail", isTimer: true, buttons: false });
    }
    exitEditMode();
  };

  const changeDisplayType = (target: EventTarget | null, item: Transaction) => {
    if (!target) return;
    const value = (target as HTMLSelectElement).value;
    ["salary", "freelance", "investments"].includes(value) ? item._type = "income" : item._type = "expense";
  };

  const handleNumberStepper = (command: string, target: EventTarget | null) => {
    if (!target) return;

    const input = (target as HTMLButtonElement).parentElement?.previousElementSibling as HTMLInputElement;
    let value = Number(input.value);

    switch (command) {
      case "increase": input.value = String(Math.round((value += 0.01) * 100) / 100); break;
      case "decrease": if (value > 0) input.value = String(Math.round((value -= 0.01) * 100) / 100); break;
    }
    input.dispatchEvent(new Event('input', { bubbles: true }));
  };

  const handleMonthChange = (delta: number) => {
    current = new Date(current.getFullYear(), current.getMonth() + delta, 1);
  };

  const handleDateJump = () => {
    if (dateToJump.trim() === '') return;
    const dateParts = dateToJump.split("-");
    if (!/^\d{4}$/.test(dateParts[0])) { sendAlert({ message: "alert.invalid-year", isTimer: true, buttons: false }); return; }
    if (!/^([1-9]|0[1-9]|1[0-2])$/.test(dateParts[1])) { sendAlert({ message: "alert.invalid-month", isTimer: true, buttons: false }); return; }
    const dateObject = new Date(dateParts[0] + '-' + dateParts[1].padStart(2, '0') + '-01');
    current = dateObject;
    stopSearch();
  };

  const orderBy = (column: string) => {
    sortData.update(current => current.column === column
      ? { column, ascending: !current.ascending }
      : { column, ascending: true });
  };

  const startSearch = () => {
    if (!searchable || searchable.trim() === '') return;
    searchRegex = new RegExp(searchable.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
  };

  const stopSearch = () => {
    searchable = null;
    searchRegex = null;
    emptySortData();
  };

</script>

{#if isFormVisible}
  <div class="overlay-container vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isFormVisible = false, additionalElements: [openFormButton] }}>
    <AddTransactionForm closeForm={() => isFormVisible = false} calendarStartDate={current} />
  </div>
{/if}

{#if isStatisticsVisible}
  <div class="overlay-container vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isStatisticsVisible = false, additionalElements: [openStatisticsButton] }}>
    <StatisticsOverlay setVisibility={(state) => isStatisticsVisible = state} />
  </div>
{/if}

<div id="transactions-table-main-container" class="vertical-flex-container">
  <div id="transactions-table-toolbar" class="vertical-flex-container">
    <div class="transactions-table-toolbar-subbar primary-toolbar horizontal-flex-container">
      <div id="transactions-table-toolbar-controls" class="horizontal-flex-container">
        <button class="transparent-button-highlight" onclick={async () => await refreshTransactions()}>
          <img src="/refresh.svg" alt="Refresh" class="img-small" />
        </button>
        <button class="transparent-button-highlight horizontal-flex-container" disabled={inEditMode} onclick={() => handleMonthChange(-1)}>
          <img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotateZ(90deg);" />
        </button>
        <button class="transparent-button-highlight horizontal-flex-container" disabled={inEditMode} onclick={() => handleMonthChange(1)}>
          <img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotateZ(-90deg);" />
        </button>
      </div>
      <div id="search-container" class="horizontal-flex-container">
        <div id="search-input-container" class="horizontal-flex-container" style="position: relative; height: 100%;">
          <input id="search-input" class="primary-input" placeholder={$t["search.placeholder"] as string} bind:value={searchable}
            onkeydown={(e) => { if (e.key === 'Enter') startSearch(); if (e.key === 'Escape') stopSearch(); }}
          />
          <button id="search-close" class="transparent-button-highlight" onclick={() => stopSearch()}><img src="/close-x.svg" alt="Close" /></button>
        </div>
        <button id="search-button" class="primary-button vertical-flex-container" onclick={() => startSearch()}><img src="/search.svg" alt="Search" class="img-small" /></button>
      </div>
      <div class="element-wrapper-for-title vertical-flex-container">
        <p class="element-paragraph-title">{$t["date-input.description"]}</p>
        <div id="date-to-jump-container" class="horizontal-flex-container" style="position: relative;">
          <input class="primary-input" style="max-width: 110px; min-width: 95px; padding-right: 32px;" bind:value={dateToJump} placeholder={$t["placeholder.isodate"].slice(0, 7) as string} 
            onkeydown={(e) => { handleKeyDownOnInput("date", e); if (e.key === 'Escape') dateToJump = ''; if (e.key === 'Enter') handleDateJump(); }}
          />
          <button id="clear-date-to-jump" class="transparent-button-highlight" onclick={() => dateToJump = ''}><img src="/close-x.svg" alt="Close" /></button>
        </div>
      </div>
      <button class="primary-button horizontal-flex-container" onclick={() => handleDateJump()} disabled={inEditMode}>
        {$t["transactions-table.datejump.button"]}
        <img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotate(-90deg);" />
      </button>
    </div>
    <div class="transactions-table-toolbar-subbar primary-toolbar horizontal-flex-container">
      <button class="primary-button" style="min-width: 105px;" bind:this={openStatisticsButton} onclick={() => isStatisticsVisible = !isStatisticsVisible}>
        {$t[!isStatisticsVisible ? "transactions-table.statistics.show" : "transactions-table.statistics.hide"]}
      </button>
      <button class="primary-button horizontal-flex-container" style="min-width: 87px; justify-content: flex-start;" bind:this={openFormButton} onclick={() => isFormVisible = !isFormVisible} disabled={inEditMode}>
        <img src="/plus.svg" alt="Add" class="img-small" style="{isFormVisible ? 'transform: rotateZ(45deg)' : ''}; transition: transform 0.1s;" />{$t[isFormVisible ? "cancel.button" : "add.button"]}
      </button>
      <button class="primary-button horizontal-flex-container" title={$t["transactions-table.edit.button.hover-title"] as string} disabled={sortedFilteredTransactions.length <= 0 || isFormVisible}
        onclick={() => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) })}
      >
        <img src="/edit-pen.svg" alt="Edit" class="img-small" />{$t[inEditMode ? "exit.button": "edit.button"]}
      </button>
      <button class="primary-button horizontal-flex-container" title={inEditMode ? $t["transactions-table.save.button.hover-title"] as string : ""} disabled={!inEditMode}
        onclick={() => sendAlert({ message: "alert.transactions-table.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() })}
      >
        <img src="/disk.svg" alt="Save" class="img-small" />{$t["commit.button"]}
      </button>
    </div>
  </div>

  <div id="transactions-table">
    {#if selectedTransactionIds.size > 0 || inEditMode}
      <div id="transactions-table-edit-banner" class="vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
        <div id="edit-banner-top-bar" class="horizontal-flex-container">
          <p style="margin: 0;">{$t["transactions-table.edit-banner.header"]}</p>
          {#if inEditMode}
            <p class="opacity-breathing" style="position: absolute; right: 50%; transform: translateX(50%);">{$t["transactions-table.edit-banner.notification.header.editmode"]}</p>
          {/if}
          <button class="transparent-button-highlight" style="width: 32px; height: 32px;"
            onclick={() => sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode() })}
          >
            <img src="close-x.svg" alt="Close" class="img-small" />
          </button>
        </div>

        {#if !inEditMode}
          <p transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>{$t["transactions-table.edit-banner.paragraph"][0]} {selectedTransactionIds.size} {$t["transactions-table.edit-banner.paragraph"][1]}</p>
        {/if}

        <div id="edit-banner-buttons" class="horizontal-flex-container">
          <button class="primary-button horizontal-flex-container" title={$t["transactions-table.edit.button.hover-title"] as string} disabled={isFormVisible}
            onclick={() => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) })}
          >
            <img src="/edit-pen.svg" alt="Edit" />{$t[inEditMode ? "exit.button": "edit.button"]}
          </button>
          <button class="primary-button horizontal-flex-container" disabled={inEditMode}
            onclick={() => sendAlert({ message: "alert.transactions-table.delete.confirmation", isTimer: false, buttons: true, onConfirm: async () => handleDelete() })}
          >
            <img src="/trash-can.svg" alt="Trash" />{$t["delete.button"]}
          </button>
          {#if inEditMode}
            <button class="primary-button horizontal-flex-container" title={$t["transactions-table.save.button.hover-title"] as string} transition:fly={{ y: 24, duration: 200, easing: cubicInOut }}
              onclick={() => sendAlert({ message: "alert.transactions-table.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() })}
            >
              <img src="/disk.svg" alt="Save" />{$t["commit.button"]}
            </button>
          {/if}
        </div>

        <div class="horizontal-flex-container" style="gap: 2px;">
          {#each $t["transactions-table.edit-banner.note"] as text, i (i)}
            <p style="font-weight: {i === 0 ? "bold" : ""}; opacity: 0.5; font-size: 13px;">{text}</p>
          {/each}
        </div>
      </div>
    {/if}

    <div id="transactions-table-headers-container" class="table-flex-container" class:selected-txs={selectedTransactionIds.size > 0 || inEditMode}>
      <input type="checkbox" class="table-checkbox" style="align-self: center;" checked={sortedFilteredTransactions.length > 0 && selectedTransactionIds.size === sortedFilteredTransactions.length && !inEditMode}
        disabled={sortedFilteredTransactions.length <= 0 || inEditMode} onclick={() => inEditMode ? {} : handleSelectAll()}
      />
      {#each $t["transactions-table.thead.headers"] as header, i (i)}
        <button class="table-header transparent-button table-flex-container"
          class:currentlyOrderedBy={$sortData.column === columnsAndTypes[i]["column"]}
          class:transactions-table-cell-small={i === 0}
          class:transactions-table-cell-medium={[1, 5].includes(i)}
          class:transactions-table-cell-large={[3, 4].includes(i)}
          style="max-width: {i === 2 ? '380px' : ''};"
          onclick={() => orderBy(columnsAndTypes[i]["column"])}
        >
          {header}
          {#if $sortData.column === columnsAndTypes[i]["column"]}
            <img src={"/arrow.svg"} alt="Arrow" class="img-small"
              style="{$sortData.ascending ? 'transform: rotateZ(180deg);' : ""}; transition: {$sortData.column === columnsAndTypes[i]["column"] ? 'transform 0.1s' : ""};"
            />
          {/if}
        </button>
      {/each}
    </div>

    <div id="transactions-table-body-outer" bind:this={CONTAINER} onscroll={handleVirtualList}>
      <div style="height: {sortedFilteredTransactions.length * ITEM_HEIGHT}px; position: relative;">
        <div id="transactions-table-body" class="vertical-flex-container" style="position: absolute; top: 0; left: 0; right: 0; transform: translateY({start * ITEM_HEIGHT}px);">
          {#if sortedFilteredTransactions.length > 0}
            {#each displayTransactions as transaction (transaction.id)}
              <div role="menuitem" tabindex="0" class="table-row table-flex-container" style="cursor: {inEditMode ? "default" : "pointer"};" onclick={() => inEditMode ? {} : handleSelect(transaction.id)} onkeydown={(e) => { if (e.key === "Enter") inEditMode ? {} : handleSelect(transaction.id)}}>
                <input type="checkbox" class="table-checkbox" checked={selectedTransactionIds.has(transaction.id) && !inEditMode} disabled={inEditMode} />
                <div class="table-cell table-flex-container transactions-table-cell-small">{transaction.id}</div>

                {#if inEditMode}
                  <div class="table-cell-edit table-flex-container transactions-table-cell-medium"><input class="primary-input" bind:value={transaction.date} onkeydown={(e) => handleKeyDownOnInput("date", e)} /></div>
                  <div class="table-cell-edit table-flex-container" style="justify-content: flex-end; max-width: 380px;">
                    <input class="primary-input" style="padding-right: 74px;" type="number" min="0" step="0.01" bind:value={transaction.amount} onkeydown={(e) => handleKeyDownOnInput("amount", e)} oninput={(e) => handleNumberInput(e.target)} />
                    <div class="transactions-table-amount-steppers-container horizontal-flex-container" style="position: absolute; gap: 6px; margin-right: 6px;">
                      <button class="transparent-button-highlight vertical-flex-container" type="button" onclick={(e) => handleNumberStepper("increase", e.target)}><img src="/arrow.svg" alt="Increase" class="img-small" style="transform: rotate(180deg);" /></button>
                      <button class="transparent-button-highlight vertical-flex-container" type="button" onclick={(e) => handleNumberStepper("decrease", e.target)}><img src="/arrow.svg" alt="Decrease" class="img-small" /></button>
                    </div>
                  </div>
                  <div class="table-cell-edit table-flex-container transactions-table-cell-large"><select class="primary-input" bind:value={transaction.category} onchange={(e) => changeDisplayType(e.target, transaction)}>
                    {#each categoryOptions as option (option.value)}
                      <option value={option.value}>{option.label}</option>
                    {/each}
                  </select></div>
                  <div class="table-cell-edit table-flex-container transactions-table-cell-large">
                    <input class="primary-input" bind:value={transaction.description} />
                  </div>
                {:else}
                  <div class="table-cell table-flex-container transactions-table-cell-medium">{transaction.date}</div>
                  <div class="table-cell table-flex-container" style="max-width: 380px;">{transaction._type === "income" ? transaction.amount : -transaction.amount}</div>
                  <div class="table-cell table-flex-container transactions-table-cell-large">
                    {(() => {
                      const item = combinedCategories.find((item) => item.value === transaction.category);
                      return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
                    })()}
                  </div>
                  <div class="table-cell table-flex-container transactions-table-cell-large" title={transaction.description}><span>{transaction.description}</span></div>
                {/if}

                <div class="table-cell table-flex-container transactions-table-cell-medium">
                  <span class="table-cell-type" style="background-color: {transaction._type === "expense" ? "rgba(195, 70, 70, 0.2)" : "rgba(170, 255, 170, 0.2)"}; outline: 1px solid {transaction._type === "expense" ? "#c34646" : "#aaffaa"}">
                    { $t[`transaction-table.type.${transaction._type}`] }
                  </span>
                </div>
              </div>
            {/each}
          {:else}
            <div class="vertical-flex-container" style="margin-top: 120px;">
              <h3>{$t["transactions-table.no-transactions"]}</h3>
              <img src="/credit-card.svg" alt="Card" style="width: 240px; height: 180px;" />
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .primary-input {
    background-color: #0f0f0f;
    color: #f6f6f6;
  }

  .overlay-container {
    position: absolute;
    z-index: 500;
    left: 4px;
    top: 108px;
    max-height: calc(100% - 112px);
    border-radius: 8px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  #transactions-table-main-container, #transactions-table {
    position: relative;
    height: 100%;
    width: 100%;
    justify-content: flex-start;
  }

  #transactions-table-body-outer {
    position: absolute;
    top: 0;
    bottom: var(--table-body-outer-bottom);
    width: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    transform: translateY(var(--table-body-outer, 0));
    transition: transform 300ms ease-in-out, bottom 300ms ease-in-out;
  }

  #transactions-table-body {
    align-items: unset;
    width: 100%;
    overflow: hidden;
    padding: 10px 4px;
  }

  #transactions-table-toolbar {
    width: 100%;
    height: 112px;
  }

  #transactions-table-toolbar button {
    gap: 8px;
  }

  #transactions-table-toolbar-controls {
    gap: 6px;
  }

  #transactions-table-toolbar-controls button {
    flex-shrink: 0;
    height: 28px;
    width: 28px;
  }

  #transactions-table-headers-container {
    position: sticky;
    top: 0;
    height: 36px;
    padding: 4px 10px;
    border-bottom: 1px solid #333;
    background-color: #0f0f0f;
  }
  .selected-txs {
    border-top: 1px solid #333;
  }

  #transactions-table-headers-container button:hover {
    color: rgba(255, 70, 70, 1);
  }

  #transactions-table-edit-banner {
    position: relative;
    width: calc(100% - 20px);
    align-items: flex-start;
    margin: 10px;
    padding: 16px;
    border-radius: 8px;
    background-color: #222;
    user-select: none;
  }
  #transactions-table-edit-banner > *:not(:nth-last-child(-n + 2)) {
    margin-bottom: 24px;
  }

  #edit-banner-top-bar {
    width: 100%;
    justify-content: space-between;
  }

  #edit-banner-top-bar p {
    font-weight: bold;
  }

  #edit-banner-buttons {
    justify-content: flex-start;
    gap: 16px;
  }

  #edit-banner-buttons button {
    justify-content: flex-start;
    gap: 8px;
    padding: 12px 16px;
  }

  #edit-banner-buttons button img {
    width: 20px;
    height: 20px;
  }

  .currentlyOrderedBy {
    color: rgba(255, 70, 70, 1);
  }

  #search-container {
    height: 31px;
    gap: 1px;
    background-color: #333;
    border-radius: 4px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  #search-input-container #search-close, #date-to-jump-container #clear-date-to-jump {
    position: absolute;
    right: 6px;
    flex-shrink: 0;
    height: 20px;
    width: 20px;
  }

  #search-input, #date-to-jump-container .primary-input {
    font-size: unset;
  }

  #search-close img, #clear-date-to-jump img {
    width: 10px;
    height: 10px;
  }

  #search-input {
    border-radius: 4px 0 0 4px;
    background: #333;
    max-width: 180px;
    outline: none;
    padding-right: 32px;
  }

  #search-container #search-button {
    border-radius: 0 4px 4px 0;
    transform: none;
    box-shadow: none;
  }

  .transactions-table-amount-steppers-container button {
    padding: 6px;
    border-radius: 4px;
  }
</style>