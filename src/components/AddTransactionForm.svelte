<script lang="ts">
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n/i18n";
  import { addTransaction, transactionCategoryTags } from "$lib/transactions";
  import { handleKeyDownOnInput, handleNumberInput, handleClickOutside } from "$lib/actions";

  import Calendar from "../components/Calendar.svelte";
  import ModalWrapper from "./ModalWrapper.svelte";

  type FormKey = "date" | "description" | "amount";

  let {
    options,
  }: {
    options?: {
      closeForm?: () => void;
      calendarStartDate?: Date;
      ignorableEls?: (HTMLElement | null)[];
      isBgTransparent?: boolean;
    },
  } = $props();

  const closeForm = $derived(options?.closeForm);
  const calendarStartDate = $derived(options?.calendarStartDate);
  const ignorableEls = $derived(options?.ignorableEls);
  const isBgTransparent = $derived(options?.isBgTransparent ?? false);

  let form = $state<{date: string; description: string; amount: number | null;}>({ date: "", description: "", amount: null });
  let selectedCategory = $state<string>('');
  let chosenCategory = $state<string>('');
  let chosenCategoryType = $state<string>('');
  let isCalendar = $state<boolean>(false);

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let dateInput = $state<HTMLInputElement | null>(null);
  let calendarToggle = $state<HTMLButtonElement | null>(null);

  const addTransactionInputs = [
    { title: "date-input.description", key: "date" },
    { title: "description-input.description", key: "description" },
    { title: "add-transaction.input.amount.title", key: "amount" },
  ];
  const addTransactionCategories = {
    expenses: transactionCategoryTags.slice(0, 12),
    income: transactionCategoryTags.slice(12, 15),
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

<div id="add-transaction-container" class="form-outer-container" style="background-color: {isBgTransparent ? 'transparent' : 'var(--color-secondary1)'};"
  use:handleClickOutside={{ onOutsideClick: () => closeForm ? closeForm() : {}, additionalElements: ignorableEls }}
>
  {#if isCalendar}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <Calendar options={{
        calendarToggle,
        calendarStartDate,
        ignorableEls: [dateInput],
        setCalendarIsoDate: (date) => { form.date = date; },
        setCalendarVisibility: (state) => { isCalendar = state; },
      }}
      />
    </ModalWrapper>
  {/if}

  <div id="add-transaction-title-container" class="flex row">
    <h2 style="margin: 0;">{$t["add-transaction-title"]}</h2>
    {#if closeForm}
      <button aria-label="Close form" type="button" class="button-primary transparent highlight static" onclick={() => closeForm()}>
        <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
      </button>
    {/if}
  </div>

  <form id="add-transaction-form" class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div id="categories" class="flex column">
      {#each Object.entries(addTransactionCategories) as [type, options], i (i)}
        <p class="form-p" style="width: 100%;">{$t[type === "expenses" ? "expenses.header" : "income.header"]}</p>
        <div class="category-options-container">
          {#each options as option, i (i)}
            <label class="button-primary transparent highlight outline default-corners category-option" class:isChecked={selectedCategory === option}>
              <input type="radio" value={option} onclick={(e) => { handleCategorySelect(e.target, type); }} bind:group={selectedCategory} />
              <span>{($t["add-transaction.categories"] as Record<string, string>)[option]}</span>
            </label>
          {/each}
        </div>
      {/each}
    </div>
    {#each addTransactionInputs as input, i (i)}
      <div style="flex column">
        <p class="form-p">{$t[input.title]}</p>
        <div class="form-input-container" style="position: relative; justify-content: flex-end;">
          <input type={input.key === "amount" ? "number" : "text"} class="primary-input" style={i === 0 ? "padding-right: 40px" : (i === 2 ? "padding-right: 86px" : "")}
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
            <button aria-label="Toggle calendar" id="calendar-toggle" class="button-primary transparent" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
              <span class="span-icon img-medium-large" style="mask-image: url('/calendar.svg');"></span>
            </button>
          {:else if i === 2}
            <div id="add-transaction-amount-steppers-container" class="flex row" style="position: absolute; gap: 10px; margin-right: 6px;">
              {#each Array.from({length: 2}, (_, i) => i) as b}
                <button aria-label="{b === 0 ? "Increase" : "Decrese"} amount" class="button-primary transparent highlight default-corners" type="button" onclick={() => handleNumberStepper(b === 0 ? "increase" : "decrease")}>
                  <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transform: rotate({b === 0 ? '180deg' : ''});"></span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/each}
    <div id="add-transaction-buttons" class="flex row">
      <button type="button" class="button-primary light" onclick={() => clearForm()}>
        <span class="span-icon img-small" style="mask-image: url('/trash-can.svg');"></span>
        {$t["clear.button"]}
      </button>
      <button type="submit" class="button-primary white-bg">
        <span class="span-icon img-small" style="mask-image: url('/plus.svg');"></span>
        {$t["add.button"]}
      </button>
    </div>
  </form>
</div>

<style>
  #add-transaction-container {
    width: 100%;
    max-width: 500px;
    min-height: 0;
    height: 100%;
    padding: 1rem 2rem 2rem;
    box-shadow: none;

    #add-transaction-title-container {
      width: 100%;
      justify-content: space-between;
      padding-bottom: 1rem;
      text-align: center;
      border-bottom: 2px solid var(--outline-color1);
    }

    #add-transaction-amount-steppers-container button {
      height: unset;
      padding: 0.25rem;
      box-shadow: none;
    }

    #add-transaction-form {
      overflow-y: auto;
      overflow-x: hidden;
      scrollbar-gutter: stable both-edges;
      padding: 1rem;
      background-color: transparent;
      box-shadow: none;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
    }

    .form-input-container {
      height: 40px;
    }

    #add-transaction-buttons {
      justify-content: flex-start;
      gap: 20px;
      margin-top: auto;

      button {
        height: unset;
        padding: 0.5rem 1rem;
      }
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
      min-height: 36px;
      box-shadow: none;

      &.isChecked, &.isChecked:not(:disabled):hover {
        background-color: var(--color-highlight1);
        outline: none;
      }

      span {
        pointer-events: none;
        text-align: center;
        font-size: clamp(0.75rem, 0.9cqw, 1rem);
      }

      input {
        display: none;
      }
    }

    #calendar-toggle {
      position: absolute;
      right: 6px;
      height: unset;
      transition: transform 0.2s;
    }

    #calendar-toggle:hover {
      transform: scale(1.1);
    }
  }
</style>