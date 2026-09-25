<script lang="ts">
  import { slide, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { writable } from "svelte/store";
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { onNavigate } from "$app/navigation";

  import { sendAlert } from "$lib/alert";
  import { transactions, expenseCategories, incomeCategories, deleteTransaction, updateTransaction, getTransactions } from "$lib/transactions";
  import { t } from "$lib/i18n/i18n";
  import type { Transaction } from "$lib/types";
  import { handleKeyDownOnInput, handleNumberInput } from "$lib/actions";

  import AddTransactionForm from "../../components/AddTransactionForm.svelte";
  import StatisticsOverlay from "../../components/transactions-table/StatisticsOverlay.svelte";
  import SearchBar from "../../components/SearchBar.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";

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
  let searchRegex = $state<RegExp | null>(null);
  const inSearchMode = $derived(searchRegex !== null ? true : false);
  let clearSearch = $state<{ runClearSearch: () => void} | null>(null)
  let inEditMode = $state<boolean>(false);
  let openFormButton = $state<HTMLButtonElement | null>(null);
  let openStatisticsButton = $state<HTMLButtonElement | null>(null);

  let CONTAINER = $state<HTMLDivElement | null>(null);
  let CONTAINER_HEIGHT = $state<number | null>(null);
  const ITEM_HEIGHT = 56;
  const BUFFER = 5;
  const VISIBLE_ITEMS = $derived(Math.ceil((CONTAINER_HEIGHT ?? 0) / ITEM_HEIGHT));
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

  const toolBarNavButtons = [
    { label: 'Refresh transactions', img: '/refresh.svg', command: async () => await refreshTransactions() },
    { label: 'Previous month', img: '/arrow.svg', command: () => handleMonthChange(-1) },
    { label: 'Next month', img: '/arrow.svg', command: () => handleMonthChange(1) },
  ];

  let toolBarLowerButtons = $state([
    {
      get text() { return $t[!isStatisticsVisible ? "transactions-table.statistics.show" : "transactions-table.statistics.hide"]; },
      img: null,
      command: () => isStatisticsVisible = !isStatisticsVisible,
      get bind() { return openStatisticsButton; },
      set bind(el) { openStatisticsButton = el; },
      disabled: null,
    },
    { 
      get text() { return $t[isFormVisible ? "cancel.button" : "add.button"]; },
      img: '/plus.svg',
      command: () => isFormVisible = !isFormVisible,
      get bind() { return openFormButton; },
      set bind(el) { openFormButton = el; },
      get disabled() { return inEditMode; },
    },
    {
      get text() { return $t[inEditMode ? "exit.button": "edit.button"]; },
      img: '/edit-pen.svg',
      command: () => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) }),
      bind: null,
      get disabled() { return sortedFilteredTransactions.length <= 0 || isFormVisible; },
    },
    {
      text: $t["commit.button"],
      img: '/disk.svg',
      command: () => sendAlert({ message: "alert.transactions-table.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() }),
      bind: null,
      get disabled() { return !inEditMode; },
    },
  ]);

  const editBannerButtons = [
    {
      get text() { return $t[inEditMode ? "exit.button": "edit.button"]; },
      img: '/edit-pen.svg',
      command: () => !inEditMode ? enterEditMode() : sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode(false) }),
      get disabled() { return isFormVisible; },
      show: true,
    },
    {
      get text() { return $t["delete.button"]; },
      img: '/trash-can.svg',
      command: () => sendAlert({ message: "alert.transactions-table.delete.confirmation", isTimer: false, buttons: true, onConfirm: async () => handleDelete() }),
      get disabled() { return inEditMode; },
      show: true,
    },
    {
      get text() { return $t["commit.button"]; },
      img: '/disk.svg',
      command: () => sendAlert({ message: "alert.transactions-table.save-changes.confirmation", isTimer: false, buttons: true, onConfirm: () => commitChanges() }),
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

  $effect(() => {
    if (selectedTransactionIds.size > 0 && !inEditMode) {
      CONTAINER?.style.setProperty('--table-body-outer', "278px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "278px");
    } else if (inEditMode) {
      CONTAINER?.style.setProperty('--table-body-outer', "230px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "230px");
    } else {
      CONTAINER?.style.setProperty('--table-body-outer', "36px");
      CONTAINER?.style.setProperty('--table-body-outer-bottom', "36px");
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
      sendAlert({ message: "alert.saving.no-changes", isTimer: true, buttons: false });
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
    clearSearch?.runClearSearch();
  };

  const handleDateJump = () => {
    if (dateToJump.trim() === '') return;
    const dateParts = dateToJump.split("-");
    if (!/^\d{4}$/.test(dateParts[0])) { sendAlert({ message: "alert.invalid-year", isTimer: true, buttons: false }); return; }
    if (!/^([1-9]|0[1-9]|1[0-2])$/.test(dateParts[1])) { sendAlert({ message: "alert.invalid-month", isTimer: true, buttons: false }); return; }
    const dateObject = new Date(dateParts[0] + '-' + dateParts[1].padStart(2, '0') + '-01');
    current = dateObject;
    clearSearch?.runClearSearch();
  };

  const orderBy = (column: string) => {
    sortData.update(current => current.column === column
      ? { column, ascending: !current.ascending }
      : { column, ascending: true });
  };

</script>

{#if isFormVisible}
  <ModalWrapper options={{ position: { left: 4, top: 108, isPositionAbsolute: true }, transition: { type: "slide", duration: 300, easing: "cubic-in-out", axis: "y" }}}>
    <AddTransactionForm options={{ closeForm: () => isFormVisible = false, calendarStartDate: current, ignorableEls: [openFormButton] }} />
  </ModalWrapper>
{/if}

{#if isStatisticsVisible}
  <ModalWrapper options={{ position: { left: 4, top: 108, isPositionAbsolute: true }, transition: { type: "slide", duration: 300, easing: "cubic-in-out", axis: "y" }}}>
    <StatisticsOverlay setVisibility={(state) => { isStatisticsVisible = state; }} ignorableEls={[openStatisticsButton]} />
  </ModalWrapper>
{/if}

<div id="transactions-table-main-container" class="flex column">
  <div id="transactions-table-toolbar" class="flex column">
    <div class="transactions-table-toolbar-subbar primary-toolbar flex row">
      <div id="transactions-table-toolbar-controls" class="flex row">
        {#each toolBarNavButtons as button, i (i)}
          <button aria-label="{button.label}" class="button-primary transparent highlight" onclick={button.command} disabled={[1, 2].includes(i) && inEditMode}>
            <span class="span-icon img-small" style="mask-image: url('{button.img}'); {[1, 2].includes(i) && `transform: rotateZ(${i === 1 ? 90 : -90}deg);`}"></span>
          </button>
        {/each}
      </div>
      <SearchBar options={{
        sendRegexToParent: (regex) => { searchRegex = regex; },
        getClearSearch: (func) => { clearSearch = func; },
        addFunctionsToClearSearch: [emptySortData],
        mirrorSearchBar: true,
        }}
      />
      <div id="date-to-jump-wrapper" class="flex row">
        <div id="date-to-jump-container" class="flex row" style="position: relative;">
          <input class="primary-input" style="max-width: 110px; min-width: 95px; padding-right: 2rem" bind:value={dateToJump} placeholder={$t["placeholder.isodate"].slice(0, 7) as string} 
            onkeydown={(e) => { handleKeyDownOnInput("date", e); if (e.key === 'Escape') dateToJump = ''; if (e.key === 'Enter') handleDateJump(); }}
          />
          <button aria-label="Clear search" id="clear-date-to-jump" class="button-primary transparent highlight" onclick={() => dateToJump = ''}>
            <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
          </button>
        </div>
        <button aria-label="Move to date" class="button-primary transparent highlight outline default-corners" onclick={() => handleDateJump()} disabled={inEditMode}>
          {$t["transactions-table.datejump.button"]}
          <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transform: rotate(-90deg);"></span>
        </button>
      </div>
    </div>
    <div class="transactions-table-toolbar-subbar primary-toolbar flex row">
      {#each toolBarLowerButtons as button, i (i)}
        <button class="button-primary {i === 3 ? 'white-bg' : 'transparent highlight outline default-corners'}" style="{i === 0 && 'min-width: 105px'};" bind:this={button.bind} onclick={button.command} disabled={button.disabled}>
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
      <div id="transactions-table-edit-banner" class="flex column" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
        <div id="edit-banner-top-bar" class="flex row">
          <p style="margin: 0;">
            {$t["transactions-table.edit-banner.header"]}
          </p>
          {#if inEditMode}
            <p class="opacity-breathing" style="position: absolute; right: 50%; transform: translateX(50%);">{$t["transactions-table.edit-banner.notification.header.editmode"]}</p>
          {/if}
          <button aria-label="Close banner" class="button-primary transparent highlight static"
            onclick={() => sendAlert({ message: "alert.transactions-table.toggle-edit.confirmation", isTimer: false, buttons: true, onConfirm: () => exitEditMode() })}
          >
            <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
          </button>
        </div>

        {#if !inEditMode}
          <p transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
            {$t["transactions-table.edit-banner.paragraph"][0]} {selectedTransactionIds.size} {$t["transactions-table.edit-banner.paragraph"][1]}
          </p>
        {/if}

        <div id="edit-banner-buttons" class="flex row">
          {#each editBannerButtons as button, i (i)}
            {#if button.show}
              <button class="button-primary {i === 2 ? 'white-bg' : 'light'}" disabled={button.disabled} onclick={button.command} transition:fly={i === 2 ? { y: 24, duration: 200, easing: cubicInOut } : undefined}>
                <span class="span-icon" style="mask-image: url('{button.img}');"></span>
                {button.text}
              </button>
            {/if}
          {/each}
        </div>

        <div class="flex row" style="gap: 2px;">
          {#each $t["transactions-table.edit-banner.note"] as text, i (i)}
            <p style="font-weight: {i === 0 ? "bold" : ""}; opacity: 0.5; font-size: 0.75rem;">{text}</p>
          {/each}
        </div>
      </div>
    {/if}

    <div id="transactions-table-headers-container" class="table-flex-container" class:selected-txs={selectedTransactionIds.size > 0 || inEditMode}>
      <input type="checkbox" class="table-checkbox" style="align-self: center;" checked={sortedFilteredTransactions.length > 0 && selectedTransactionIds.size === sortedFilteredTransactions.length && !inEditMode}
        disabled={sortedFilteredTransactions.length <= 0 || inEditMode} onclick={() => inEditMode ? {} : handleSelectAll()}
      />
      {#each $t["transactions-table.thead.headers"] as header, i (i)}
        <button class="table-header button-primary transparent table-flex-container"
          class:currentlyOrderedBy={$sortData.column === columnsAndTypes[i]["column"]}
          class:transactions-table-cell-small={i === 0}
          class:transactions-table-cell-medium={[1, 5].includes(i)}
          class:transactions-table-cell-large={[3, 4].includes(i)}
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

    <div id="transactions-table-body-outer" bind:this={CONTAINER} bind:clientHeight={CONTAINER_HEIGHT} onscroll={handleVirtualList}>
      <div style="height: {sortedFilteredTransactions.length * ITEM_HEIGHT + 20}px; position: relative;">
        <div id="transactions-table-body" class="flex column" style="transform: translateY({start * ITEM_HEIGHT}px);">
          {#if sortedFilteredTransactions.length > 0}
            {#each displayTransactions as transaction (transaction.id)}
              <div role="menuitem" tabindex="0" class="table-row table-flex-container" style="cursor: {inEditMode ? "default" : "pointer"};"
                onclick={() => inEditMode ? {} : handleSelect(transaction.id)}
                onkeydown={(e) => { if (e.key === "Enter") inEditMode ? {} : handleSelect(transaction.id)}}
              >
                <input type="checkbox" class="table-checkbox" checked={selectedTransactionIds.has(transaction.id) && !inEditMode} disabled={inEditMode} />
                <div class="table-cell table-flex-container transactions-table-cell-small">
                  {transaction.id}
                </div>

                {#if inEditMode}
                  <div class="table-cell-edit table-flex-container transactions-table-cell-medium"><input class="primary-input" bind:value={transaction.date} onkeydown={(e) => handleKeyDownOnInput("date", e)} /></div>
                  <div class="table-cell-edit table-flex-container" style="justify-content: flex-end; max-width: 380px;">
                    <input class="primary-input" style="padding-right: 56px;" type="number" min="0" step="0.01" bind:value={transaction.amount} onkeydown={(e) => handleKeyDownOnInput("amount", e)} oninput={(e) => handleNumberInput(e.target)} />
                    <div class="transactions-table-amount-steppers-container flex row" style="position: absolute; gap: 0.25rem; margin-right: 6px;">
                      <button aria-label="Increase amount" class="button-primary transparent highlight default-corners" type="button" onclick={(e) => handleNumberStepper("increase", e.target)}>
                        <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate(180deg);"></span>
                      </button>
                      <button aria-label="Decrease amount" class="button-primary transparent highlight default-corners" type="button" onclick={(e) => handleNumberStepper("decrease", e.target)}>
                        <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg');"></span>
                      </button>
                    </div>
                  </div>
                  <div class="table-cell-edit table-flex-container transactions-table-cell-large">
                    <select class="primary-input" bind:value={transaction.category} onchange={(e) => changeDisplayType(e.target, transaction)}>
                      {#each categoryOptions as option (option.value)}
                        <option value={option.value}>{option.label}</option>
                      {/each}
                    </select>
                  </div>
                  <div class="table-cell-edit table-flex-container transactions-table-cell-large">
                    <input class="primary-input" bind:value={transaction.description} />
                  </div>
                {:else}
                  <div class="table-cell table-flex-container transactions-table-cell-medium">
                    {transaction.date}
                  </div>
                  <div class="table-cell table-flex-container" style="max-width: 380px;">
                    {transaction._type === "income" ? transaction.amount : -transaction.amount}
                  </div>
                  <div class="table-cell table-flex-container transactions-table-cell-large">
                    {(() => {
                      const item = combinedCategories.find((item) => item.value === transaction.category);
                      return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
                    })()}
                  </div>
                  <div class="table-cell table-flex-container transactions-table-cell-large" title={transaction.description}>
                    <span>
                      {transaction.description}
                    </span>
                  </div>
                {/if}

                <div class="table-cell table-flex-container transactions-table-cell-medium">
                  <span class="table-cell-type"
                    style="
                      background-color: {transaction._type === "expense"? "var(--color-negative-dimmed)" : "var(--color-positive-dimmed)"};
                      outline: 1px solid {transaction._type === "expense" ? "var(--color-negative)" : "var(--color-positive)"}
                    "
                  >
                    { $t[`transaction-table.type.${transaction._type}`] }
                  </span>
                </div>
              </div>
            {/each}
          {:else}
            <div class="flex column" style="margin-top: 120px;">
              <h3>{$t["transactions-table.no-transactions"]}</h3>
              <span class="span-icon" style="mask-image: url('/credit-card.svg'); width: 240px; height: 180px; mask-position: center;"></span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .primary-input {
    background-color: var(--color-primary2);
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
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));
  }

  #transactions-table-body {
    position: absolute;
    top: 0;
    left: 0;
    right: 0; 
    align-items: unset;
    width: 100%;
    overflow: hidden;
    padding: 10px 0.25rem;
  }

  #transactions-table-toolbar {
    width: 100%;
    height: 112px;
  }

  #transactions-table-toolbar-controls {
    gap: 0.25rem;
  }

  #transactions-table-headers-container {
    position: sticky;
    top: 0;
    height: 2.25rem;
    padding: 0.25rem 10px;
    border-bottom: 1px solid var(--outline-color1);
    background-color: var(--color-primary2);
  }
  .selected-txs {
    border-top: 1px solid var(--outline-color1);
  }

  #transactions-table-headers-container button:hover {
    color: var(--color-highlight1);
  }

  #transactions-table-edit-banner {
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

  .currentlyOrderedBy {
    color: var(--color-highlight1);
  }

  #date-to-jump-wrapper {
    align-items: unset;
    gap: 0.5rem;
    padding: 0.25rem;
    border-radius: 0.25rem;
    background-color: var(--color-secondary1);

    #date-to-jump-container {
      outline: 1px solid var(--outline-color1);
      border-radius: 0.25rem;
    
      #clear-date-to-jump {
        position: absolute;
        right: 6px;
        flex-shrink: 0;
        height: 20px;
        width: 20px;
      }

      .primary-input {
        background-color: transparent;
        font-size: unset;
        outline: none;
      }

      span {
        width: 10px;
        height: 10px;
      }
    }
  }

  .transactions-table-amount-steppers-container button.button-primary.transparent.highlight {
    height: fit-content;
    padding: 0.25rem;
  }
</style>