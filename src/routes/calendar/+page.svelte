<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fly } from "svelte/transition";
  import { onNavigate } from "$app/navigation";
  import { SvelteSet } from "svelte/reactivity";

  import { calendarDays, calendarDate, getCalendarEvents, calendarEvents, deleteCalendarEvent, getCalendarTags } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { viewport } from "$lib/viewport";
  import type { CalendarEvent, CalendarEventWithTag, CalendarTag } from "$lib/types";
  import { capitalizeString, Gutter, HoverTitle, moveGutter } from "$lib/actions.svelte";
  import { updateUserPrefs, userPrefs } from "$lib/prefsStore";

  import EventForm from "../../components/calendar/EventForm.svelte";
  import TagsList from "../../components/calendar/TagsList.svelte";
  import SearchBar from "../../components/SearchBar.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import DateBox from "../../components/calendar/DateBox.svelte";

  type MatchOptions = {
    e: CalendarEventWithTag;
    filter: "regex";
    regex: RegExp;
  } | {
    e: CalendarEventWithTag;
    filter: "set";
    regex?: never;
  };

  type HoverTarget = { element: "filter" | "sort" | "order" | "add" | "tags" | "nav-back" | "nav-forward" | "event-list" | string };
  type EventControl = {
    ariaLabel: string;
    icon: string;
    onClick: (event: CalendarEvent, tags: CalendarTag[]) => void;
  };

  const NAVBAR_WIDTH = $derived($userPrefs.mainPrefs.navBarWidth);
  const EVENT_LIST_WIDTH = $derived($userPrefs.calendarPrefs.eventListWidth);
  const isEventsListVisible = $derived(EVENT_LIST_WIDTH >= 255);
  let isEventFormVisible = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  let isFilterVisible = $state<boolean>(false);
  const monthTransitionWidth = $derived($viewport.width / 2);
  let direction = $state(1);
  const todayIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`)(new Date());
  const yearMonthString = $derived(((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)($calendarDate));
  let searchRegex = $state<RegExp | null>(null);
  let selectedFilterTagIds = $state<SvelteSet<number>>(new SvelteSet());
  let sortData = $state<{ type: 'date' | 'text', ascending: boolean }>({ type: 'date', ascending: true });

  const hover = new HoverTitle<HoverTarget>();
  const gutter = new Gutter();

  let editedEvent = $state<CalendarEventWithTag | null>(null);
  let frozenIds = $state<SvelteSet<number> | null>(null);
  const displayEvents = $derived.by(() => {
    const source = $calendarEvents;
    let base = source;

    if (frozenIds && selectedFilterTagIds.size > 0) {
      base = source.filter((e) => frozenIds!.has(e.event.id) && matches({ e, filter: "set" }));
    } else if (frozenIds) {
      base = source.filter((e) => frozenIds!.has(e.event.id));
    } else if (selectedFilterTagIds.size > 0) {
      base = source.filter((e) => matches({ e, filter: "set" }));
    }

    return sortEvents(base);
  });
  const displayEventsTags = $derived.by(() => {
    let tagMap: Map<number, CalendarTag> = new Map();
    for (const content of displayEvents) {
      for (const tag of content.tags) {
        if (!tagMap.has(tag.id)) tagMap.set(tag.id, tag);
      }
    }
    return [...tagMap.values()];
  });
  let filterTags = $derived(displayEventsTags.map((tag) => ({
    tag,
    isChecked: selectedFilterTagIds.has(tag.id),
  })));

  const eventListControls = [
    { ariaLabel: "Open form", onClick: () => toggleEventFormVisibility(), img: "plus.svg" },
    { ariaLabel: "Open tags", onClick: () => isTagsListVisible = !isTagsListVisible, img: "tags.svg" },
    { ariaLabel: "Open filters", onClick: () => isFilterVisible = !isFilterVisible, img: "filter.svg" },
    { ariaLabel: "Sort by event property", onClick: () => sortData.type === 'date' ? sortData.type = 'text' : sortData.type = 'date', img: "bars-sort.svg" },
    { ariaLabel: "Switch sort order", onClick: () => sortData.ascending = !sortData.ascending, img: "arrow.svg" },
  ];

  const eventControls: EventControl[] = [
    {
      ariaLabel: "Edit event",
      icon: "/edit-pen.svg",
      onClick: (event: CalendarEvent, tags: CalendarTag[]) => {
        editEvent({event, tags});
      }
    },
    {
      ariaLabel: "Delete event",
      icon: "/trash-can.svg",
      onClick: (event: CalendarEvent) => {
        sendAlert({
          message: "alert.delete-calendar-event.confirmation",
          isTimer: false,
          buttons: true,
          additionalText: [event.title],
          onConfirm: () => handleEventDelete(event)
        });
      }
    },
  ];

  let openEventFormButton = $state<HTMLButtonElement | null>(null);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);
  let filtersToggleButton = $state<HTMLButtonElement | null>(null);
  let navButtonRefs = $state<HTMLButtonElement[]>([]);
  let calendarEventRefs = $state<HTMLDivElement[]>([]);
  let eventListButtonRefs = $state<HTMLButtonElement[]>([]);

  onMount(() => {
    calendarDate.set(new Date());
    getCalendarEvents(yearMonthString);
    getCalendarTags();
  });

  onNavigate(() => {
    const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
    statusBar.textContent = null;
  });

  $effect(() => {
    hover.destroy();
    gutter.destroy();
  });

  $effect(() => {
    if ($calendarDate !== null) {
      const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
      statusBar.textContent = `${(i18n.t["calendar.monthnames"] as string[])[$calendarDate.getMonth()]}, ${$calendarDate.getFullYear()}`;
    }
  });

  $effect(() => {
    if (eventListButtonRefs[0]) openEventFormButton = eventListButtonRefs[0];
    if (eventListButtonRefs[1]) tagsListToggleButton = eventListButtonRefs[1];
    if (eventListButtonRefs[2]) filtersToggleButton = eventListButtonRefs[2];
  });

  $effect(() => {
    const regex = searchRegex;
    frozenIds = regex ? new SvelteSet(untrack(() => $calendarEvents).filter((e) => matches({ e, filter: "regex", regex })).map((e) => e.event.id)) : null;

    if (!regex) {
      return;
    }

    untrack(() => {
      const source = $calendarEvents;
      let base = source;
      base = selectedFilterTagIds.size > 0 ? source.filter((e) => matches({ e, filter: "set" })) : source;

      if (!base.some((e) => matches({ e, filter: "regex", regex }))) {
        sendAlert({
          message: "test",
          isTimer: true,
          buttons: false,
        });
      }
    });
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const toggleEventFormVisibility = () => {
    isEventFormVisible = !isEventFormVisible;
    editedEvent = null;
  };
  const isButtonToggled = (index: number): boolean => {
    return index === 0 && isEventFormVisible || index === 1 && isTagsListVisible || index === 2 && isFilterVisible;
  };
  const matches = (options: MatchOptions) => {
    switch (options.filter) {
      case "regex": return [options.e.event.title, options.e.event.description, options.e.event.isodate].some((val) => options.regex.test(val as string));
      case "set": return options.e.tags.some((tag) => selectedFilterTagIds.has(tag.id));
    }
  };
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => {
    direction = delta;
    calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1));
    getCalendarEvents(yearMonthString);
    stopEdit();
    selectedFilterTagIds.clear();
  };

  const editEvent = (event: CalendarEventWithTag) => {
    isEventFormVisible = true;
    editedEvent = event;
  };

  const stopEdit = () => {
    isEventFormVisible = false;
    editedEvent = null;
  };

  const sortEvents = (data: CalendarEventWithTag[]) => {
    return [...data].sort((a, b) => {
      let order = 0;

      switch (sortData.type) {
        case "date": order = new Date(a.event.isodate).getTime() - new Date(b.event.isodate).getTime(); break;
        case "text": order = a.event.title.localeCompare(b.event.title); break;
      }

      return sortData.ascending ? order : -order;
    });
  };

  const handleEventDelete = (event: CalendarEvent) => {
    deleteCalendarEvent(event);
    stopEdit();
  };

  const toggleFilterTag = (tagId: number) => {
    if (!selectedFilterTagIds.has(tagId)) selectedFilterTagIds.add(tagId);
    else selectedFilterTagIds.delete(tagId);
  };
</script>

<div id="calendar-main-container" class="flex column">
  {#if isEventFormVisible}
    <ModalWrapper options={{
      position: { left: (NAVBAR_WIDTH + EVENT_LIST_WIDTH + 20), top: 116, isDraggable: true },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      onOutsideClick: stopEdit,
      ignorableEls: [...navButtonRefs, ...calendarEventRefs, openEventFormButton]
      }}
    >
      <EventForm options={{
        editedEvent,
        stopEdit: stopEdit,
        ignorableEls: navButtonRefs,
      }}
      />
    </ModalWrapper>
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper options={{
      position: { isDraggable: true },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      onOutsideClick: () => { isTagsListVisible = false; },
      ignorableEls: [tagsListToggleButton],
      }}
    >
      <TagsList options={{
        setListVisibility: (state) => { isTagsListVisible = state; },
      }}
      />
    </ModalWrapper>
  {/if}

  {#if isFilterVisible}
    <ModalWrapper options={{
      position: { isDraggable: true },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      onOutsideClick: () => { isFilterVisible = false; },
      ignorableEls: [filtersToggleButton],
      }}
    >
      <div id="calendar-filter-list-container" class="flex column">
        <div id="calendar-filter-list-top-bar" class="flex row">
          <h2>{i18n.t["calendar.filter-list-header"]}</h2>
          <button aria-label="Close filter list" class="button-primary transparent highlight static" onclick={() => isFilterVisible = false}>
            <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
          </button>
        </div>
        <div id="calendar-filters-wrapper" class="flex column">
        {#each filterTags as {tag, isChecked} (tag.id)}
          <label class="flex row">
            <input type="checkbox" checked={isChecked} onchange={() => toggleFilterTag(tag.id)} />
            <span>{tag.name}</span>
          </label>
        {/each}
        </div>
      </div>
    </ModalWrapper>
  {/if}

  {#if hover.isHovering}
    {@const content = (() => {
      switch (hover.target?.element) {
        case "add": return i18n.t[isButtonToggled(0) ? "cancel.button" : "calendar.add-event.header"];
        case "filter": return i18n.t["calendar.filter-list-header"];
        case "order": return i18n.t["sorted-by.order"] + ((i18n.t["sorted-by.order.options"] as string[])[sortData.ascending ? 0 : 1] as string);
        case "sort": return i18n.t["sorted-by.title"] + capitalizeString(sortData.type);
        case "tags": return i18n.t["calendar.tags-list-header"];
        case "nav-back": return (i18n.t["month-transition-buttons"] as string[])[0];
        case "nav-forward": return (i18n.t["month-transition-buttons"] as string[])[1];
        case "event-list": return `${EVENT_LIST_WIDTH}px`;
        default: return hover.target?.element;
      }
    })()}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      options={{
        position: hover.target?.element === "event-list" ? { isContinuousUpdate: true, centerElement: true, moveTop: -50 } : { moveTop: -30, moveLeft: 10 },
        transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
        outline: { width: 1, color: 'var(--outline-color1)'},
        borderRadius: 8,
      }}
    >
      <p id="hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  <div id="calendar-toolbar" class="primary-toolbar flex row">
    <div id="calendar-nav-buttons" class="flex row">
      {#each [...Array(2)] as _, i (i)}
        <button
          bind:this={navButtonRefs[i]}
          aria-label={`${i === 0 ? "Previous" : "Next"} month`}
          class="button-primary transparent highlight {i === 1 && 'static'}"
          onclick={() => goToMonth(i === 0 ? -1 : 1)}
          onmouseenter={() => hover.enter({ element: i === 0 ? "nav-back" : "nav-forward" })}
          onmouseleave={(e) => hover.leave(e)}
        >
          <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({i === 0 ? '90deg' : '-90deg'});"></span>
        </button>
      {/each}
    </div>
  </div>

  <div id="calendar-content" class="flex row">
    <div id="calendar-event-container" class="flex column" style="width: {EVENT_LIST_WIDTH}px;">
      <div class="calendar-event-container-top-bar flex row" style="border-bottom: {isEventsListVisible ? '1px solid var(--outline-color1)' : ''};">
        {#if isEventsListVisible}
          <SearchBar options={{ sendRegexToParent: (regex) => { searchRegex = regex; } }} />
        {/if}
      </div>

      {#if isEventsListVisible}
        {@const options = ["add", "tags", "filter", "sort", "order"] as const}
        <div class="calendar-event-container-top-bar sub-bar flex row" style="border-bottom: {isEventsListVisible ? '1px solid var(--outline-color1)' : ''};">
          {#each eventListControls as button, i (i)}
            <button
              bind:this={eventListButtonRefs[i]}
              aria-label={button.ariaLabel}
              class="button-primary transparent highlight static default-corners"
              class:toggled={isButtonToggled(i)}
              onclick={button.onClick}
              onmouseenter={() => hover.enter({ element: options[i] })}
              onmouseleave={(e) => hover.leave(e)}
            >
              <span
                class="span-icon img-small"
                style="
                  mask-image: url({button.img});
                  transform: {i === 0 ? `rotate(${isEventFormVisible ? '45deg' : '0'})` : i === 4 ? `rotate(${sortData.ascending ? '180deg' : '0'})` : ''};
                  transition: transform 0.1s;"
              ></span>
            </button>
          {/each}
        </div>
      {/if}

      {#if isEventsListVisible}
        <div id="calendar-event-wrapper" class="flex column">
          {#each displayEvents as { event, tags }, i (event.id)}
            {@const startTime = event.start_time ? `${String(Math.floor(event.start_time / 3600)).padStart(2, '0')}:${String(Math.floor(event.start_time % 3600 / 60)).padStart(2, '0')}` : null}
            {@const endTime = event.end_time ? `${String(Math.floor(event.end_time / 3600)).padStart(2, '0')}:${String(Math.floor(event.end_time % 3600 / 60)).padStart(2, '0')}` : null}
            <div role="button" tabindex="0" bind:this={calendarEventRefs[i]} class="calendar-event flex column" in:fly={{ x: -300, duration: 400, easing: cubicInOut }}>
              <div class="event-content flex column">
                <div class="flex">
                  <DateBox options={{ date: event.isodate, bgColor: "darker", noPadding: true }} />
                  {#if startTime && endTime}
                    {#each [startTime, endTime] as time, i (i)}
                      <div class="event-time-container flex row">
                        <span role="contentinfo" class="span-icon img-small" style="mask-image: url('/{i === 0 ? 'clock' : 'hourglass-end'}.svg');"
                          onmouseenter={() => hover.enter({ element: i18n.t[`calendar.${i === 0 ? 'start' : 'end'}-time.description`] as string })}
                          onmouseleave={(e) => hover.leave(e)}
                        ></span>
                        <p>{time}</p>
                      </div>
                    {/each}
                  {/if}
                </div>
                <div class="event-title-container">
                  <p
                    onmouseenter={() => hover.enter({ element: event.title })}
                    onmouseleave={(e) => hover.leave(e)}
                  >
                    {event.title}
                  </p>
                </div>
                <div class="event-controls flex row">
                  {#each eventControls as button, i (i)}
                    <button
                      aria-label={button.ariaLabel}
                      class="button-primary transparent highlight default-corners lower-padding"
                      onclick={() => button.onClick(event, tags)}
                    >
                      <span class="span-icon img-small-medium" style="mask-image: url('{button.icon}');"></span>
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div role="slider" aria-valuenow={EVENT_LIST_WIDTH} tabindex="0" class="resize-gutter-default flex row" class:highlight={gutter.isHovered}
      use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("calendarPrefs", "eventListWidth", newWidth); },  min: 40, max: 400, threshold: { at: 255, jumpTo: 40 } }}
      onmouseenter={() => { hover.enter({ element: "event-list" }); gutter.enter(); }}
      onmouseleave={(e) => { hover.leave(e); gutter.leave(); }}
    ></div>

    <div id="calendar-days-container" class="flex column">
      <div id="calendar-weekdays">
        {#each (i18n.t["calendar.weekdays"] as string[]) as weekDay (weekDay)}
          <p>{weekDay}</p>
        {/each}
      </div>
      <div id="calendar-grid-wrapper">
        {#key `${$calendarDate.getFullYear()}-${$calendarDate.getMonth()}`}
          <div id="calendar-grid" in:fly={{ x: direction * monthTransitionWidth, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * -monthTransitionWidth, duration: 300, easing: cubicInOut }}>
            {#each $calendarDays as day (day.date)}
              <div class="flex row" class:disabled-day={!day.enabled}>
                <p class:today={day.isodate === todayIsodate}>
                  {day.number}
                </p>
                {#if $calendarEvents.some(obj => obj.event.isodate === day.isodate)}
                  <span class="event-indicator" title={i18n.lang === 'en' ? "You have events on this day" : "Sinulla on tapahtumia tässä päivässä"}></span>
                {/if}
              </div>
            {/each}
          </div>
        {/key}
      </div>
    </div>
  </div>
</div>

<style>
  .disabled-day > * {
    opacity: 0.4;
  }

  .today {
    background-color: var(--color-highlight1);
    border-radius: 50%;
    font-weight: bold;
  }

  #calendar-main-container {
    #hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary1);
    }
  }

  #calendar-main-container,
  #calendar-content {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #calendar-content {
    height: calc(100% - 3.5rem);
    gap: 0.25rem;

    .resize-gutter-default {
      justify-content: center;
    }

    > div {
      height: 100%;
      justify-content: flex-start;
    }
  }

  #calendar-filter-list-container {
    padding: 1rem 1.5rem;
    gap: 1rem;
    background-color: var(--color-secondary1);

    #calendar-filter-list-top-bar {
      width: 100%;
      justify-content: space-between;
      min-width: 240px;
      padding-bottom: 1rem;
      border-bottom: 2px solid var(--outline-color1);
    }

    #calendar-filters-wrapper {
      width: 100%;
      gap: 0.25rem;

      label {
        width: 100%;
        justify-content: flex-start;
        padding: 0.5rem;
        gap: 0.75rem;
        background-color: var(--color-secondary2);
        border-radius: 0.25rem;

        &:hover {
          cursor: pointer;
        }
      }

      input[type="checkbox"] {
        margin: 0;
        width: 14px;
        height: 14px;
      }
    }

    h2 {
      margin: 0;
    }
  }

  #calendar-toolbar {
    #calendar-nav-buttons {
      gap: 0.25rem;
    }

    > div:not(:first-of-type) {
      gap: 0.75rem;
    }
  }

  #calendar-content #calendar-event-container {
    flex-shrink: 0;
    justify-content: flex-start;
    align-items: flex-start;
    border-right: 1px solid var(--outline-color1);
    will-change: width;

    .calendar-event-container-top-bar {
      justify-content: space-between;
      width: 100%;
      gap: 0.25rem;
      padding: 0.25rem;

      &.sub-bar {
        justify-content: flex-start;
      }

      button {
        &.toggled {
          background-color: var(--color-highlight2);
        }
      }
    }

    #calendar-event-wrapper {
      justify-content: flex-start;
      width: 100%;
      height: 100%;
      overflow-y: auto;

      div.calendar-event {
        width: 100%;
        gap: 0.5rem;
        padding: 0.5rem;
        border-bottom: 1px solid var(--outline-color1);
        background-color: var(--color-secondary1);

        div {
          justify-content: flex-start;
          width: 100%;
        }

        > div {
          border-radius: 0.5rem;
          background-color: var(--color-primary2);
        }

        .event-content {
          align-items: flex-end;
          padding: 0.75rem;
          gap: 1rem;

          > div {
            gap: 1rem;

            &.event-controls {
              max-width: 100%;
              width: unset;
              gap: 0.25rem;

              button {
                height: unset;
              }
            }

            &.event-title-container {
              width: 100%;
              padding: 0.25rem;
              background-color: var(--color-secondary1);
              outline: 1px solid var(--outline-color1);
              border-radius: 0.5rem;
              overflow: hidden;
            }

            .event-time-container {
              width: unset;
              gap: 0.25rem;
              padding: 0.25rem 0.5rem;
              border-radius: 0.5rem;
              background-color: var(--color-secondary1);
              outline: 1px solid var(--outline-color1);
              overflow: hidden;
            }
          }

          p {
            margin: 0;
            font-size: 14px;
            text-wrap: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
          }
        }
      }
    }
  }

  #calendar-days-container {
    flex: 1 1 auto;
    border-left: 1px solid var(--outline-color1);
    overflow-x: auto;

    > div:not(#calendar-weekdays) {
      height: 100%;
    }

    #calendar-weekdays {
      text-align: center;
      
      > p {
        margin: 0;
        user-select: none;
        min-width: 8rem;
        border-bottom: 1px solid var(--outline-color1);
      }
    }
  }

  #calendar-grid-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
  }

  #calendar-grid, #calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    width: 100%;
  }

  #calendar-grid {
    position: absolute;
    inset: 0;

    > div {
      justify-content: space-between;
      min-width: 8rem;
      padding: 6px;

      p {
        align-self: flex-start;
        margin: 0;
        padding: 6px;
        height: 32px;
        width: 32px;
        font-size: 14px;
        line-height: normal;
        text-align: center;
      }

      span.event-indicator {
        position: relative;
        align-self: flex-start;
        width: 0.5rem;
        height: 0.5rem;
        border-radius: 50%;
        background-color: var(--color-highlight1);
      }
      span.event-indicator::after {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 2px solid var(--color-highlight1);
        animation: pulse 1.5s ease-out infinite;
      }
    }

    > div:hover {
      background-color: var(--color-secondary2);
      cursor: pointer;
    }

    > div:not(:nth-child(7n)) {
      border-right: 1px solid var(--outline-color1-dimmed);
    }

    > div:not(:nth-last-child(-n+7)) {
      border-bottom: 1px solid var(--outline-color1-dimmed);
    }
  }
</style>