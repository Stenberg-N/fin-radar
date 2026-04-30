<script lang="ts">
  import { slide, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { writable, get } from "svelte/store";
  import { onMount } from "svelte";

  import { sendAlert } from "$lib/alert";
  import { user } from "$lib/user";
  import { transactions, expenseCategories, incomeCategories, deleteTransaction, updateTransaction, getTransactions } from "$lib/transactions";
  import { t } from "$lib/i18n";
  import type { Transaction } from "$lib/types";
  import { handleKeyDownOnInput, handleNumberInput } from "$lib/functions";

  import AddTransactionForm from "../../components/AddTransactionForm.svelte";

  const combinedCategories = [...expenseCategories, ...incomeCategories];
  let selectedTransactionIds = $state<number[]>([]);
  let current = $state(new Date());
  let isFormVisible = $state<boolean>(false);
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
  let searchRegex = $state<RegExp | string>('');
  let inSearchMode = $derived.by(() => {return searchRegex !== '' ? true : false; });

  let CONTAINER = $state<HTMLDivElement | null>(null);
  const ITEM_HEIGHT = 56;
  const BUFFER = 5;
  const VISIBLE_ITEMS = $derived(Math.ceil((CONTAINER ? CONTAINER.clientHeight : 0) / ITEM_HEIGHT));
  let scrollTop = $state<number>(0);
  let HIGH_WATERMARK = $state(0); // Keeps track of the furthest row from index 0 to not un-render previous rows.

  let inEditMode = $state<boolean>(false);
  let editedTransactions = $state<Transaction[]>([]);
  let originalTransactions = $state<Transaction[]>([]);
  let editableTransactions = $state<Transaction[]>([]);
  let displayTransactions = $state<Transaction[]>([]);

  onMount(() => {
    handleVirtualList();
  });

  $effect(() => {
    const tableBodyOuter = document.getElementById("transactions-table-body-outer");
    if (selectedTransactionIds.length > 0 || inEditMode) tableBodyOuter?.style.setProperty('--table-body-outer', "303px");
    else tableBodyOuter?.style.setProperty('--table-body-outer', "84px");
  });

  $effect(() => {
    if (!$user) return;

    const yearMonth = `${String(current.getFullYear())}-${String(current.getMonth() + 1).padStart(2, '0')}`;
    const timer = setTimeout(async () => {
      await getTransactions($user.id, yearMonth, $user.name);
      emptySortData();
    }, 300);

    return () => clearTimeout(timer);
  });

  $effect(() => {
    const start = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - BUFFER);
    const end = Math.min($transactions.length, (start + VISIBLE_ITEMS + BUFFER * 2));

    /***********************************************************************************************************************************
    | displayTransactions does not get correctly updated if entering edit mode and NOT scrolling.
    |
    | Since the displayTransactions are inside a setTimeout, Svelte's $effect will not re-run if the states of the variables used inside the setTimeout change,
    |
    | hence these variables are captured before the setTimeout to make displayTransactions reactive again. 
    \***********************************************************************************************************************************/
    const _HIGH_WATERMARK = HIGH_WATERMARK;
    const _searchRegex = searchRegex;
    const _inSearchMode = inSearchMode;
    const _inEditMode = inEditMode;
    const _editableTransactions = editableTransactions;
    const _transactions = $transactions;

    const timer = setTimeout(() => {
      HIGH_WATERMARK = Math.max(_HIGH_WATERMARK, end);
      displayTransactions = _inEditMode && _inSearchMode && searchRegex !== ''
        ? _editableTransactions.filter(t => Object.values(t).some(val => (_searchRegex as RegExp).test(String(val))))
        : (_inSearchMode && searchRegex !== ''
          ? _transactions.filter(t => Object.values(t).some(val => (_searchRegex as RegExp).test(String(val))))
          : (_inEditMode
            ? _editableTransactions.slice(0, HIGH_WATERMARK)
            : _transactions.slice(0, HIGH_WATERMARK)));
    }, 100);

    return () => clearTimeout(timer);
  });

  /***********************************************************************************************************************************
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const tryDelete = async () => { if (!$user) return; const result = await deleteTransaction($user.id, selectedTransactionIds, $user.name); return result; };
  const closeForm = () => { isFormVisible = false; };
  const emptySortData = () => { sortData.set({ column: '', ascending: true }); };
  const handleVirtualList = () => { if (!CONTAINER) return; scrollTop = CONTAINER.scrollTop; };
  const loadAllTransactions = () => { if (HIGH_WATERMARK === $transactions.length) return; HIGH_WATERMARK = $transactions.length; };

  /***********************************************************************************************************************************/

  const handleSelect = (id: number) => {
    if (selectedTransactionIds.includes(id)) selectedTransactionIds = selectedTransactionIds.filter(tid => tid !== id);
    else selectedTransactionIds = [ ...selectedTransactionIds, id ];
  };

  const handleSelectAll = () => {
    if (selectedTransactionIds.length === $transactions.length) selectedTransactionIds = [];
    else selectedTransactionIds = $transactions.map(t => t.id);
  };

  const handleDelete = async () => {
    const result = await tryDelete();

    if (result?.success) {
      sendAlert("alert.transactions-table.delete.success", true, false, undefined, undefined, String(result.deleted));
      selectedTransactionIds = [];
    } else sendAlert("alert.transactions-table.delete.fail", true, false);
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
    if (clearIds !== false || clearIds === undefined) selectedTransactionIds = [];
  };

  const commitChanges = async () => {
    if (!$user) return;

    const originalMap = new Map(originalTransactions.map(t => [t.id, t]));
    const changed: Transaction[] = [];

    for (const edited of editableTransactions) {
      const original = originalMap.get(edited.id);
      if (!original) continue;

      if (
        edited.date !== original.date ||
        String(edited.amount) !== String(original.amount) ||
        edited.category !== original.category ||
        edited.description !== original.description
      ) {
        changed.push(edited);
      }
    }

    if (changed.length === 0) {
      sendAlert("alert.transactions-table.no-changes", true, false);
      exitEditMode();
      return;
    }

    editedTransactions = changed;
    const result = await updateTransaction($user.id, editedTransactions, $user.name);

    if (result.success) {
      sendAlert("alert.transactions-table.update.success", true, false, undefined, undefined, String(result.amount));
    } else {
      sendAlert("alert.transactions-table.update.fail", true, false);
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
    HIGH_WATERMARK = 0;
  };

  const handleDateJump = () => {
    if (dateToJump.trim() === '') return;
    const dateParts = dateToJump.split("-");
    if (!/^\d{4}$/.test(dateParts[0])) { sendAlert("alert.transactions-table.date-jump.invalid-year", true, false); return; }
    if (!/^0*([1-9]|1[0-2])$/.test(dateParts[1])) { sendAlert("alert.transactions-table.date-jump.invalid-month", true, false); return; }
    const dateObject = new Date(dateParts[0] + '-' + dateParts[1].padStart(2, '0') + '-01');
    current = dateObject;
    stopSearch();
  };

  const orderBy = (column: string, type: string) => {
    const newSort = get(sortData);

    if (newSort.column === column) {
      newSort.ascending = !newSort.ascending;
    } else {
      newSort.column = column;
      newSort.ascending = true;
    }

    displayTransactions = [ ...displayTransactions].sort((a, b) => {
      const aValue = a[column as keyof Transaction];
      const bValue = b[column as keyof Transaction];

      let order = 0;
      switch (type) {
        case "datetime": order = new Date(aValue).getTime() - new Date(bValue).getTime(); break;
        case "number": order = Number(aValue) - Number(bValue); break;
        case "text": order = String(aValue).localeCompare(String(bValue)); break;
      }

      return newSort.ascending ? order : -order;
    });

    sortData.set(newSort);
  };

  const startSearch = () => {
    if (!searchable || searchable.trim() === '') return;
    searchRegex = new RegExp(searchable.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
  };

  const stopSearch = () => {
    searchable = null;
    searchRegex = '';
    emptySortData();
  };

</script>

{#if isFormVisible}
  <div class="form-container vertical-flex-container" transition:slide={{ axis: "x", duration: 200, easing: cubicInOut }}>
    <AddTransactionForm closeForm={closeForm} />
  </div>
{/if}

<div class="horizontal-flex-container" style="position: fixed; left: 158px; top: 8px; height: 32px; justify-content: flex-start;">
  <p style="margin: 0; font-weight: bold;">{`${$t["calendar.monthnames"][current.getMonth()]}, ${current.getFullYear()}`}</p>
</div>

<div id="transactions-table-main-container" class="vertical-flex-container">
  <div id="transactions-table-toolbar" class="horizontal-flex-container">
    <button class="transparent-button-highlight horizontal-flex-container" class:disabled={inEditMode} disabled={inEditMode} onclick={() => handleMonthChange(-1)}><img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotateZ(90deg);" /></button>
    <button class="transparent-button-highlight horizontal-flex-container" class:disabled={inEditMode} disabled={inEditMode} onclick={() => handleMonthChange(1)}><img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotateZ(-90deg);" /></button>

    <button class="primary-button" style="min-width: 88px;" class:disabled={HIGH_WATERMARK === $transactions.length} disabled={HIGH_WATERMARK === $transactions.length} onclick={() => loadAllTransactions()}>{$t["tranasctions-table.show-all"]}</button>
    <button class="primary-button horizontal-flex-container" onclick={() => isFormVisible = !isFormVisible} class:disabled={inEditMode} disabled={inEditMode}>
      <img src="/plus.svg" alt="Add" class="img-small" style="{isFormVisible ? 'transform: rotateZ(45deg)' : ''}; transition: transform 0.1s;" />{$t[isFormVisible ? "cancel.button" : "add.button"]}
    </button>
    <button class="primary-button horizontal-flex-container" title={$t["transactions-table.edit.button.hover-title"] as string} class:disabled={$transactions.length <= 0 || isFormVisible} disabled={$transactions.length <= 0 || isFormVisible}
      onclick={() => !inEditMode ? enterEditMode() : sendAlert("alert.transactions-table.toggle-edit.confirmation", false, true, () => exitEditMode(false))}
    >
      <img src="/edit-pen.svg" alt="Edit" class="img-small" />{$t[inEditMode ? "exit.button": "edit.button"]}
    </button>
    <button class="primary-button horizontal-flex-container" title={inEditMode ? $t["transactions-table.save.button.hover-title"] as string : ""} class:disabled={!inEditMode} disabled={!inEditMode}
      onclick={() => sendAlert("alert.transactions-table.save-changes.confirmation", false, true, () => commitChanges())}
    >
      <img src="/disk.svg" alt="Save" class="img-small" />{$t["commit.button"]}
    </button>

    <div id="search-container" class="horizontal-flex-container">
      <div id="search-input-container" class="horizontal-flex-container" style="position: relative; height: 100%;">
        <input id="search-input" class="primary-input" placeholder={$t["transactions-table.search.placeholder"] as string} bind:value={searchable} />
        <button id="search-close" class="transparent-button-highlight" onclick={() => stopSearch()}><img src="/close-x.svg" alt="Close" /></button>
      </div>
      <button id="search-button" class="primary-button vertical-flex-container" onclick={() => startSearch()}><img src="/search.svg" alt="Search" class="img-small" /></button>
    </div>

    <div id="date-to-jump-container" class="horizontal-flex-container" style="position: relative; height: 28px;">
      <input class="primary-input" style="max-width: 100px;" bind:value={dateToJump} placeholder="2024-06" />
      <button id="clear-date-to-jump" class="transparent-button-highlight" onclick={() => dateToJump = ''}><img src="/close-x.svg" alt="Close" /></button>
    </div>
    <button class="primary-button horizontal-flex-container" onclick={() => handleDateJump()} class:disabled={inEditMode} disabled={inEditMode}>{$t["transactions-table.datejump.button"]}<img src="/arrow.svg" alt="Arrow" class="img-small" style="transform: rotate(-90deg);" /></button>
  </div>

  <div id="transactions-table">
    {#if selectedTransactionIds.length > 0 || inEditMode}
      <div id="transactions-table-edit-banner" class="vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
        <div id="edit-banner-top-bar" class="horizontal-flex-container">
          <p>{$t["transactions-table.edit-banner.header"]}</p>
          {#if inEditMode}
            <p class="opacity-breathing" style="position: absolute; right: 50%; transform: translateX(50%);">{$t["transactions-table.edit-banner.notification.header.editmode"]}</p>
          {/if}
          <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => sendAlert("alert.transactions-table.toggle-edit.confirmation", false, true, () => exitEditMode())}>
            <img src="close-x.svg" alt="Close" class="img-small" />
          </button>
        </div>

        <p>{$t["transactions-table.edit-banner.paragraph"][0]} {selectedTransactionIds.length} {$t["transactions-table.edit-banner.paragraph"][1]}</p>

        <div id="edit-banner-buttons" class="horizontal-flex-container">
          <button class="primary-button horizontal-flex-container" title={$t["transactions-table.edit.button.hover-title"] as string} class:disabled={isFormVisible} disabled={isFormVisible}
            onclick={() => !inEditMode ? enterEditMode() : sendAlert("alert.transactions-table.toggle-edit.confirmation", false, true, () => exitEditMode(false))}
          >
            <img src="/edit-pen.svg" alt="Edit" />{$t[inEditMode ? "exit.button": "edit.button"]}
          </button>
          <button class="primary-button horizontal-flex-container" class:disabled={inEditMode} disabled={inEditMode} onclick={() => sendAlert("alert.transactions-table.delete.confirmation", false, true, async () => handleDelete())}>
            <img src="/trash-can.svg" alt="Trash" />{$t["delete.button"]}
          </button>
          {#if inEditMode}
            <button class="primary-button horizontal-flex-container" title={$t["transactions-table.save.button.hover-title"] as string} transition:fly={{ y: -40, duration: 200, easing:cubicInOut }}
              onclick={() => sendAlert("alert.transactions-table.save-changes.confirmation", false, true, () => commitChanges())}
            >
              <img src="/disk.svg" alt="Save" />{$t["commit.button"]}
            </button>
          {/if}
        </div>

        <div class="horizontal-flex-container" style="gap: 2px;">
          {#each $t["transactions-table.edit-banner.note"] as text, i (i)}
            <p style="font-weight: {i === 0 ? "bold" : ""}; opacity: 0.3; font-size: 13px;">{text}</p>
          {/each}
        </div>
      </div>
    {/if}

    <div id="transactions-table-headers-container" class="table-grid-layout">
      <input type="checkbox" class="table-checkbox" style="align-self: center;" class:disabled={$transactions.length <= 0 || inEditMode} checked={$transactions.length > 0 && selectedTransactionIds.length === $transactions.length && !inEditMode}
        disabled={$transactions.length <= 0 || inEditMode} onclick={() => inEditMode ? {} : handleSelectAll()}
      />
      {#each $t["transactions-table.thead.headers"] as header, i (i)}
        <button class="table-header transparent-button horizontal-flex-container" class:currentlyOrderedBy={$sortData.column === columnsAndTypes[i]["column"]} onclick={() => orderBy(columnsAndTypes[i]["column"], columnsAndTypes[i]["type"])}>
          {header}
          <img src={$sortData.column === columnsAndTypes[i]["column"] ? "/arrow.svg" : "/arrows-up-down.svg"} alt="Arrow" class="img-small" 
            style="{$sortData.ascending ? 'transform: rotateZ(180deg);' : ""}; transition: {$sortData.column === columnsAndTypes[i]["column"] ? 'transform 0.1s' : ""};"
          />
        </button>
      {/each}
    </div>

    <div id="transactions-table-body-outer" bind:this={CONTAINER} onscroll={handleVirtualList}>
      <div id="transactions-table-body" class="vertical-flex-container">
        {#if $transactions.length > 0}
          {#each displayTransactions as transaction (transaction.id)}
            <div role="menuitem" tabindex="0" class="table-row table-grid-layout horizontal-flex-container" style="cursor: {inEditMode ? "default" : "pointer"};" onclick={() => inEditMode ? {} : handleSelect(transaction.id)} onkeydown={(e) => { if (e.key === "Enter") inEditMode ? {} : handleSelect(transaction.id)}}>
              <input type="checkbox" class="table-checkbox" checked={selectedTransactionIds.includes(transaction.id) && !inEditMode} class:disabled={inEditMode} disabled={inEditMode} />
              <div class="table-cell">{transaction.id}</div>

              {#if inEditMode}
                <div class="table-cell-edit"><input class="primary-input" bind:value={transaction.date} onkeydown={(e) => handleKeyDownOnInput("date", e)} /></div>
                <div class="table-cell-edit horizontal-flex-container" style="justify-content: flex-end;">
                  <input class="primary-input" style="padding-right: 82px;" type="number" min="0" step="0.01" bind:value={transaction.amount} onkeydown={(e) => handleKeyDownOnInput("amount", e)} oninput={(e) => handleNumberInput(e.target)} />
                  <div class="transactions-table-amount-steppers-container horizontal-flex-container" style="position: absolute; gap: 6px; margin-right: 6px;">
                    <button class="primary-button vertical-flex-container" type="button" onclick={(e) => handleNumberStepper("increase", e.target)}><img src="/arrow.svg" alt="Increase" class="img-small" style="transform: rotate(180deg);" /></button>
                    <button class="primary-button vertical-flex-container" type="button" onclick={(e) => handleNumberStepper("decrease", e.target)}><img src="/arrow.svg" alt="Decrease" class="img-small" /></button>
                  </div>
                </div>
                <div class="table-cell-edit"><select class="primary-input" bind:value={transaction.category} onchange={(e) => changeDisplayType(e.target, transaction)}>
                  {#each combinedCategories as option, i (i)}
                    <option value={option.value}>{($t[option.parent] as Array<Record<string, string>>)[option.index][option.key]}</option>
                  {/each}
                </select></div>
                <div class="table-cell-edit"><input class="primary-input" bind:value={transaction.description} /></div>
              {:else}
                <div class="table-cell">{transaction.date}</div>
                <div class="table-cell">{transaction.amount}</div>
                <div class="table-cell">
                  {(() => {
                    const item = combinedCategories.find((item) => item.value === transaction.category);
                    return item ? ($t[item.parent] as Array<Record<string, string>>)[item.index][item.key] : 'Unknown';
                  })()}
                </div>
                <div class="table-cell">{transaction.description}</div>
              {/if}

              <div class="table-cell"><span style="background-color: {transaction._type === "expense" ? "#c34646" : "#73c873"}">{ $t[`transaction-table.type.${transaction._type}`] }</span></div>
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

<style>
  #transactions-table-main-container, #transactions-table {
    height: 100%;
    width: 100%;
    justify-content: flex-start;
  }

  #transactions-table-body-outer {
    position: absolute;
    top: var(--table-body-outer);
    bottom: 0;
    width: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable;
    transition: top 300ms cubic-bezier(0.645, 0.045, 0.355, 1);
  }

  #transactions-table-body {
    align-items: unset;
    width: 100%;
    overflow: hidden;
    padding: 10px;
  }

  #transactions-table-toolbar {
    justify-content: flex-start;
    height: 48px;
    width: 100%;
    gap: 12px;
    padding: 8px;
    border-bottom: 1px solid #333;
    user-select: none;
  }

  #transactions-table-toolbar button {
    gap: 8px;
  }
  #transactions-table-toolbar button:nth-child(-n+2) {
    height: 32px;
    width: 32px;
  }
  #transactions-table-toolbar button:nth-child(-n+2).disabled:hover {
    background-color: transparent;
  }
  #transactions-table-toolbar > button:not(.disabled):nth-of-type(-n+2):hover {
    outline: 1px solid rgba(255, 70, 70, 1);
  }

  #transactions-table-headers-container {
    position: sticky;
    top: 0;
    height: 36px;
    margin-bottom: 2px;
    padding: 6px 26px 6px 20px;
    border-bottom: 1px solid #333;
    background-color: #0f0f0f;
  }

  #transactions-table-headers-container button {
    justify-content: space-between;
  }
  #transactions-table-headers-container button:hover {
    background-color: #222;
  }

  #transactions-table-edit-banner {
    position: relative;
    width: calc(100% - 20px);
    align-items: flex-start;
    margin: 10px;
    padding: 16px;
    border-radius: 12px;
    background-color: #181818;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
    user-select: none;
  }
  #transactions-table-edit-banner > *:not(:nth-last-child(-n + 2)) {
    margin-bottom: 24px;
  }

  #transactions-table-edit-banner p {
    margin: 0;
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

  .disabled {
    opacity: 0.5;
    outline: none;
  }
  .disabled:hover {
    cursor: not-allowed;
    transform: none;
    background-color: #222;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  .primary-input {
    font-size: 15px;
    color: #f6f6f6;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  .form-container {
    position: absolute;
    z-index: 500;
    left: 4px;
    top: 52px;
    height: calc(100% - 56px);
  }

  .currentlyOrderedBy {
    color: rgba(255, 70, 70, 1);
    background-color: #222;
  }

  #search-container {
    height: 32px;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  #search-input-container #search-close, #date-to-jump-container #clear-date-to-jump {
    position: absolute;
    right: 6px;
    height: 20px;
    width: 20px;
  }

  #search-close img, #clear-date-to-jump img {
    width: 10px;
    height: 10px;
  }

  #search-input {
    border-radius: 8px 0 0 8px;
    background: #222;
    max-width: 180px;
    outline: none;
    padding-right: 32px;
  }
  #search-input:focus {
    border: 2px solid rgba(255, 70, 70, 1);
  }

  #search-container #search-button {
    border-radius: 0 8px 8px 0;
    transform: none;
    outline: none;
    box-shadow: none;
  }
</style>