<script lang="ts">
  import { t, lang } from "$lib/i18n";
  import { calendarDate, addCalendarEvent, updateCalendarEvent } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { handleKeyDownOnInput } from "$lib/actions";
  import type { CalendarEventForm, CalendarEventWithTag } from "$lib/types";

  import Calendar from "../Calendar.svelte";
  import TagsList from "./TagsList.svelte";

  type FormKey = "isodate" | "title" | "description";
  type TimeKey = "startTimeHours" | "startTimeMinutes" | "endTimeHours" | "endTimeMinutes";

  let {
    closeForm,
    navButtonRefs,
    editedEvent,
  }: {
    closeForm: () => void;
    navButtonRefs: HTMLButtonElement[];
    editedEvent: CalendarEventWithTag | null;
  } = $props();

  // svelte-ignore state_referenced_locally
  let form = $state<CalendarEventForm>(formFromEvent(editedEvent));
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  const textInputs = [
    { title: "title-input.description", key: "title" },
    { title: "date-input.description", key: "isodate" },
  ];
  const otherInputs = [
    { title: "calendar.start-time.description", keys: ["startTimeHours", "startTimeMinutes"] },
    { title: "calendar.end-time.description", keys: ["endTimeHours", "endTimeMinutes"] },
  ];
  const excludedKeys = ["Backspace", "Control", "ArrowLeft", "ArrowRight"];
  const timeInputRegex = /^[0-9]$/;

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);
  let dateInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    form = formFromEvent(editedEvent);
  });

  $effect(() => {
    if (formInputRefs[1]) dateInput = formInputRefs[1];
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleTimeInput = (target: EventTarget | null, e: KeyboardEvent) => {
    if (!target) return;
    if (excludedKeys.includes(e.key) || (e.ctrlKey && (e.key.toLowerCase() === 'a' || e.key.toLowerCase() === 'z'))) return;
    if (!timeInputRegex.test(e.key)) e.preventDefault();

    const input = target as HTMLInputElement;
    if (input.value.length >= 2) {
      const nextInput = input.parentElement?.lastChild as HTMLInputElement;
      if (nextInput) nextInput.focus();
    }
  };
  function formFromEvent(source: CalendarEventWithTag | null): CalendarEventForm {
    if (source) return {
      isodate: source.event.isodate,
      title: source.event.title,
      description: source.event.description,
      startTimeHours: source.event.start_time ? String(Math.floor(source.event.start_time / 3600)).padStart(2, '0') : null,
      startTimeMinutes: source.event.start_time ? String(Math.floor(source.event.start_time % 3600 / 60)).padStart(2, '0') : null,
      endTimeHours: source.event.end_time ? String(Math.floor(source.event.end_time / 3600)).padStart(2, '0') : null,
      endTimeMinutes: source.event.end_time ? String(Math.floor(source.event.end_time % 3600 / 60)).padStart(2, '0') : null,
      // It is necessary to deep clone tags for the calendar event update function!
      tags: structuredClone($state.snapshot(source.tags))
    };
    return {
      isodate: '',
      title: '',
      description: null,
      startTimeHours: null,
      startTimeMinutes: null,
      endTimeHours: null,
      endTimeMinutes: null,
      tags: [],
    };
  }
  
  /***********************************************************************************************************************************/

  const handleSubmit = async () => {
    const result = editedEvent ? await updateCalendarEvent(form, editedEvent) : await addCalendarEvent(form);
    if (result.success) { clearForm(); closeForm(); }
  };

  const resetForm = () => {
    sendAlert({
      message: "alert.clear-form.question",
      isTimer: false,
      buttons: true,
      onConfirm: () => clearForm()
    });
  };

  const clearForm = () => {
    form.isodate = '',
    form.title = '',
    form.description = null,
    form.startTimeHours = null,
    form.startTimeMinutes = null,
    form.endTimeHours = null,
    form.endTimeMinutes = null
  };

</script>

<div id="add-calendar-event-form-container" class="form-outer-container">
  {#if isCalendar}
    <Calendar {calendarToggle} calendarStartDate={$calendarDate} ignorableEls={[dateInput, ...navButtonRefs]} isMonthChangeEnabled={false}
      setCalendarIsoDate={(date) => form.isodate = date}
      setCalendarVisibility={(state) => isCalendar = state}
    />
  {/if}

  {#if isTagsListVisible}
    <TagsList options={{
      setListVisibility: (state) => isTagsListVisible = state,
      tagsListToggleButton,
      isTagsListVisible,
      onAddButtonClick: (tag) => form.tags.push(tag),
      form
    }}
    />
  {/if}

  <div id="add-calendar-event-title-container" class="horizontal-flex-container">
    <div class="vertical-flex-container">
      <h2>{$t[editedEvent ? "calendar.edit-event.header" : "calendar.add-event.header"]}</h2>
      {#if editedEvent}
        <p>{editedEvent.event.title}</p>
      {/if}
    </div>
    <button type="button" class="transparent-button-highlight" onclick={() => closeForm()}>
      <img src="/close-x.svg" alt="Close" class="img-small" />
    </button>
  </div>

  <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div id="calendar-event-form-main-info-container" class="horizontal-flex-container">
      <div class="vertical-flex-container">
        <div id="calendar-event-form-time-container">
          <p>{$t["calendar.time-frame.description"]}</p>
          <div class="time-input-outer-container horizontal-flex-container">
            {#each otherInputs as input, i (i)}
              <div class="vertical-flex-container">
                <div class="time-container horizontal-flex-container">
                  <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[0] as TimeKey]} onkeydown={(e) => handleTimeInput(e.target, e)} />
                  <span>:</span>
                  <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[1] as TimeKey]} />
                </div>
                <p>{$t[input.title]}</p>
              </div>
              {#if i === 0}
                <img src="arrow.svg" alt="Arrow" class="img-medium" style="transform: rotate(-90deg); align-self: flex-start; margin: 18px 8px;" />
              {/if}
            {/each}
          </div>
        </div>
        <div id="date-title-container" class="vertical-flex-container">
          {#each textInputs as input, i (i)}
            <div class="vertical-flex-container">
              <p>{$t[input.title]}</p>
              <div class="calendar-event-input-container form-input-container">
                <input class="primary-input" type="text" style="{i === 0 ? 'padding-right: 44px' : ''};" placeholder={$t[i === 1 ? "placeholder.isodate" : "title-input.description"] as string}
                  bind:value={form[input.key as FormKey]}
                  bind:this={formInputRefs[i]}
                  onkeydown={(e) => { if (i === 0) handleKeyDownOnInput("date", e) }}
                  required
                />
                {#if i === 1}
                  <button class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
                    <img src="calendar.svg" alt="Calendar" class="img-large" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

      <div id="calendar-event-form-tags-container" class="vertical-flex-container">
        <p>{$t["calendar.tags-list-header"]}</p>
        <div id="event-tags-list" class="vertical-flex-container">
          <button type="button" bind:this={tagsListToggleButton} id="event-form-add-tag-button" class="primary-button horizontal-flex-container" onclick={() => isTagsListVisible = !isTagsListVisible}>
            <img src="plus.svg" alt="Plus" class="img-small" style="transform: rotate({isTagsListVisible ? '-45deg' : ''});" />
            {$t[isTagsListVisible ? "cancel.button" : "add.button"]}
          </button>
          <div style="width: 100%; border-top: 2px solid #333; margin: 8px 0;"></div>
          <div id="event-tag-rows-wrapper" class="vertical-flex-container">
            {#if form.tags.length > 0}
              {#each form.tags as tag (tag.id)}
                <div>
                  <p>{tag.name}</p>
                  <button type="button" onclick={() => form.tags = form.tags.filter(t => t.id !== tag.id)}>
                    DEL
                  </button>
                </div>
              {/each}
            {:else}
              <p style="align-self: center;">{$t["calendar.tags-list.no-tags"]}</p>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div id="calendar-event-form-description-container" class="vertical-flex-container">
      <p>{$t["description-input.description"]}</p>
      <textarea placeholder={$lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}></textarea>
    </div>

    <div id="calendar-event-form-buttons" class="horizontal-flex-container">
      <button type="submit" class="primary-button-light horizontal-flex-container">
        <img src="{editedEvent ? 'disk.svg' : 'plus.svg'}" alt="{editedEvent ? 'Save disk' : 'Plus'}" class="img-small" />
        {$t[editedEvent ? "commit.button" : "add.button"]}
      </button>
      <button type="button" class="primary-button-light horizontal-flex-container" onclick={() => resetForm()}>
        <img src="trash-can.svg" alt="Trash can" class="img-small" />
        {$t["clear.button"]}
      </button>
    </div>
  </form>
</div>

<style>
  p {
    margin: 0;
    margin-bottom: 4px;
    font-weight: bold;
    color: #f6f6f6;
    user-select: none;
  }

  #add-calendar-event-form-container {
    box-shadow: none;
    background-color: #222;
    min-height: 0;
    height: 100%;
    padding: 16px 32px 32px;

    #add-calendar-event-title-container {
      position: relative;
      padding-bottom: 16px;
      border-bottom: 2px solid #333;

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
      padding: 16px;
      gap: 32px;
      overflow-y: auto;
      overflow-x: hidden;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      > div, #date-title-container {
        justify-content: flex-start;
        width: 100%;

        > div > p {
          align-self: flex-start;
        }
      }
    }

    #calendar-event-form-main-info-container {
      gap: 40px;

      > div {
        max-height: 360px;
      }

      > div, #date-title-container {
        gap: 32px;
      }

      #date-title-container > div {
        width: 100%;
      }

      #calendar-event-form-time-container {
        width: 100%;

        div.time-input-outer-container {
          align-self: flex-end;
          align-items: flex-end;
          height: 100%;
          padding: 16px 16px 0;
          outline: 2px solid #333;
          border-radius: 8px;
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

          > div {
            justify-content: flex-start;
            gap: 6px;
            
            p {
              margin-bottom: 2px;
            }
          }

          div.time-container {
            border-radius: 8px;
            background-color: #333;

            input.primary-input {
              outline: none;
              color: #f6f6f6;
            }
          }

          input {
            flex-shrink: 0;
            width: 1.5em;
            height: 60px;
            padding: 4px;
            font-size: 24px;
            text-align: center;
          }

          span {
            padding: 0;
            color: #666;
            font-weight: bold;
            font-size: 24px;
            user-select: none;
          }
        }
      }

      .calendar-event-input-container {
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

      #calendar-event-form-tags-container {
        align-self: stretch;
        justify-content: flex-start;
        gap: 0;
        width: 100%;

        #event-tags-list {
          flex: 1 1 auto;
          width: 100%;
          align-items: flex-start;
          justify-content: flex-start;
          padding: 6px;
          outline: 2px solid #333;
          border-radius: 8px;
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
          overflow: hidden;

          #event-tag-rows-wrapper {
            justify-content: flex-start;
            align-items: flex-start;
            width: 100%;
            overflow-y: auto;
            mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
          }

          #event-form-add-tag-button {
            height: 32px;
            gap: 8px;
            background-color: #444;
            
            img {
              transition: transform 0.1s;
            }
          }
          #event-form-add-tag-button:hover {
            background-color: #555;
          }

          p {
            word-break: break-word;
          }
        }
      }
    }

    #calendar-event-form-description-container {
      align-items: flex-start;

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