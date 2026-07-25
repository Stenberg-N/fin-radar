<script lang="ts">
  import { t, lang } from "$lib/i18n";
  import { calendarDate } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";

  import Calendar from "../Calendar.svelte";
  import { handleKeyDownOnInput } from "$lib/actions";

  type FormKey = "date" | "title" | "description";

  let {
    closeForm,
    navButtonRefs,
  }: {
    closeForm: () => void;
    navButtonRefs: HTMLButtonElement[];
  } = $props();

  let form = $state<{ date: string, title: string, description: string }>({ date: '', title: '', description: '' });
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $state<boolean>(false);
  const formInputs = [
    { title: "date-input.description", key: "date" },
    { title: "title-input.description", key: "title" },
    { title: "description-input.description", key: "description" }
  ];

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let dateInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (formInputRefs[0]) dateInput = formInputRefs[0];
  });

  const handleSubmit = () => {};

  const clearForm = () => {
    sendAlert({
      message: "alert.clear-form.question",
      isTimer: false,
      buttons: true,
      onConfirm: () => { form.date = '', form.description = '', form.title = '' }
    });
  };
</script>

<div id="add-calendar-event-form-container" class="form-outer-container">
  {#if isCalendar}
    <Calendar {calendarToggle} calendarStartDate={$calendarDate} ignorableEls={[dateInput, ...navButtonRefs]} isMonthChangeEnabled={false}
      setCalendarIsoDate={(date) => form.date = date}
      setCalendarVisibility={(state) => isCalendar = state}
    />
  {/if}

  <div id="add-calendar-event-title-container" class="horizontal-flex-container">
    <h2>{$t["calendar.add-event.header"]}</h2>
    <button type="button" class="transparent-button-highlight" onclick={() => closeForm()}>
      <img src="/close-x.svg" alt="Close" class="img-small" />
    </button>
  </div>

  <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    {#each formInputs as input, i (i)}
      <div>
        <p>{$t[input.title]}</p>
        <div id="calendar-event-input-container" class="form-input-container horizontal-flex-container">
          {#if [0, 1].includes(i)}
            <input class="primary-input" type="text" style="{i === 0 ? 'padding-right: 44px' : ''};" placeholder={$t[i === 0 ? "placeholder.isodate" : "title-input.description"] as string} 
              bind:value={form[input.key as FormKey]}
              bind:this={formInputRefs[i]}
              onkeydown={(e) => { if (i === 0) handleKeyDownOnInput("date", e) }}
            />
          {:else}
            <textarea placeholder={$lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}></textarea>
          {/if}
          {#if i === 0}
            <button id="calendar-toggle" class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
              <img src="calendar.svg" alt="Calendar" class="img-large" />
            </button>
          {/if}
        </div>
      </div>
    {/each}

    <div id="calendar-event-form-buttons" class="horizontal-flex-container">
      <button type="submit" class="primary-button horizontal-flex-container">
        <img src="plus.svg" alt="Plus" class="img-small" />
        {$t["add.button"]}
      </button>
      <button type="button" class="primary-button horizontal-flex-container" onclick={() => clearForm()}>
        <img src="trash-can.svg" alt="Trash can" class="img-small" />
        {$t["clear.button"]}
      </button>
    </div>
  </form>
</div>

<style>
  #add-calendar-event-form-container {
    box-shadow: none;
    background-color: #181818;
    min-height: 0;
    height: 100%;
    max-width: 500px;
    padding: 16px 32px 32px;

    #add-calendar-event-title-container {
      position: relative;
      padding-bottom: 16px;
      border-bottom: 1px solid #333;

      h2 {
        margin: 0;
        color: #f6f6f6;
      }

      button {
        position: absolute;
        right: 0;
        height: 32px;
        width: 32px;
      }
    }

    .form-bg {
      flex: 1;
      padding: 16px;
      overflow-y: auto;
      overflow-x: hidden;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      p {
        margin: 0;
        font-weight: bold;
        color: #f6f6f6;
        user-select: none;
      }
    }

    #calendar-event-input-container {
      position: relative;
      justify-content: flex-end;
      height: fit-content;

      button {
        position: absolute;
        border-radius: 6px;
        padding: 6px;
        transform: transition 0.2s;
      }
      button:hover {
        transition: scale(1.1);
      }

      .primary-input {
        min-height: 52px;
        flex-shrink: 0;
        outline: 2px solid #333;
        color: #f6f6f6;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
      }
      .primary-input:focus, textarea:focus {
        outline-color: rgba(255, 70, 70, 1);
      }

      textarea {
        min-height: 120px;
        min-width: 320px;
        width: 100%;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        padding: 6px;
        border: none;
        outline: 2px solid #333;
        border-radius: 4px;
        background-color: transparent;
        color: #f6f6f6;
        font-size: 16px;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
      }
    }

    #calendar-event-form-buttons {
      justify-content: flex-start;
      gap: 12px;

      button {
        padding: 12px 24px;
        gap: 8px;
        font-size: 18px;
      }
    }
  }
</style>