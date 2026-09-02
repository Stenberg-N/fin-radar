<script lang="ts">
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n/i18n";
  import { addTransaction } from "$lib/transactions";
  import { expenseCategories, incomeCategories } from "$lib/transactions";
  import { handleKeyDownOnInput, handleNumberInput, handleClickOutside } from "$lib/actions";

  import Calendar from "../components/Calendar.svelte";
  import ModalWrapper from "./ModalWrapper.svelte";

  type FormKey = "date" | "description" | "amount";

  let {
    closeForm,
    calendarStartDate,
    ignorableEls,
  }: {
    closeForm?: () => void;
    calendarStartDate?: Date;
    ignorableEls?: (HTMLElement | null)[];
  } = $props();

  let selectedCategory = $state<string>('');
  let chosenCategory = $state<string>('');
  let chosenCategoryType = $state<string>('');
  let form = $state<{date: string; description: string; amount: number | null;}>({ date: "", description: "", amount: null });
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $state<boolean>(false);

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let dateInput = $state<HTMLInputElement | null>(null);

  const addTransactionInputs = [
    { title: "date-input.description", key: "date" },
    { title: "description-input.description", key: "description" },
    { title: "add-transaction.input.amount.title", key: "amount" },
  ];
  const addTransactionCategories = {
    expenses: expenseCategories,
    income: incomeCategories,
  };

  $effect(() => {
    if (formInputRefs[0]) dateInput = formInputRefs[0];
  });

  const handleSubmit = async () => {
    if (!chosenCategory) { sendAlert({ message: "alert.add-transaction.no-category", isTimer: true, buttons: false }); return; }
    if (!form.date || !form.description || !form.amount) { sendAlert({ message: "alert.add-transaction.input-missing", isTimer: true, buttons: false }); return; }
    if (form.amount <= 0) { sendAlert({ message: "alert.input-missing", isTimer: true, buttons: false }); return; }

    const result = await addTransaction(chosenCategory, form.date, form.description, form.amount, chosenCategoryType)
    result.success ? (() => {
      sendAlert({ message: "alert.add-transaction.success", isTimer: true, buttons: false });
      selectedCategory = '';
      chosenCategory = '';
      chosenCategoryType = '';
      form.date = '';
      form.description = '';
      form.amount = null;
    })() : sendAlert({ message: "alert.add-transaction.fail", isTimer: true, buttons: false });
  };

  const handleCategorySelect = (target: EventTarget | null, type: string) => {
    if (!target) return;

    const node = target as HTMLInputElement;
    chosenCategory = node.value;
    chosenCategoryType = type === "expenses" ? "expense" : "income";
  };

  const clearForm = () => {
    sendAlert({
      message: "alert.clear-form.question",
      isTimer: false,
      buttons: true,
      onConfirm: () => { chosenCategory = ''; selectedCategory = ''; chosenCategoryType = ''; form.amount = null; form.date = ''; form.description = ''; }
    });
  };

  const handleNumberStepper = (command: string) => {
    let value = Number(form.amount);

    switch (command) {
      case "increase": form.amount = (Math.round((value += 0.01) * 100) / 100); break;
      case "decrease": if (value > 0) form.amount = (Math.round((value -= 0.01) * 100) / 100); break;
    }
  };
</script>

<div id="add-transaction-container" class="form-outer-container" use:handleClickOutside={{ onOutsideClick: () => closeForm ? closeForm() : {}, additionalElements: ignorableEls }}>
  {#if isCalendar}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <Calendar options={{
        calendarToggle,
        calendarStartDate,
        ignorableEls: [dateInput],
        setCalendarIsoDate: (date) => form.date = date,
        setCalendarVisibility: (state) => isCalendar = state,
      }}
      />
    </ModalWrapper>
  {/if}

  <div id="add-transaction-title-container" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["add-transaction-title"]}</h2>
    {#if closeForm}
      <button type="button" class="transparent-button-highlight" onclick={() => closeForm()}><img src="/close-x.svg" alt="Close" class="img-small" /></button>
    {/if}
  </div>

  <form id="add-transaction-form" class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div id="categories" class="vertical-flex-container">
      {#each Object.entries(addTransactionCategories) as [type, options], i (i)}
        <p class="form-p" style="width: 100%;">{$t[type === "expenses" ? "expenses.header" : "income.header"]}</p>
        <div class="category-options-container">
          {#each options as option, i (i)}
            <label class="primary-button-light category-option" class:isChecked={selectedCategory === option.value}>
              <input type="radio" value={option.value} onclick={(e) => { handleCategorySelect(e.target, type); }} bind:group={selectedCategory} />
              <span>{($t[option.parent][i] as Record<string, string>)[option.key]}</span>
            </label>
          {/each}
        </div>
      {/each}
    </div>
    {#each addTransactionInputs as input, i (i)}
      <div style="vertical-flex-container">
        <p class="form-p">{$t[input.title]}</p>
        <div class="form-input-container" style="position: relative; justify-content: flex-end;">
          <input type={input.key === "amount" ? "number" : "text"} class="primary-input" style={i === 0 ? "padding-right: 44px" : (i === 2 ? "padding-right: 86px" : "")}
            placeholder={i === 0 ? $t["placeholder.isodate"] as string : (i === 1 ? $t[input.title] as string : "20.60")}
            title=""
            bind:value={form[input.key as FormKey]}
            bind:this={formInputRefs[i]}
            {...(input.key === "amount"
              ? { min: 0, step: 0.01, onkeydown: (e) => handleKeyDownOnInput("amount", e), oninput: (e) => handleNumberInput(e.target) }
              : (input.key === "date" ? { onkeydown: (e) => handleKeyDownOnInput("date", e), onclick: () => isCalendar = true } : {}))
            }
            required
          />
          {#if i === 0}
            <button id="calendar-toggle" class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}><img src="/calendar.svg" alt="Calendar" class="img-large" /></button>
          {:else if i === 2}
            <div id="add-transaction-amount-steppers-container" class="horizontal-flex-container" style="position: absolute; gap: 10px; margin-right: 6px;">
              <button class="primary-button-light" type="button" onclick={() => handleNumberStepper("increase")}><img src="/arrow.svg" alt="Increase" class="img-small" style="transform: rotate(180deg);" /></button>
              <button class="primary-button-light" type="button" onclick={() => handleNumberStepper("decrease")}><img src="/arrow.svg" alt="Decrease" class="img-small" /></button>
            </div>
          {/if}
        </div>
      </div>
    {/each}
    <div id="add-transaction-buttons" class="horizontal-flex-container">
      <button type="button" class="primary-button-light" onclick={() => clearForm()}><img src="/trash-can.svg" alt="Trash can" class="img-small" />{$t["clear.button"]}</button>
      <button type="submit" class="primary-button-light"><img src="/plus.svg" alt="Plus" class="img-small" />{$t["add.button"]}</button>
    </div>
  </form>
</div>

<style>
  #add-transaction-container {
    width: 100%;
    max-width: 500px;
    min-height: 0;
    height: 100%;
    background-color: #222;
    color: #f6f6f6;
    padding: 16px 32px 32px;
    box-shadow: none;
  }

  #add-transaction-title-container {
    position: relative;
    width: 100%;
    padding-bottom: 16px;
    text-align: center;
    border-bottom: 2px solid #333;

    button {
      position: absolute;
      right: 0;
      height: 32px;
      width: 32px;
    }
  }

  #add-transaction-form {
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-gutter: stable both-edges;
    padding: 16px;
    background-color: transparent;
    box-shadow: none;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
  }

  .primary-input {
    outline: 2px solid #333;
    font-size: unset;
  }
  .primary-input:focus {
    outline-color: rgba(255, 70, 70, 1);
  }

  #add-transaction-buttons {
    justify-content: flex-start;
    gap: 20px;
    margin-top: auto;
  }

  #add-transaction-buttons button {
    font-size: 18px;
    padding: 12px 24px;
    gap: 8px;
  }

  #categories {
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 10px;
    text-align: center;
  }

  .category-options-container {
    display: grid;
    width: 100%;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }

  .category-option {
    text-align: center;
    min-height: 42px;
    box-shadow: none;
  }

  .category-option input {
    display: none;
  }

  .category-option span {
    pointer-events: none;
    text-align: center;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  .category-option.isChecked, .category-option.isChecked:hover {
    background-color: rgba(255, 70, 70, 1);
  }

  #calendar-toggle {
    position: absolute;
    border-radius: 6px;
    padding: 6px;
    transition: transform 0.2s;
  }

  #calendar-toggle:hover {
    transform: scale(1.1);
  }
</style>