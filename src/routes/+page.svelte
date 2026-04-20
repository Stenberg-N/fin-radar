<script lang="ts">
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { viewStore, setViewState } from "$lib/viewStore";
  import { addTransaction } from "$lib/transactions";
  import { expenseCategories, incomeCategories } from "$lib/transactions";

  import Calendar from "../components/Calendar.svelte";

  type FormKey = "date" | "description" | "amount";

  let selectedCategory = $state<string>('');
  let chosenCategory = $state<string>('');
  let chosenCategoryType = $state<string>('');
  let form = $state<{date: string; description: string; amount: number | null;}>({ date: "", description: "", amount: null });
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $derived($viewStore.isCalendar);

  const addTransactionInputs = [
    { title: "add-transaction.input.date.title", key: "date" },
    { title: "add-transaction.input.description.title", key: "description" },
    { title: "add-transaction.input.amount.title", key: "amount" },
  ];
  const addTransactionCategories = {
    expenses: expenseCategories,
    income: incomeCategories,
  };

  /*
  **********************************************************************************************************************************

  Context, Helper & Wrapper functions
  
  **********************************************************************************************************************************
  */
  const setCalendarIsoDate = (date: string) => {
    form.date = date;
  };

  /* ********************************************************************************************************************************** */

  const handleSubmit = async () => {
    if (!chosenCategory) { sendAlert("alert.add-transaction.no-category", true, false); return; }
    if (!form.date || !form.description || !form.amount) { sendAlert("alert.add-transaction.input-missing", true, false); return; }

    if ($user) {
      const result = await addTransaction($user?.id, chosenCategory, form.date, form.description, form.amount, chosenCategoryType, $user?.name)
      result.success ? (() => {
        sendAlert("alert.add-transaction.success", true, false);
        selectedCategory = '';
        chosenCategory = '';
        chosenCategoryType = '';
        form.date = '';
        form.description = '';
        form.amount = null;
      })() : sendAlert("alert.add-transaction.fail", true, false);
    }
  };

  const handleCategorySelect = (target: EventTarget | null, type: string) => {
    if (!target) return;

    const node = target as HTMLInputElement;
    chosenCategory = node.value;
    chosenCategoryType = type === "expenses" ? "expense" : "income";
  };

  const clearForm = () => {
    sendAlert("alert.add-transaction.cancel.question", false, true, () => { chosenCategory = ''; selectedCategory = ''; chosenCategoryType = ''; form.amount = null; form.date = ''; form.description = ''; });
  };

  const handleKeyDown = (command: string, event: KeyboardEvent) => {
    const allowedKeys = ["Backspace", "Delete", "ArrowLeft", "ArrowRight", "Tab", "Home", "End"];
    const regex = /^[0-9\-]+$/g;

    switch (command) {
      case "amount": {
        if (event.key === ",") {
          event.preventDefault();
          sendAlert("alert.add-transaction.amount.comma", true, false);
        }
        if (event.key === "-") {
          event.preventDefault();
          sendAlert("alert.add-transaction.amount.minus", true, false);
        }
        break;
      }
      case "date": {
        if (allowedKeys.includes(event.key)) return;

        if (!regex.test(event.key)) {
          event.preventDefault();
          sendAlert("alert.add-transaction.date.input", true, false);
        }
        break;
      }
    }
  };

  const handleAmountInput = (target: EventTarget | null) => {
    if (!target) return;

    const node = target as HTMLInputElement;
    const value = Number(node.value);
    if (value < 0) node.value = "0";
  };

  const handleAmount = (command: string) => {
    let value = Number(form.amount);

    switch (command) {
      case "increase": form.amount = (Math.round((value += 0.01) * 100) / 100); break;
      case "decrease": if (value > 0) form.amount = (Math.round((value -= 0.01) * 100) / 100); break;
    }
  };
</script>

<div id="home-main-container" class="horizontal-flex-container">
  <div id="add-transaction-container" class="form-outer-container">
    {#if isCalendar}
      <Calendar setCalendarIsoDate={setCalendarIsoDate} {calendarToggle} />
    {/if}

    <h2>{$t["add-transaction-title"]}</h2>
    <form id="add-transaction-form" class="form-bg" style="background-color: #181818; margin-top: 16px; box-shadow: none;" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <h3 style="margin: 0;">{$t["add-transaction.categories.title"]}</h3>
      <div id="categories" class="horizontal-flex-container">
        {#each Object.entries(addTransactionCategories) as [type, options]}
          <p class="form-p" style="width: 100%;">{$t[type === "expenses" ? "add-transaction.categories.sub-title.expenses" : "add-transaction.categories.sub-title.income"]}</p>
          {#each options as option, i}
            <label class="primary-button category-option" class:isChecked={selectedCategory === option.value}>
              <input type="radio" value={option.value} onclick={(e) => { handleCategorySelect(e.target, type); }} bind:group={selectedCategory} />
              <span>{($t[option.parent][i] as Record<string, string>)[option.key]}</span>
            </label>
          {/each}
        {/each}
      </div>
      {#each addTransactionInputs as input, i (i)}
        <div style="vertical-flex-container">
          <p class="form-p">{$t[input.title]}</p>
          <div class="form-input-container" style="position: relative; justify-content: flex-end;">
            <input type={input.key === "amount" ? "number" : "text"} class="form-input" style={i === 0 ? "padding-right: 44px" : (i === 2 ? "padding-right: 86px" : "")} placeholder={i === 0 ? "31-12-2025" as string : (i === 1 ? $t[input.title] as string : "20.60")} bind:value={form[input.key as FormKey]}
              {...(input.key === "amount" ? { min: 0, step: 0.01, onkeydown: (e) => handleKeyDown("amount", e), oninput: (e) => handleAmountInput(e.target) } : (input.key === "date" ? { onkeydown: (e) => handleKeyDown("date", e)} : {}) )} required title=""
            />
            {#if i === 0}
              <button id="calendar-toggle" class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => setViewState("isCalendar", undefined, true)}><img src="/calendar.svg" alt="Calendar" class="img-large" style="filter: invert(0.9);" /></button>
            {:else if i === 2}
              <div id="add-transaction-amount-steppers-container" class="horizontal-flex-container" style="position: absolute; gap: 10px; margin-right: 6px;">
                <button class="primary-button vertical-flex-container" type="button" onclick={() => handleAmount("increase")}><img src="/arrow.svg" alt="Increase" class="img-small" style="transform: rotate(180deg);" /></button>
                <button class="primary-button vertical-flex-container" type="button" onclick={() => handleAmount("decrease")}><img src="/arrow.svg" alt="Decrease" class="img-small" /></button>
              </div>
            {/if}
          </div>
        </div>
      {/each}
      <div id="add-transaction-buttons" class="horizontal-flex-container">
        <button type="button" class="primary-button" onclick={() => clearForm()}>{$t["clear.button"]}</button>
        <button type="submit" class="primary-button">{$t["add.button"]}</button>
      </div>
    </form>
  </div>
</div>

<style>
  #home-main-container {
    width: 100%;
    height: 100%;
    align-items: unset;
    justify-content: flex-start;
    padding: 20px;
  }

  #add-transaction-container {
    max-width: 40%;
    min-width: 595px;
    background-color: #181818;
    color: #f6f6f6;
    padding: 16px 32px 32px;
  }

  #add-transaction-container h2 {
    width: 100%;
    margin: 0;
    padding-bottom: 16px;
    text-align: center;
    border-bottom: 1px solid #333;
  }

  #add-transaction-form {
    overflow-y: auto;
    scrollbar-gutter: stable;
    padding: 24px 26px 24px 32px;
  }

  .form-input {
    outline: 2px solid #333;
    color: #f6f6f6;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }
  .form-input:focus {
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
  }

  #categories {
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 10px;
  }

  .category-option {
    display: flex;
    justify-content: center;
    width: calc(33% - 5.1px);
    min-width: 134px;
  }

  .category-option input {
    display: none;
  }

  .category-option span {
    pointer-events: none;
    justify-self: center;
  }

  .category-option:not(.isChecked):hover {
    outline-color: rgba(255, 70, 70, 1);
  }

  .isChecked {
    font-weight: bold;
  }

  .isChecked, .isChecked:hover {
    outline-color: transparent;
    background-color: rgba(255, 70, 70, 1);
  }

  #add-transaction-amount-steppers-container button img {
    filter: brightness(0) invert(0.9);
  }

  #calendar-toggle {
    position: absolute;
    border-radius: 6px;
    padding: 6px;
    transition: transform 0.2s;
  }

  #calendar-toggle:hover {
    transform: translateY(-4px);
  }
</style>