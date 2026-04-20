<script lang="ts">
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { sendAlert } from "$lib/alert";
  import { user } from "$lib/user";
  import { transactions, expenseCategories, incomeCategories, deleteTransaction, updateTransaction } from "$lib/transactions";
  import { t } from "$lib/i18n";
  import type { Transaction } from "$lib/types";

  const combinedCategories = [...expenseCategories, ...incomeCategories];
  let selectedTransactionIds = $state<number[]>([]);

  let inEditMode = $state<boolean>(false);
  let editedTransactions = $state<Transaction[]>([]);
  let originalTransactions = $state<Transaction[]>([]);
  let editableTransactions = $state<Transaction[]>([]);
  let displayTransactions = $derived(inEditMode ? editableTransactions : $transactions);

  $effect(() => {
    const tableBodyOuter = document.getElementById("transactions-table-body-outer");
    if (selectedTransactionIds.length > 0 || inEditMode) tableBodyOuter?.style.setProperty('--table-body-outer', "231px");
    else tableBodyOuter?.style.setProperty('--table-body-outer', "36px");
  });

  /*
  **********************************************************************************************************************************

  Context, Helper & Wrapper functions

  **********************************************************************************************************************************
  */
  const tryDelete = async () => { if (!$user) return; const result = await deleteTransaction($user.id, selectedTransactionIds, $user.name); return result; };
  const clearEditAndIds = () => { selectedTransactionIds = []; inEditMode = false; };

  /* ********************************************************************************************************************************** */

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
      inEditMode = false;
      return;
    }

    editedTransactions = changed;
    const result = await updateTransaction($user.id, editedTransactions, $user.name);

    if (result.success) {
      sendAlert("alert.transactions-table.update.success", true, false, undefined, undefined, String(result.amount));
    } else {
      sendAlert("alert.transactions-table.update.fail", true, false);
    }
    clearEditAndIds();
  };

</script>

<div class="horizontal-flex-container" style="position: fixed; top: 8px; right: 100px; height: 32px; gap: 12px;">
  <button class="primary-button horizontal-flex-container" style="gap: 8px;" class:disabled={!inEditMode} disabled={!inEditMode} onclick={() => commitChanges()}><img src="/disk.svg" alt="Save" class="img-small" style="filter: brightness(0) invert(0.9);" />{$t["commit.button"]}</button>
  <button class="primary-button horizontal-flex-container" style="gap: 8px;" onclick={() => !inEditMode ? enterEditMode() : inEditMode = false}>
    <img src="/edit-pen.svg" alt="Edit" class="img-small" style="filter: brightness(0) invert(0.9);" />{$t["edit.button"]}
  </button>
</div>

<div id="transactions-table-main-container" class="vertical-flex-container">
  <div id="transactions-table">
    {#if selectedTransactionIds.length > 0 || inEditMode}
      <div id="transactions-table-controls" class="vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}>
        <div id="controls-top-bar" class="horizontal-flex-container">
          <p>{$t["transactions-table.controls.header"]}</p>
          {#if inEditMode}
            <p class="opacity-breathing">{$t["transactions-table.controls.notification.header.editmode"]}</p>
          {/if}
          <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => clearEditAndIds()}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0) invert(0.9);" /></button>
        </div>
        <p>{$t["transactions-table-controls.paragraph"][0]} {selectedTransactionIds.length} {$t["transactions-table-controls.paragraph"][1]}</p>
        <div id="controls-buttons" class="horizontal-flex-container">
          <button class="primary-button horizontal-flex-container" onclick={() => !inEditMode ? enterEditMode() : inEditMode = false}>
            <img src="/edit-pen.svg" alt="Edit" />{$t["edit.button"]}
          </button>
          <button class="primary-button horizontal-flex-container" class:disabled={inEditMode} disabled={inEditMode} onclick={() => sendAlert("alert.transactions-table.delete.confirmation", false, true, async () => handleDelete(), () => selectedTransactionIds = [])}>
            <img src="/trash-can.svg" alt="Trash" />{$t["delete.button"]}
          </button>
          {#if inEditMode}
            <button class="primary-button horizontal-flex-container" onclick={() => commitChanges()}><img src="/disk.svg" alt="Save" />{$t["commit.button"]}</button>
          {/if}
        </div>
      </div>
    {/if}

    <div id="transactions-table-headers-container" class="grid-layout">
      <input type="checkbox" class="table-input" style="align-self: center;" class:disabled={$transactions.length <= 0 || inEditMode} checked={$transactions.length > 0 && selectedTransactionIds.length === $transactions.length && !inEditMode}
        disabled={$transactions.length <= 0 || inEditMode} onclick={() => inEditMode ? {} : handleSelectAll()}
      />
      {#each $t["transactions-table.thead.headers"] as header, i (i)}
        <p class="table-header">{header}</p>
      {/each}
    </div>
    <div id="transactions-table-body-outer">
      <div id="transactions-table-body" class="vertical-flex-container">
        {#each displayTransactions as transaction (transaction.id)}
          <div role="menuitem" tabindex="0" class="table-row grid-layout horizontal-flex-container" style="cursor: {inEditMode ? "default" : "pointer"};" onclick={() => inEditMode ? {} : handleSelect(transaction.id)} onkeydown={(e) => { if (e.key === "Enter") inEditMode ? {} : handleSelect(transaction.id)}}>
            <input type="checkbox" class="table-input" checked={selectedTransactionIds.includes(transaction.id) && !inEditMode} class:disabled={inEditMode} disabled={inEditMode} />
            <div class="table-cell">{transaction.id}</div>

            {#if inEditMode}
              <input bind:value={transaction.date} />
              <input type="number" min="0" step="0.01" bind:value={transaction.amount} />
              <select bind:value={transaction.category}>
                {#each combinedCategories as option, i (i)}
                  <option value={option.value}>{($t[option.parent] as Array<Record<string, string>>)[option.index][option.key]}</option>
                {/each}
              </select>
              <input bind:value={transaction.description} />
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

  #transactions-table-headers-container {
    position: sticky;
    top: 0;
    height: 36px;
    margin-bottom: 2px;
    padding: 6px 26px 6px 20px;
    border-bottom: 1px solid #333;
    background-color: #0f0f0f;
  }

  .grid-layout {
    display: grid;
    grid-template-columns: 24px 0.5fr minmax(100px, 120px) 1.2fr 1.5fr 2fr 100px;
  }

  .table-row {
    height: 56px;
    justify-content: unset;
    padding: 6px 10px;
    border-radius: 8px;
    transition: transform 0.2s;
  }

  .table-row:hover {
    cursor: pointer;
    transform: translateY(-2px);
    background-color: #222;
    outline: 1px solid rgba(255, 70, 70, 1);
  }

  .table-header {
    margin: 0;
    padding: 0 10px;
    font-weight: bold;
    user-select: none;
    border-left: 2px solid #555;
  }

  .table-cell {
    font-size: 15px;
    padding: 0 10px;
    pointer-events: none;
  }
  .table-cell span {
    padding: 6px 10px;
    border-radius: 6px;
    user-select: none;
  }

  .table-input {
    width: 14px;
    height: 14px;
    margin: 0;
    transition: transform 100ms;
  }

  .table-input:hover {
    cursor: pointer;
    transform: scale(1.2);
  }

  #transactions-table-controls {
    width: calc(100% - 20px);
    align-items: flex-start;
    margin: 10px;
    gap: 24px;
    padding: 16px;
    border-radius: 12px;
    background-color: #181818;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
    user-select: none;
  }

  #transactions-table-controls p {
    margin: 0;
  }

  #controls-top-bar {
    width: 100%;
    justify-content: space-between;
  }

  #controls-top-bar p {
    font-weight: bold;
  }

  #controls-buttons {
    justify-content: flex-start;
    gap: 16px;
  }

  #controls-buttons button {
    justify-content: flex-start;
    gap: 8px;
    padding: 12px 16px;
  }

  #controls-buttons button img {
    width: 20px;
    height: 20px;
    filter: brightness(0) invert(0.9);
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
</style>