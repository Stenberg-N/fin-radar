<script lang="ts">
  import { beforeNavigate, goto } from "$app/navigation";

  import { i18n } from "#lib/i18n/i18n.svelte.ts";
  import { calendarDate, addCalendarEvent, updateCalendarEvent } from "#lib/calendar.ts";
  import { sendAlert } from "#lib/classes/alertStore.svelte.ts";
  import { handleKeyDownOnInput } from "#lib/utils/helpers.svelte.ts";
  import { HoverTitle } from "#lib/classes/hoverTitle.svelte.ts";
  import type { CalendarEventForm, CalendarEventWithTag } from "#lib/types.ts";

  import Calendar from "../Calendar.svelte";
  import TagsList from "./TagsList.svelte";
  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    stopEdit: () => void;
    editedEvent: CalendarEventWithTag | null;
    ignorableEls: (HTMLElement | null)[];
    /** Used to return a boolean value on the form's edited state. */
    updateChanges: (areChanges: boolean) => void;
    selectedIsodate?: string | null;
  }

  type FormKey = "isodate" | "title" | "description";
  type TimeKey = "startTimeHours" | "startTimeMinutes" | "endTimeHours" | "endTimeMinutes";
  type HoverContent = { content: string };

  let {
    stopEdit: propStopEdit,
    editedEvent,
    ignorableEls,
    updateChanges,
    selectedIsodate,
  }: Props = $props();

  const stopEdit = () => {
    if (isFormChanged) {
      sendAlert({
        message: (i18n.t["alert.unsaved-changes"] as string[])[0],
        isTimer: false,
        buttons: true,
        onConfirm: () => { propStopEdit(); updateChanges(false); },
        additionalText: [i18n.t["alert.navigate-without-saving"] as string],
      });
    } else {
      propStopEdit();
      updateChanges(false);
    }
  };

  // svelte-ignore state_referenced_locally
  let form = $state<CalendarEventForm>(formFromEvent(editedEvent));
  // svelte-ignore state_referenced_locally
  let originalEditedEvent = formFromEvent(editedEvent);

  const isFormChanged = $derived(isEventEdited(originalEditedEvent, form));
  $effect(() => updateChanges(isFormChanged));

  let isCalendar = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  let isTagRemove = $state<{ tagId: number | null; clickCount: number }>({ tagId: null, clickCount: 0 });
  const excludedKeys = ["Backspace", "Control", "ArrowLeft", "ArrowRight", "Tab"];
  const timeInputRegex = /^[0-9]$/;
  const hover = new HoverTitle<HoverContent>();

  const textInputs = [
    { title: "title-input.description", key: "title" },
    { title: "date-input.description", key: "isodate" },
  ];
  const otherInputs = [
    {
      get title() { return i18n.t["calendar.start-time.description"] as string; },
      keys: ["startTimeHours", "startTimeMinutes"],
      icon: "/clock.svg"
    },
    {
      get title() { return i18n.t["calendar.end-time.description"] as string; },
      keys: ["endTimeHours", "endTimeMinutes"],
      icon: "/hourglass-end.svg"
    },
  ];

  let formInputRefs = $state<HTMLInputElement[]>([]);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);
  let dateInput = $state<HTMLInputElement | null>(null);
  let calendarToggle = $state<HTMLButtonElement | null>(null);

  let allowNavigation = false;

  beforeNavigate(({ cancel, to, shallow, type }) => {
    if (shallow && type === 'goto') return;
    if (allowNavigation) return;
    if (!isFormChanged) return;

    cancel();
    const pendingNavigation = to?.url.pathname;
    sendAlert({
      message: (i18n.t["alert.unsaved-changes"] as string[])[0],
      isTimer: false,
      buttons: true,
      onConfirm: () => pendingNavigation ? (allowNavigation = true, goto(pendingNavigation)) : {},
      additionalText: [i18n.t["alert.navigate-without-saving"] as string],
    });
  });

  $effect(() => {
    if (formInputRefs[1]) dateInput = formInputRefs[1];
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });

  $effect(() => {
    const incomingEvent = editedEvent;
    form = formFromEvent(incomingEvent);
    originalEditedEvent = formFromEvent(incomingEvent);
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleTimeInput = (target: EventTarget | null, e: KeyboardEvent) => {
    if (!target) return;
    if (e.key === 'Escape') return;
    if (excludedKeys.includes(e.key) || e.ctrlKey && (e.key.toLowerCase() === 'a' || e.key.toLowerCase() === 'z')) return;
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
      startTimeHours: source.event.start_time !== null ? String(Math.floor(source.event.start_time / 3600)).padStart(2, '0') : null,
      startTimeMinutes: source.event.start_time !== null ? String(Math.floor(source.event.start_time % 3600 / 60)).padStart(2, '0') : null,
      endTimeHours: source.event.end_time ? String(Math.floor(source.event.end_time / 3600)).padStart(2, '0') : null,
      endTimeMinutes: source.event.end_time ? String(Math.floor(source.event.end_time % 3600 / 60)).padStart(2, '0') : null,
      // It is necessary to deep clone tags for the calendar event update function!
      tags: $state.snapshot(source.tags)
    };
    return {
      isodate: selectedIsodate ?? '',
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
  const clearElementContent = (e: KeyboardEvent, key: Exclude<keyof CalendarEventForm, 'tags'>) => {
    e.stopPropagation();
    form[key] = '';
  };
  function isEventEdited(originalEvent: CalendarEventForm, form: CalendarEventForm) {
    const keys = Object.keys(form) as (keyof CalendarEventForm)[];

    const isUnchanged = keys.every((key) => {
      if (key === "tags") {
        const a = originalEvent.tags;
        const b = form.tags;
        return a.length === b.length && a.every((tag) => b.some((t) => tag.id === t.id));
      }
      return form[key] === originalEvent[key];
    });

    return !isUnchanged;
  };

  /***********************************************************************************************************************************/

  const handleSubmit = async (e: SubmitEvent) => {
    e.preventDefault();

    const result = editedEvent ? await updateCalendarEvent(form, editedEvent) : await addCalendarEvent(form);
    if (result.success) { clearForm(); propStopEdit(); updateChanges(false); }
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
    (
      form.isodate = '',
      form.title = '',
      form.description = null,
      form.startTimeHours = null,
      form.startTimeMinutes = null,
      form.endTimeHours = null,
      form.endTimeMinutes = null
    );
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

<div id="add-calendar-event-form-container" class="form-outer-container">
  {#if hover.isHovering && hover.target?.content.trim() !== ''}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ centerElement: true, moveTop: -40 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      borderRadius={8}
      outline={{ width: 1, color: 'var(--outline-color3)'}}
    >
      <p id="event-form-hover-title-content">
        {hover.target?.content}
      </p>
    </ModalWrapper>
  {/if}

  {#if isCalendar}
    <ModalWrapper
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
    >
      <Calendar
        calendarToggle={calendarToggle}
        calendarStartDate={$calendarDate}
        ignorableEls={[dateInput, ...ignorableEls]}
        isMonthChangeEnabled={false}
        setCalendarIsoDate={(date) => { form.isodate = date; }}
        setCalendarVisibility={(state) => { isCalendar = state; }}
      />
    </ModalWrapper>
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper
      position={{ isDraggable: true }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      onOutsideClick={() => { isTagsListVisible = false; }}
      ignorableEls={[tagsListToggleButton]}
      dragHandleColor="lighter1"
      outline={{ width: 1, color: 'var(--outline-color3)'}}
    >
      <TagsList
        setListVisibility={(state) => {
          isTagsListVisible = state;
        }}
        onAddButtonClick={(tag) => {
          form.tags = [...form.tags, tag];
        }}
        form={form}
        theme="lighter"
      />
    </ModalWrapper>
  {/if}

  <div id="add-calendar-event-top-container" class="flex row">
    <div id="title-wrapper" class="flex column">
      <h2>{i18n.t[editedEvent ? "calendar.edit-event.header" : "calendar.add-event.header"]}</h2>
      {#if editedEvent}
        <p
          onmouseenter={(e) => hover.enter(e, { content: editedEvent.event.title ?? '' })}
          onmouseleave={(e) => hover.leave(e)}
        >
          {editedEvent.event.title}
        </p>
      {/if}
    </div>
    <button aria-label="Close form" type="button" class="button-primary transparent highlight static" onclick={stopEdit}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>

  <form id="calendar-event-form" class="form-bg" onsubmit={(e) => handleSubmit(e)}>
    <div id="event-body-wrapper">
      <div id="event-body-content" class="flex column">
        {#each textInputs as {title, key}, i (key)}
          <div id={`event-${i === 1 ? "date" : "title"}-container`} class="flex row">
            {#if i === 1}
              <button aria-label="Toggle calendar" class="button-primary transparent" type="button" bind:this={calendarToggle} onclick={() => isCalendar = !isCalendar}>
                <span
                  role="img"
                  class="span-icon img-small-medium"
                  style="mask-image: url('calendar.svg');"
                  onmouseenter={(e) => hover.enter(e, { content: i18n.t["date-input.description"] as string }) }
                  onmouseleave={(e) => hover.leave(e)}
                ></span>
              </button>
            {:else}
              <span
                role="img"
                class="span-icon img-small-medium"
                style="mask-image: url('edit-pen.svg');"
                onmouseenter={(e) => hover.enter(e, { content: i18n.t["title-input.description"] as string }) }
                onmouseleave={(e) => hover.leave(e)}
              ></span>
            {/if}
            <input
              class="primary-input"
              type="text"
              placeholder={i18n.t[i === 1 ? "placeholder.isodate" : "title-input.description"] as string}
              bind:value={form[key as FormKey]}
              bind:this={formInputRefs[i]}
              onkeydown={(e) => {
                if (i === 1) handleKeyDownOnInput("date", e);
                switch (e.key) {
                  case 'Escape': clearElementContent(e, key as FormKey);
                }
              }}
              onfocus={() => i === 1 ? isCalendar = true : {}}
            />
          </div>
        {/each}

        {#each otherInputs as input, i (i)}
          <div class="time-container flex row">
            <span
              aria-label={input.title}
              role="img"
              class="span-icon img-small-medium"
              style="mask-image: url({input.icon});"
              onmouseenter={(e) => { hover.enter(e, { content: input.title }) }}
              onmouseleave={(e) => hover.leave(e)}
            ></span>
            <div class="flex">
              <input maxlength="2" placeholder="00" class="primary-input" bind:value={form[input.keys[0] as TimeKey]}
                onkeydown={(e) => {
                  handleTimeInput(e.target, e);
                  switch (e.key) {
                    case 'Escape': clearElementContent(e, input.keys[0] as TimeKey);
                  }
                }}
              />
              <span>:</span>
              <input maxlength="2" placeholder="00" class="primary-input" bind:value={form[input.keys[1] as TimeKey]}
                onkeydown={(e) => {
                  handleTimeInput(e.target, e);
                  switch (e.key) {
                    case 'Escape': clearElementContent(e, input.keys[1] as TimeKey);
                  }
                }}
              />
            </div>
          </div>
        {/each}
        <textarea placeholder={i18n.lang === 'en' ? 'Add an optional description...' : 'Lisää vaihtoehtoinen kuvaus...'} bind:value={form.description as FormKey}
          onkeydown={(e) => {
            switch (e.key) {
              case 'Escape': clearElementContent(e, "description");
            }
          }}
        ></textarea>
      </div>

      <div
        id="add-calendar-event-tags-list"
        class="flex column"
      >
        <button
          type="button"
          bind:this={tagsListToggleButton}
          id="event-form-add-tag-button"
          class="button-primary light"
          onclick={() => isTagsListVisible = !isTagsListVisible}
        >
          <span
            class="span-icon img-small"
            style="mask-image: url('plus.svg'); transform: rotate({isTagsListVisible ? '-45deg' : ''});"
          ></span>

          {i18n.t[isTagsListVisible ? "cancel.button" : "add.button"] + " " + (isTagsListVisible ? "" : i18n.lang === 'en' ? "tag" : "tunniste")}
        </button>
        <div style="width: 100%; border-top: 2px solid var(--outline-color2); margin: 0.5rem 0;"></div>
        <div id="event-tag-rows-wrapper" class="flex column">
          {#if form.tags.length > 0}
            {#each form.tags as tag (tag.id)}
              <div class="event-tag-row flex row">
                <p
                  onmouseenter={(e) => hover.enter(e, { content: tag.name })}
                  onmouseleave={(e) => hover.leave(e)}
                >
                  {tag.name}
                </p>
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
            <p>{i18n.t["calendar.tags-list.no-tags"]}</p>
          {/if}
        </div>
      </div>
    </div>

    <div id="add-calendar-event-form-buttons" class="flex row">
      <button type="button" class="button-primary light" onclick={() => resetForm()}>
        <span class="span-icon img-small" style="mask-image: url('trash-can.svg');"></span>
        {i18n.t["clear.button"]}
      </button>
      <button type="submit" class="button-primary white-bg">
        <span class="span-icon img-small" style="mask-image: url('{editedEvent ? "/disk" : "/plus"}.svg');"></span>
        {i18n.t[`${editedEvent ? "commit" : "add"}.button`]}
      </button>
    </div>
  </form>
</div>

<style>
  h2, p {
    margin: 0;
  }

  #add-calendar-event-form-container {
    background-color: var(--color-secondary2);
    min-height: 0;
    height: 100%;
    padding: 1rem 2rem 2rem;
    gap: 0.75rem;
    border-radius: 0;
    max-width: 600px;

    #event-form-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary3);
    }
  }

  #add-calendar-event-top-container {
    position: relative;
    justify-content: flex-start;
    padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--outline-color2);

    #title-wrapper {
      align-items: flex-start;
      max-width: calc(100% - 48px);
      gap: 0.25rem;
      overflow: hidden;
      text-wrap: nowrap;
    }

    p {
      max-width: 100%;
      color: var(--color-highlight1);
      overflow: hidden;
      text-overflow: ellipsis;
    }

    button {
      position: absolute;
      top: 0;
      right: 0;
      margin-left: 1rem;
    }
  }

  #calendar-event-form {
    padding: 0.5rem 0;
    gap: 2rem;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));
  }

  #event-body-wrapper {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 1.5rem;

    #event-body-content {
      align-items: flex-start;
      gap: 0.5rem;
      padding: 0.5rem;
      border-radius: 0.5rem;
      outline: 2px solid var(--outline-color2);

      .primary-input, textarea {
        font-size: 1rem;
      }

      .primary-input {
        outline: none;
        padding: 0.25rem 0;
      }

      > div {
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
        outline: 2px solid var(--outline-color2);
      }
      
      #event-title-container, #event-date-container {
        width: 100%;
        gap: 0.5rem;
      }

      #event-date-container {
        button:hover .span-icon {
          transform: scale(1.15);
        }
      }

      .time-container {
        justify-content: flex-start;
        gap: 0.5rem;

        > div {
          gap: 0.25rem;
          padding: 0;
          border-radius: 0.25rem;

          span {
            line-height: 1em;
            padding-bottom: 2px;
            font-weight: bold;
            user-select: none;
          }
        }

        input {
          width: 2ch;
          text-align: center;
        }
      }

      textarea {
        min-height: 2.25rem;
        max-height: 6rem;
        min-width: 100%;
        max-width: 100%;
        padding: 0.25rem 0.5rem;
        border: none;
        outline: none;
        border-radius: 0.25rem;
        background-color: var(--color-secondary4);
      }
    }
  }

  #add-calendar-event-tags-list {
    justify-content: unset;
    align-items: flex-start;
    align-self: stretch;
    padding: 0.5rem;
    border-radius: 0.5rem;
    outline: 2px solid var(--outline-color2);
    overflow: hidden;

    #event-tag-rows-wrapper {
      justify-content: flex-start;
      align-items: flex-start;
      width: 100%;
      max-height: calc(5 * 40px + 4 * 0.25rem + 0.5rem);
      gap: 0.25rem;
      padding: 0.25rem;
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));

      > p {
        align-self: center;
        width: 100%;
        text-wrap: wrap;
        text-align: center;
      }

      .event-tag-row {
        width: 100%;
        height: 40px;
        justify-content: space-between;
        padding: 0.5rem;
        gap: 0.5rem;
        border-radius: 0.2rem;
        background-color: var(--color-secondary4);

        button {
          width: 1.5rem;
          height: 1.5rem;
          border-radius: 0.25rem;
        }
      }
    }

    #event-form-add-tag-button {
      height: 2rem;
      text-wrap: nowrap;
      
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
