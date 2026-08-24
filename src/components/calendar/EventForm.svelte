<script lang="ts">
  import { getContext } from "svelte";

  import { t, lang } from "$lib/i18n";
  import { calendarDate, addCalendarEvent, updateCalendarEvent } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { handleKeyDownOnInput, handleClickOutside } from "$lib/actions";
  import type { CalendarEventForm, CalendarEventWithTag, CalendarTag } from "$lib/types";

  import Calendar from "../Calendar.svelte";
  import TagsList from "./TagsList.svelte";
  import ModalWrapper from "../ModalWrapper.svelte";

  type FormKey = "isodate" | "title" | "description";
  type TimeKey = "startTimeHours" | "startTimeMinutes" | "endTimeHours" | "endTimeMinutes";

  let {
    options,
  }: {
    options: {
      stopEdit: () => void;
      editedEvent: CalendarEventWithTag | null;
      navButtonRefs: HTMLButtonElement[];
      openEventFormButton: HTMLButtonElement | null;
      calendarEventRefs: HTMLElement[];
    },
  } = $props();

  // svelte-ignore state_referenced_locally
  let form = $state<CalendarEventForm>(formFromEvent(options.editedEvent));
  let calendarToggle = $state<HTMLButtonElement | null>(null);
  let isCalendar = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  let isTagRemove = $state<{ tagId: number | null, clickCount: number}>({tagId: null, clickCount: 0});
  const textInputs = [
    { title: "date-input.description", key: "isodate" },
    { title: "title-input.description", key: "title" },
  ];
  const otherInputs = [
    { title: "calendar.start-time.description", keys: ["startTimeHours", "startTimeMinutes"] },
    { title: "calendar.end-time.description", keys: ["endTimeHours", "endTimeMinutes"] },
  ];
  const excludedKeys = ["Backspace", "Control", "ArrowLeft", "ArrowRight", "Tab"];
  const timeInputRegex = /^[0-9]$/;

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);
  let dateInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    form = formFromEvent(options.editedEvent);
  });

  $effect(() => {
    if (formInputRefs[1]) dateInput = formInputRefs[1];
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
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
      tags: $state.snapshot(source.tags)
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
  const clearTagRemove = () => {
    isTagRemove.tagId = null;
    isTagRemove.clickCount = 0;
  };
  
  /***********************************************************************************************************************************/

  const handleSubmit = async (e: SubmitEvent) => {
    e.preventDefault();

    const result = options.editedEvent ? await updateCalendarEvent(form, options.editedEvent) : await addCalendarEvent(form);
    if (result.success) { clearForm(); options.stopEdit(); }
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

  const handleTagRemove = (tagId: number) => {
    if (isTagRemove.clickCount === 0) isTagRemove.tagId = tagId;
    isTagRemove.clickCount++;

    if (isTagRemove.clickCount >= 2 && isTagRemove.tagId === tagId) {
      form.tags = form.tags.filter((t) => t.id !== isTagRemove.tagId);
      clearTagRemove();
    }
  };

</script>

<div id="add-calendar-event-form-container" class="form-outer-container"
  use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => options.stopEdit(), additionalElements: [options.openEventFormButton, ...options.calendarEventRefs] }}
>
  {#if isCalendar}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <Calendar options={{
        calendarToggle,
        calendarStartDate: $calendarDate,
        ignorableEls: [dateInput, ...options.navButtonRefs],
        isMonthChangeEnabled: options.editedEvent ? false : true,
        setCalendarIsoDate: (date) => form.isodate = date,
        setCalendarVisibility: (state) => isCalendar = state,
      }}
      />
    </ModalWrapper>
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <TagsList options={{
        setListVisibility: (state) => isTagsListVisible = state,
        tagsListToggleButton,
        isTagsListVisible,
        onAddButtonClick: (tag) => form.tags = [...form.tags, tag],
        form
      }}
      />
    </ModalWrapper>
  {/if}

  <div id="add-calendar-event-top-container" class="horizontal-flex-container">
    <div id="title-wrapper" class="horizontal-flex-container">
      <h2>{$t[options.editedEvent ? "calendar.edit-event.header" : "calendar.add-event.header"]}</h2>
      {#if options.editedEvent}
        <p title={options.editedEvent.event.title}>{options.editedEvent.event.title}</p>
      {/if}
    </div>
    <button aria-label="Close form" type="button" class="transparent-button-highlight" onclick={() => options.stopEdit()}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>

  <form id="add-calendar-event-form" class="form-bg" onsubmit={(e) => handleSubmit(e)}>
    <div id="add-calendar-event-date-title-container" class="horizontal-flex-container">
      {#each textInputs as input, i (i)}
        <div id={`add-calendar-event-${i === 0 ? "date" : "title"}-container`} class="horizontal-flex-container">
          {#if i === 0}
            <button aria-label="Toggle calendar" class="transparent-button horizontal-flex-container" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
              <span class="span-icon img-medium" style="mask-image: url('calendar.svg');"></span>
            </button>
          {:else}
            <span class="span-icon img-medium" style="mask-image: url('edit-pen.svg'); position: absolute; left: 6px;"></span>
          {/if}
          <input
            class="primary-input"
            type="text"
            placeholder={$t[i === 0 ? "placeholder.isodate" : "title-input.description"] as string}
            bind:value={form[input.key as FormKey]}
            bind:this={formInputRefs[i]}
            onkeydown={(e) => { if (i === 0) handleKeyDownOnInput("date", e) }}
            required
          />
        </div>
      {/each}
    </div>

    <div id="add-calendar-event-body-container" class="horizontal-flex-container">
      <div class="vertical-flex-container">
        <textarea placeholder={$lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}></textarea>
        <div id="add-calendar-event-timeframe-container" class="horizontal-flex-container">
          {#each otherInputs as input, i (i)}
            <div class="time-container-wrapper vertical-flex-container">
              <div class="time-container horizontal-flex-container">
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[0] as TimeKey]} onkeydown={(e) => handleTimeInput(e.target, e)} />
                <span>:</span>
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[1] as TimeKey]} />
              </div>
              <p>{$t[input.title]}</p>
            </div>
            {#if i === 0}
              <span class="span-icon img-medium" style="mask-image: url('arrow.svg'); transform: rotate(-90deg); align-self: flex-start; margin-top: 22px;"></span>
            {/if}
          {/each}
        </div>
      </div>

      <div id="add-calendar-event-tags-list" class="vertical-flex-container">
        <button type="button" bind:this={tagsListToggleButton} id="event-form-add-tag-button" class="primary-button-light" onclick={() => isTagsListVisible = !isTagsListVisible}>
          <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isTagsListVisible ? '-45deg' : ''});"></span>
          {$t[isTagsListVisible ? "cancel.button" : "add.button"] + " " + ($lang === 'en' ? "tag" : "tunniste")}
        </button>
        <div style="width: 100%; border-top: 2px solid #333; margin: 8px 0;"></div>
        <div id="event-tag-rows-wrapper" class="vertical-flex-container">
          {#if form.tags.length > 0}
            {#each form.tags as tag (tag.id)}
              <div class="event-tag-row horizontal-flex-container">
                <p title={tag.name}>{tag.name}</p>
                <div class="horizontal-flex-container" style="gap: 4px;">
                  {#if isTagRemove.tagId === tag.id && isTagRemove.clickCount > 0}
                    <button aria-label="Delete tag" type="button" class="transparent-button-highlight" onclick={clearTagRemove}>
                    <span class="span-icon" style="mask-image: url('close-x.svg'); width: 12px; height: 12px;"></span>
                  </button>
                  {/if}
                  <button aria-label="Delete tag" type="button" class="transparent-button-highlight" onclick={() => handleTagRemove(tag.id)} disabled={isTagRemove.tagId !== null && isTagRemove.tagId !== tag.id}>
                    <span class="span-icon img-small" style="mask-image: url('trash-can.svg'); background-color: {isTagRemove.tagId === tag.id ? 'rgb(255, 70, 70)' : '#f6f6f6'}"></span>
                  </button>
                </div>
              </div>
            {/each}
          {:else}
            <p>{$t["calendar.tags-list.no-tags"]}</p>
          {/if}
        </div>
      </div>
    </div>

    <div id="add-calendar-event-form-buttons" class="horizontal-flex-container">
      <button type="button" class="primary-button-light" onclick={() => resetForm()}>
        <span class="span-icon img-small" style="mask-image: url('trash-can.svg');"></span>
        {$t["clear.button"]}
      </button>
      <button type="submit" class="primary-button-light">
        <span class="span-icon img-small" style="mask-image: url({options.editedEvent ? 'disk.svg' : 'plus.svg'});"></span>
        {$t[options.editedEvent ? "commit.button" : "add.button"]}
      </button>
    </div>
  </form>

</div>

<style>
  h2, p {
    margin: 0;
    color: #f6f6f6;
  }

  #add-calendar-event-form-container {
    background-color: #222;
    min-height: 0;
    height: 100%;
    padding: 16px 32px 32px;
    gap: 12px;
  }

  #add-calendar-event-top-container {
    position: relative;
    justify-content: flex-start;

    #title-wrapper {
      justify-content: flex-start;
      max-width: calc(100% - 48px);
      gap: 16px;
      overflow: hidden;
      text-wrap: nowrap;
    }

    p {
      font-weight: bold;
      color: rgb(255, 70, 70);
      overflow: hidden;
      text-overflow: ellipsis;
    }

    button {
      position: absolute;
      right: 0;
      width: 32px;
      height: 32px;
      margin-left: 16px;
    }
  }

  #add-calendar-event-form {
    padding: 6px;
    gap: 32px;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));
  }

  #add-calendar-event-date-title-container {
    justify-content: flex-start;
    gap: 40px;

    div {
      position: relative;
      flex: 1 1 auto;
    }

    #add-calendar-event-date-container {
      max-width: 148px;

      button {
        position: absolute;
        left: 0;
        padding-left: 6px;
        transition: transform 0.2s;

        &:hover {
          transform: scale(1.1);
        }
      }
    }

    input {
      height: 32px;
      padding-left: 42px;
      outline: 2px solid #333;

      &:focus {
        outline-color: rgba(255, 70, 70, 1);
      }
    }
  }

  #add-calendar-event-body-container {
    justify-content: flex-start;
    align-items: flex-start;
    gap: 32px;

    > div:first-child {
      gap: 32px;
    }

    textarea {
      min-height: 80px;
      min-width: 100%;
      max-width: fit-content;
      font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
      padding: 6px;
      border: none;
      outline: 2px solid #333;
      border-radius: 4px;
      background-color: transparent;
      color: #f6f6f6;
      font-size: 16px;
    }
    textarea:focus {
      outline-color: rgba(255, 70, 70, 1);
    }
  }

  #add-calendar-event-timeframe-container {
    gap: 4px;
    padding: 6px;
    border-radius: 8px;
    outline: 2px solid #333;

    .time-container-wrapper {
      gap: 4px;
      
      p {
        margin: 0;
        color: #f6f6f6;
        font-weight: bold;
        user-select: none;
      }
    }

    .time-container {
      gap: 4px;
      padding: 12px;
      background-color: #333;
      border-radius: 4px;

      input {
        flex-shrink: 0;
        width: 1.5em;
        height: 36px;
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

  #add-calendar-event-tags-list {
    flex: 1 1 auto;
    justify-content: unset;
    align-items: flex-start;
    max-width: 240px;
    padding: 6px;
    border-radius: 8px;
    outline: 2px solid #333;

    #event-tag-rows-wrapper {
      justify-content: flex-start;
      align-items: flex-start;
      width: 100%;
      max-height: 224px;
      gap: 4px;
      padding: 4px 0;
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      > p { align-self: center; }

      .event-tag-row {
        width: 100%;
        justify-content: space-between;
        padding: 8px;
        border-radius: 4px;
        background-color: #333;

        button {
          width: 24px;
          height: 24px;
          border-radius: 4px;
        }
      }
    }

    #event-form-add-tag-button {
      height: 32px;
      
      span {
        transition: transform 0.1s;
      }
    }

    p {
      overflow: hidden;
      text-wrap: nowrap;
      text-overflow: ellipsis;
    }
  }

  #add-calendar-event-form-buttons {
    justify-content: flex-end;
    gap: 12px;

    button {
      padding: 12px 24px;
      gap: 8px;
      font-size: 18px;
    }
  }
</style>