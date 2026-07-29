<script lang="ts">
  import { t, lang } from "$lib/i18n";
  import { calendarDate, addCalendarEvent } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { handleKeyDownOnInput } from "$lib/actions";
  import type { CalendarEventForm } from "$lib/types";

  import Calendar from "../Calendar.svelte";

  type FormKey = "isodate" | "title" | "description";
  type timeKey = "startTimeHours" | "startTimeMinutes" | "endTimeHours" | "endTimeMinutes";

  let {
    closeForm,
    navButtonRefs,
  }: {
    closeForm: () => void;
    navButtonRefs: HTMLButtonElement[];
  } = $props();

  let form = $state<CalendarEventForm>({ isodate: '', title: '', description: null, startTimeHours: null, startTimeMinutes: null, endTimeHours: null, endTimeMinutes: null });
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $state<boolean>(false);
  const textInputs = [
    { title: "date-input.description", key: "isodate" },
    { title: "title-input.description", key: "title" },
  ];
  const otherInputs = [
    { title: "calendar.start-time.description", keys: ["startTimeHours", "startTimeMinutes"] },
    { title: "calendar.end-time.description", keys: ["endTimeHours", "endTimeMinutes"] },
  ];

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let dateInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (formInputRefs[0]) dateInput = formInputRefs[0];
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const clearForm = () => { form.isodate = '', form.title = '', form.description = null, form.startTimeHours = null, form.startTimeMinutes = null, form.endTimeHours = null, form.endTimeMinutes = null };
  const handleTimeInput = (target: EventTarget | null, e: KeyboardEvent) => {
    if (!target) return;
    if (e.key === 'Backspace' || e.key === 'Control' || (e.ctrlKey && (e.key.toLowerCase() === 'a' || e.key.toLowerCase() === 'z'))) return;

    const input = target as HTMLInputElement;
    if (input.value.length >= 2) {
      const nextInput = input.parentElement?.lastChild as HTMLInputElement;
      if (nextInput) nextInput.focus();
    }
  };
  
  /***********************************************************************************************************************************/

  const handleSubmit = async () => {
    const result = await addCalendarEvent(form);
    if (result.success) clearForm();
  };

  const resetForm = () => {
    sendAlert({
      message: "alert.clear-form.question",
      isTimer: false,
      buttons: true,
      onConfirm: () => clearForm()
    });
  };

</script>

<div id="add-calendar-event-form-container" class="form-outer-container">
  {#if isCalendar}
    <Calendar {calendarToggle} calendarStartDate={$calendarDate} ignorableEls={[dateInput, ...navButtonRefs]} isMonthChangeEnabled={false}
      setCalendarIsoDate={(date) => form.isodate = date}
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
    <div class="horizontal-flex-container" style="gap: 40px;">
      <div class="vertical-flex-container">
        <p>{$t["calendar.time-frame.description"]}</p>
        <div class="time-input-outer-container horizontal-flex-container">
          {#each otherInputs as input, i (i)}
            <div class="vertical-flex-container">
              <p>{$t[input.title]}</p>
              <div class="time-container horizontal-flex-container">
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[0] as timeKey]} onkeydown={(e) => handleTimeInput(e.target, e)} />
                <span>:</span>
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[1] as timeKey]} />
              </div>
            </div>
            {#if i === 0}
              <img src="arrow.svg" alt="Arrow" class="img-medium" style="transform: rotate(-90deg); align-self: flex-end; margin: 0 8px 16px;" />
            {/if}
          {/each}
        </div>
      </div>

      <div id="date-title-container" class="vertical-flex-container">
        {#each textInputs as input, i (i)}
          <div class="vertical-flex-container">
            <p>{$t[input.title]}</p>
            <div id="calendar-event-input-container" class="form-input-container">
              <input class="primary-input" type="text" style="{i === 0 ? 'padding-right: 44px' : ''};" placeholder={$t[i === 0 ? "placeholder.isodate" : "title-input.description"] as string}
                bind:value={form[input.key as FormKey]}
                bind:this={formInputRefs[i]}
                onkeydown={(e) => { if (i === 0) handleKeyDownOnInput("date", e) }}
                required
              />
              {#if i === 0}
                <button class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
                  <img src="calendar.svg" alt="Calendar" class="img-large" />
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="vertical-flex-container" style="align-items: flex-start;">
      <p>{$t["description-input.description"]}</p>
      <textarea placeholder={$lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}></textarea>
    </div>

    <div id="calendar-event-form-buttons" class="horizontal-flex-container">
      <button type="submit" class="primary-button horizontal-flex-container">
        <img src="plus.svg" alt="Plus" class="img-small" />
        {$t["add.button"]}
      </button>
      <button type="button" class="primary-button horizontal-flex-container" onclick={() => resetForm()}>
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
    max-width: 650px;
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
      gap: 32px;
      overflow-y: auto;
      overflow-x: hidden;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      > div, #date-title-container {
        align-items: unset;

        > div > p {
          align-self: flex-start;
        }
      }

      #date-title-container {
        gap: 16px;

        #calendar-event-input-container {
          position: relative;
          justify-content: flex-end;
          padding: 0;
          width: 100%;

          button {
            position: absolute;
            height: 100%;
            padding: 6px;
            transition: transform 0.2s;
          }
          button:hover {
            transform: scale(1.1);
          }

          .primary-input {
            outline: 2px solid #333;
            color: #f6f6f6;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
          }
          .primary-input:focus {
            outline-color: rgba(255, 70, 70, 1);
          }
        }
      }

      div.time-input-outer-container {
        align-self: flex-end;
        align-items: flex-end;
        height: 100%;
        padding: 0 16px 16px;
        outline: 2px solid #333;
        border-radius: 8px;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

        > div {
          justify-content: flex-start;
          gap: 6px;
          border-radius: 8px;
        }

        div.time-container {
          border-radius: 8px;
          background-color: #222;

          input.primary-input {
            outline: none;
            color: #f6f6f6;
          }
        }

        input {
          height: 60px;
          font-size: 32px;
          max-width: 1.5em;
          min-width: 1.5em;
          text-align: center;
        }

        span {
          padding: 0 4px;
          color: #666;
          font-weight: bold;
          font-size: 24px;
          user-select: none;
        }
      }

      p {
        margin: 0;
        margin-bottom: 4px;
        font-weight: bold;
        color: #f6f6f6;
        user-select: none;
      }

      textarea {
        min-height: 80px;
        min-width: 100%;
        max-width: 100%;
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
      textarea:focus {
        outline-color: rgba(255, 70, 70, 1);
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