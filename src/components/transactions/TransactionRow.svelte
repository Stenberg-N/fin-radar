<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";

  import { capitalizeString, handleKeyDownOnInput, handleNumberInput } from "$lib/utils/helpers.svelte";
  import { transactionCategoryTags } from "$lib/transactions";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { userPrefs } from "$lib/utils/prefsStore";
  import { type Transaction } from "$lib/types";

  interface Props {
    transaction: Transaction,
    /** Used to pass the set of selected transactions IDs. Syncs the state of checkboxes. */
    selectedTransactionIds: SvelteSet<number>,
    /** Used to return the transaction ID to update the selectedTransactionIds in the parent. */
    handleSelect: (id: number) => void;
    /** Used to return the transaction ID and the updated field. */
    onChange: (id: number, patch: Partial<Transaction>) => void;
    inEditMode: boolean,
    CONTAINER_WIDTH: number,
    onMouseLeave: (e: MouseEvent) => void;
    /** Used to return the transaction field's value for hover title. */
    onMouseEnter: (str: string) => void,
  }

  let {
    transaction,
    selectedTransactionIds,
    handleSelect,
    onChange,
    inEditMode,
    CONTAINER_WIDTH,
    onMouseLeave,
    onMouseEnter,
  }: Props = $props();

  const categoryOptions = $derived(
    transactionCategoryTags.map(cat => ({
      value: cat,
      label: (i18n.t["add-transaction.categories"] as Record<string, string>)[cat]
    }))
  );

  const categoryLabel = $derived.by(() => {
    const item = transactionCategoryTags.find(t => t === transaction.category);
    return item
      ? (i18n.t["add-transaction.categories"] as Record<string, string>)[item]
      : 'Unknown';
  });

  const amountLabel = $derived(
    (transaction._type === "income" ? "" : "-") +
    transaction.amount +
    $userPrefs.mainPrefs.currency
  );

  const step = (delta: number) => {
    onChange(transaction.id, { amount: Math.max(0, Math.round((transaction.amount + delta) * 100) / 100) });
  };
</script>

<div role="menuitem" tabindex="0" class="table-row gridbox" style="cursor: {inEditMode ? "default" : "pointer"};"
  onclick={() => inEditMode ? {} : handleSelect(transaction.id)}
  onkeydown={(e) => { if (e.key === "Enter") inEditMode ? {} : handleSelect(transaction.id)}}
>
  <input type="checkbox" class="table-checkbox" checked={selectedTransactionIds.has(transaction.id) && !inEditMode} disabled={inEditMode} />
  <p class="table-cell">
    <span role="cell" tabindex="0"
      onmouseenter={() => onMouseEnter(String(transaction.id))}
      onmouseleave={(e) => onMouseLeave(e)}
    >
      {transaction.id}
    </span>
  </p>

  {#if inEditMode}
    <div class="table-cell-edit">
      <input
        class="primary-input"
        value={transaction.date}
        oninput={(e) => onChange(transaction.id, { date: e.currentTarget.value })}
        onkeydown={(e) => handleKeyDownOnInput("date", e)}
      />
    </div>
    <div class="table-cell-edit amount-container" class:smaller={CONTAINER_WIDTH <= 810}>
      <input
        class="primary-input"
        type="number"
        min="0"
        step="0.01"
        value={transaction.amount}
        oninput={(e) => {
          const value = e.currentTarget.valueAsNumber;
          handleNumberInput(e.currentTarget);
          onChange(transaction.id, { amount: Number.isNaN(value) ? 0 : value });
        }}
        onkeydown={(e) => handleKeyDownOnInput("amount", e)} />
      <div class="amount-steppers-container flex row">
        {#each Array.from({length: 2}, (_, i) => i) as btn (btn)}
          <button
            aria-label={`${btn === 0 ? 'increase' : 'decrease'} amount`}
            class="button-primary transparent highlight default-corners"
            type="button"
            onclick={() => step(btn === 0 ? 0.01 : -0.01)}
          >
            <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); {btn === 0 && 'transform: rotate(180deg);'}"></span>
          </button>
        {/each}
      </div>
    </div>
    <div class="table-cell-edit">
      <select
        class="primary-input"
        value={transaction.category}
        onchange={(e) => {
          const category = e.currentTarget.value;
          onChange(transaction.id, { category, _type: ["salary", "freelance", "investments"].includes(category) ? "income" : "expense" })
        }}
      >
        {#each categoryOptions as option (option.value)}
          <option value={option.value}>
            {option.label}
          </option>
        {/each}
      </select>
    </div>
    <div class="table-cell-edit">
      <input
        class="primary-input"
        value={transaction.description}
        oninput={(e) => onChange(transaction.id, { description: e.currentTarget.value })}
      />
    </div>
  {:else}
    {#snippet cell(text: string)}
      <p class="table-cell">
      <span role="cell" tabindex="0"
        onmouseenter={() => onMouseEnter(capitalizeString(text))}
        onmouseleave={(e) => onMouseLeave(e)}
      >
        {text}
      </span>
    </p>
    {/snippet}
    {@render cell(transaction.date)}
    {@render cell(amountLabel)}
    {@render cell(categoryLabel)}
    {@render cell(transaction.description)}
  {/if}

  <div class="table-cell type-cell flex"
    style:--type-cell-bg={transaction._type === "expense"? "var(--color-negative-dimmed)" : "var(--color-positive-dimmed)"}
    style:--type-cell-outline={transaction._type === "expense" ? "var(--color-negative)" : "var(--color-positive)"}
  >
    <span class="span-icon img-small-medium" style="mask-image: url('/arrow-up.svg'); transform: rotate({transaction._type === "expense" ? '180' : ''}deg);"></span>
  </div>
</div>

<style>
  .table-checkbox {
    align-self: center;
    width: 14px;
    height: 14px;
    margin: 0.25rem 1rem;
  }

  .gridbox {
    display: grid;
    grid-template-columns: 46px minmax(40px, 70px) minmax(90px, 120px) minmax(128px, 1fr) minmax(136px, 1fr) minmax(96px, 1fr) 48px;
  }

  .table-row {
    height: 56px;
    border-radius: 4px;
    transition: transform 0.2s;
    contain-intrinsic-size: 56px;

    &:focus, &:focus:hover {
      background-color: var(--color-secondary2);
    }

    &:hover {
      cursor: pointer;
      background-color: var(--color-secondary1);
    }

    .primary-input {
      background-color: var(--color-primary2);
      font-size: clamp(0.75rem, 1.1cqw, 1rem);
    }

    .table-cell, .table-cell-edit {
      align-content: center;
      height: 100%;
      padding: 0.5rem;
      margin: 0;
      font-size: clamp(0.75rem, 1.1cqw, 1rem);
      text-align: left;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .table-cell.type-cell {
      position: relative;
      padding: 0.5rem;
      user-select: none;
      
      &::before {
        content: '';
        position: absolute;
        height: 32px;
        width: 32px;
        border-radius: 0.25rem;
        background-color: var(--type-cell-bg);
        outline: 1px solid var(--type-cell-outline);
      }
    }

    .table-cell-edit.amount-container {
      position: relative;

      button.button-primary.transparent.highlight {
        height: fit-content;
        padding: 0.25rem;
      }

      .amount-steppers-container {
        position: absolute;
        right: 0;
        top: 50%;
        transform: translateY(-50%);
        gap: 0.25rem;
        margin-right: 1rem;
      }

      &.smaller .amount-steppers-container {
        top: unset;
        bottom: 0;

        button.button-primary.transparent.highlight {
          padding: 2px;
          
          span {
            height: 10px;
            width: 10px;
          }
        }
      }
    }
  }
</style>