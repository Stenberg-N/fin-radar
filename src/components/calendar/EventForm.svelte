<script lang="ts">
  import { t, lang } from "$lib/i18n/i18n";
  import { calendarDate, addCalendarEvent, updateCalendarEvent } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { handleKeyDownOnInput, handleClickOutside } from "$lib/actions";
  import type { CalendarEventForm, CalendarEventWithTag } from "$lib/types";

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
  use:handleClickOutside={{ onOutsideClick: options.stopEdit, additionalElements: [options.openEventFormButton, ...options.calendarEventRefs] }}
>
  {#if isCalendar}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <Calendar options={{
        calendarToggle,
        calendarStartDate: $calendarDate,
        ignorableEls: [dateInput, ...options.navButtonRefs],
        isMonthChangeEnabled: options.editedEvent ? false : true,
        setCalendarIsoDate: (date) => { form.isodate = date; },
        setCalendarVisibility: (state) => { isCalendar = state; },
      }}
      />
    </ModalWrapper>
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <TagsList options={{
        setListVisibility: (state) => { isTagsListVisible = state; },
        tagsListToggleButton,
        isTagsListVisible,
        onAddButtonClick: (tag) => { form.tags = [...form.tags, tag]; },
        form,
        bgColor: "lighter",
      }}
      />
    </ModalWrapper>
  {/if}

  <div id="add-calendar-event-top-container" class="flex row">
    <div id="title-wrapper" class="flex row">
      <h2>{$t[options.editedEvent ? "calendar.edit-event.header" : "calendar.add-event.header"]}</h2>
      {#if options.editedEvent}
        <p title={options.editedEvent.event.title}>{options.editedEvent.event.title}</p>
      {/if}
    </div>
    <button aria-label="Close form" type="button" class="button-primary transparent highlight static" onclick={() => options.stopEdit()}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>

  <form id="add-calendar-event-form" class="form-bg" onsubmit={(e) => handleSubmit(e)}>
    <div id="add-calendar-event-date-title-container" class="flex row">
      {#each textInputs as input, i (i)}
        <div id={`add-calendar-event-${i === 0 ? "date" : "title"}-container`} class="flex row">
          {#if i === 0}
            <button aria-label="Toggle calendar" class="button-primary transparent" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
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
            onclick={() => i === 0 ? isCalendar = true : {}}
            required
          />
        </div>
      {/each}
    </div>

    <div id="add-calendar-event-body-container" class="flex row">
      <div class="flex column">
        <textarea placeholder={$lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}></textarea>
        <div id="add-calendar-event-timeframe-container" class="flex row">
          {#each otherInputs as input, i (i)}
            <div class="time-container-wrapper flex column">
              <div class="time-container flex row">
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[0] as TimeKey]} onkeydown={(e) => handleTimeInput(e.target, e)} />
                <span>:</span>
                <input maxlength="2" class="primary-input" placeholder="00" bind:value={form[input.keys[1] as TimeKey]} />
              </div>
              <p>{$t[input.title]}</p>
            </div>
            {#if i === 0}
              <span class="span-icon img-medium" style="mask-image: url('arrow.svg'); transform: rotate(-90deg); align-self: flex-start; margin-top: 18px;"></span>
            {/if}
          {/each}
        </div>
      </div>

      <div id="add-calendar-event-tags-list" class="flex column">
        <button type="button" bind:this={tagsListToggleButton} id="event-form-add-tag-button" class="button-primary light" onclick={() => isTagsListVisible = !isTagsListVisible}>
          <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isTagsListVisible ? '-45deg' : ''});"></span>
          {$t[isTagsListVisible ? "cancel.button" : "add.button"] + " " + (isTagsListVisible ? "" : ($lang === 'en' ? "tag" : "tunniste"))}
        </button>
        <div style="width: 100%; border-top: 2px solid var(--outline-color1); margin: 0.5rem 0;"></div>
        <div id="event-tag-rows-wrapper" class="flex column">
          {#if form.tags.length > 0}
            {#each form.tags as tag (tag.id)}
              <div class="event-tag-row flex row">
                <p title={tag.name}>{tag.name}</p>
                <div class="flex row" style="gap: 0.25rem;">
                  {#if isTagRemove.tagId === tag.id && isTagRemove.clickCount > 0}
                    <button aria-label="Delete tag" type="button" class="button-primary transparent highlight" onclick={clearTagRemove}>
                    <span class="span-icon img-extra-small" style="mask-image: url('close-x.svg');"></span>
                  </button>
                  {/if}
                  <button aria-label="Delete tag" type="button" class="button-primary transparent highlight" onclick={() => handleTagRemove(tag.id)} disabled={isTagRemove.tagId !== null && isTagRemove.tagId !== tag.id}>
                    <span class="span-icon img-small" style="mask-image: url('trash-can.svg'); background-color: {isTagRemove.tagId === tag.id ? 'var(--color-highlight1)' : 'var(--color-white-primary1)'}"></span>
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

    <div id="add-calendar-event-form-buttons" class="flex row">
      <button type="button" class="button-primary light" onclick={() => resetForm()}>
        <span class="span-icon img-small" style="mask-image: url('trash-can.svg');"></span>
        {$t["clear.button"]}
      </button>
      <button type="submit" class="button-primary white-bg">
        <span class="span-icon img-small" style="mask-image: url({options.editedEvent ? 'disk.svg' : 'plus.svg'});"></span>
        {$t[options.editedEvent ? "commit.button" : "add.button"]}
      </button>
    </div>
  </form>

</div>

<style>
  h2, p {
    margin: 0;
  }

  #add-calendar-event-form-container {
    background-color: var(--color-secondary1);
    min-height: 0;
    height: 100%;
    padding: 1rem 2rem 2rem;
    gap: 0.75rem;
  }

  #add-calendar-event-top-container {
    position: relative;
    justify-content: flex-start;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--outline-color1);

    #title-wrapper {
      justify-content: flex-start;
      max-width: calc(100% - 48px);
      gap: 1rem;
      overflow: hidden;
      text-wrap: nowrap;
    }

    p {
      font-weight: bold;
      color: var(--color-highlight1);
      overflow: hidden;
      text-overflow: ellipsis;
    }

    button {
      position: absolute;
      right: 0;
      margin-left: 1rem;
    }
  }

  #add-calendar-event-form {
    padding: 6px;
    gap: 2rem;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));
  }

  #add-calendar-event-date-title-container {
    justify-content: flex-start;
    gap: 1.5rem;

    div {
      position: relative;
      flex: 1 1 auto;
    }

    #add-calendar-event-date-container {
      max-width: 150px;

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
      height: 2rem;
      padding-left: 42px;
      outline: 2px solid var(--outline-color1);
      font-size: inherit;

      &:focus {
        outline-color: var(--color-highlight1);
      }
    }
  }

  #add-calendar-event-body-container {
    justify-content: flex-start;
    align-items: flex-start;
    gap: 1.5rem;
    max-height: 286px;

    > div:first-child {
      align-self: stretch;
      justify-content: flex-start;
      gap: 1.5rem;
      max-height: inherit;
    }

    textarea {
      flex: 1 1 auto;
      min-height: 80px;
      min-width: 100%;
      max-width: fit-content;
      padding: 6px;
      border: none;
      outline: 2px solid var(--outline-color1);
      border-radius: 0.25rem;
      background-color: transparent;
      font-size: 1rem;
    }
    textarea:focus {
      outline-color: var(--color-highlight1);
    }
  }

  #add-calendar-event-timeframe-container {
    gap: 0.25rem;
    padding: 6px;
    border-radius: 0.5rem;
    outline: 2px solid var(--outline-color1);

    .time-container-wrapper {
      gap: 0.25rem;
      
      p {
        margin: 0;
        font-weight: bold;
        user-select: none;
      }
    }

    .time-container {
      gap: 0.25rem;
      padding: 0.75rem;
      background-color: var(--color-secondary2);
      border-radius: 0.25rem;

      input {
        max-width: 1.3em;
        height: 36px;
        padding: 0;
        font-size: clamp(1rem, 1.6cqw, 1.25rem);
        text-align: center;
        outline: 2px solid var(--outline-color2);

        &:not(:disabled):focus {
          outline-color: var(--color-highlight1);
        }
      }

      span {
        padding: 0 0 6px;
        color: #666;
        font-weight: bold;
        font-size: 1.5rem;
        user-select: none;
      }
    }
  }

  #add-calendar-event-tags-list {
    flex: 1 1 auto;
    justify-content: unset;
    align-items: flex-start;
    align-self: stretch;
    max-width: 250px;
    padding: 6px;
    border-radius: 0.5rem;
    outline: 2px solid var(--outline-color1);

    #event-tag-rows-wrapper {
      justify-content: flex-start;
      align-items: flex-start;
      width: 100%;
      max-height: 224px;
      gap: 0.25rem;
      padding: 0;
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      > p { align-self: center; }

      .event-tag-row {
        width: 100%;
        justify-content: space-between;
        padding: 0.5rem;
        gap: 0.5rem;
        border-radius: 0.2rem;
        background-color: var(--color-secondary2);

        button {
          width: 1.5rem;
          height: 1.5rem;
          border-radius: 0.25rem;
        }
      }
    }

    #event-form-add-tag-button {
      height: 2rem;
      
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
    gap: 0.75rem;

    button {
      height: unset;
      padding: 0.5rem 1rem;
    }
  }
</style>