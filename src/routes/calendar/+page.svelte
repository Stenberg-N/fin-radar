<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fade, fly } from "svelte/transition";
  import { onNavigate } from "$app/navigation";
  import { SvelteSet } from "svelte/reactivity";

  import { calendarDays, calendarDate, getCalendarEvents, calendarEvents, deleteCalendarEvent, getCalendarTags } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { CalendarDay, CalendarEvent, CalendarEventWithTag, CalendarTag } from "$lib/types";
  import { moveGutter, handleHorizontalScroll } from "$lib/utils/actions.svelte";
  import { Gutter } from "$lib/classes/gutter.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";
  import { updateUserPrefs, userPrefs } from "$lib/utils/prefsStore";
  import { formatMonthYear, toIsoDate, toYearMonth } from "$lib/utils/helpers.svelte";

  import EventForm from "../../components/calendar/EventForm.svelte";
  import TagsList from "../../components/calendar/TagsList.svelte";
  import SearchBar from "../../components/elements/SearchBar.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";
  import DateBox from "../../components/elements/DateBox.svelte";
  import HorizontalView from "../../components/calendar/HorizontalView.svelte";
  import CalendarGrid from "../../components/calendar/CalendarGrid.svelte";
  import DateSelect from "../../components/elements/DateSelect.svelte";
  import ItemSelect from "../../components/elements/ItemSelect.svelte";

  type MatchOptions = {
    e: CalendarEventWithTag;
    filter: "regex";
    regex: RegExp;
  } | {
    e: CalendarEventWithTag;
    filter: "set";
    regex?: never;
  };
  type EventControl = {
    ariaLabel: string;
    icon: string;
    onClick: (event: CalendarEvent, tags: CalendarTag[]) => void;
  };
  type HoverTarget = { element: "filter" | "sort" | "order" | "add" | "tags" | "nav-back" | "nav-forward" | "event-list" | "calendar-now" | "calendar-selected" | string };
  type ViewType = "calendar" | "horizontal" | "vertical";

  const eventListControls = [
    {
      ariaLabel: "Open form",
      onClick: () => toggleEventFormVisibility(),
      img: "plus.svg"
    },
    {
      ariaLabel: "Open tags",
      onClick: () => isTagsListVisible = !isTagsListVisible,
      img: "tags.svg"
    },
    {
      ariaLabel: "Open filters",
      onClick: () => isFilterVisible = !isFilterVisible,
      img: "filter.svg"
    },
    {
      ariaLabel: "Sort by event property",
      onClick: () => sortData.type === 'date' ? sortData.type = 'text' : sortData.type = 'date',
      img: "bars-sort.svg"
    },
    {
      ariaLabel: "Switch sort order",
      onClick: () => sortData.ascending = !sortData.ascending,
      img: "arrow.svg"
    },
  ];

  const eventControls: EventControl[] = [
    {
      ariaLabel: "Edit event",
      icon: "/edit-pen.svg",
      onClick: (event: CalendarEvent, tags: CalendarTag[]) => {
        isEventFormChanged
          ? sendAlert({
            message: (i18n.t["alert.unsaved-changes"] as string[])[0],
            isTimer: false,
            buttons: true,
            onConfirm: () => editEvent({event, tags}),
            additionalText: [i18n.t["alert.navigate-without-saving"] as string],
          })
          : editEvent({event, tags});
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

  const NAVBAR_WIDTH = $derived($userPrefs.mainPrefs.navBarWidth);
  const EVENT_LIST_WIDTH = $derived($userPrefs.calendarPrefs.eventListWidth);
  let CALENDAR_CONTENT_WIDTH = $state<number>(0);
  const VIEWS = {
    calendar: CalendarGrid,
    horizontal: HorizontalView,
    vertical: null,
  };

  const isEventsListVisible = $derived(EVENT_LIST_WIDTH >= 200);
  let isEventFormVisible = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  let isFilterVisible = $state<boolean>(false);
  let isEventFormChanged = $state<boolean>(false);

  const todayIsodate = toIsoDate(new Date());
  let selectedIsodate = $state<string | null>(null);
  let navDateValue = $state<Date | null>(null);
  let currentView = $state<ViewType>("calendar");
  let direction = $state(1);
  let searchRegex = $state<RegExp | null>(null);
  let selectedFilterTagIds: SvelteSet<number> = new SvelteSet();
  let sortData = $state<{ type: 'date' | 'text', ascending: boolean }>({ type: 'date', ascending: true });

  const hover = new HoverTitle<HoverTarget>();
  const gutter = new Gutter();

  let editedEvent = $state<CalendarEventWithTag | null>(null);
  let frozenIds = $state<Set<number> | null>(null);
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

  let statusBar: HTMLParagraphElement | null = null;
  let openEventFormButton = $state<HTMLButtonElement | null>(null);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);
  let filtersToggleButton = $state<HTMLButtonElement | null>(null);
  let sidebarCalendar = $state<HTMLDivElement | null>(null);
  let navButtonRefs = $state<HTMLButtonElement[]>([]);
  let eventListButtonRefs = $state<HTMLButtonElement[]>([]);

  let eventButtons: Record<string, HTMLButtonElement | null> = $state({});
  let eventButtonRefs = $derived(Object.values(eventButtons).filter((el): el is HTMLButtonElement => el !== null));

  onMount(() => {
    calendarDate.set(new Date());
    getCalendarEvents();
    getCalendarTags();
    statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
  });

  onNavigate(() => {
    if (!statusBar) return;
    statusBar.textContent = null;
  });

  $effect(() => {
    return () => {
      hover.destroy();
      gutter.destroy();
    };
  });

  $effect(() => {
    if ($calendarDate !== null) {
      if (!statusBar) return;
      statusBar.textContent = formatMonthYear($calendarDate);
    }
  });

  $effect(() => {
    if (eventListButtonRefs[0]) openEventFormButton = eventListButtonRefs[0];
    if (eventListButtonRefs[1]) tagsListToggleButton = eventListButtonRefs[1];
    if (eventListButtonRefs[2]) filtersToggleButton = eventListButtonRefs[2];
  });

  $effect(() => {
    const regex = searchRegex;

    if (!regex) {
      frozenIds = null;
      return;
    }

    untrack(() => {
      const source = $calendarEvents;
      let base = source;
      base = selectedFilterTagIds.size > 0 ? source.filter((e) => matches({ e, filter: "set" })) : source;
      frozenIds = new Set(untrack(() => base).filter((e) => matches({ e, filter: "regex", regex })).map((e) => e.event.id));

      if (!base.some((e) => matches({ e, filter: "regex", regex }))) {
        sendAlert({
          message: "alert.search.nothing-found",
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
  const isButtonToggled = (index: number): boolean => {
    return index === 0 && isEventFormVisible || index === 1 && isTagsListVisible || index === 2 && isFilterVisible;
  };
  const matches = (options: MatchOptions) => {
    switch (options.filter) {
      case "regex": return [options.e.event.title, options.e.event.description, options.e.event.isodate].some((val) => options.regex.test(val as string));
      case "set": return options.e.tags.some((tag) => selectedFilterTagIds.has(tag.id));
    }
  };
  /**
   * Compares the two given dates in the format of YYYY-MM and returns 1 or -1 based on the comparison.
   * @param target The current date.
   * @param cmp The date to go to.
   */
  const calculateDelta = (target: string, cmp: string): 1 | -1 => target <= cmp ? 1 : -1;
  const alertUnsavedChanges = (onConfirm: () => void) => {
    sendAlert({
      message: (i18n.t["alert.unsaved-changes"] as string[])[0],
      isTimer: false,
      buttons: true,
      onConfirm: () => onConfirm(),
      additionalText: [i18n.t["alert.navigate-without-saving"] as string],
    });
  };
  
  /***********************************************************************************************************************************/

  const goToDate = (delta: number) => {
    const _delta = navDateValue ? calculateDelta(toYearMonth($calendarDate), toYearMonth(navDateValue)) : delta;
    const defaultDate = new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1);
    const newDate = navDateValue ? navDateValue : defaultDate;

    direction = _delta;
    if (newDate.getTime() !== $calendarDate.getTime()) calendarDate.set(newDate);
    navDateValue = null;
    getCalendarEvents();
    stopEdit();
    selectedFilterTagIds.clear();
  };

  const editEvent = (event: CalendarEventWithTag) => {
    isEventFormVisible = true;
    editedEvent = event;
    selectedIsodate = event.event.isodate;
    if (isEventFormChanged) isEventFormChanged = false;
  };

  const stopEdit = () => {
    isEventFormVisible = false;
    editedEvent = selectedIsodate = null;
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

  const handleEventDelete = async (event: CalendarEvent) => {
    await deleteCalendarEvent(event);
    stopEdit();
  };

  const toggleFilterTag = (tagId: number) => {
    if (!selectedFilterTagIds.has(tagId)) selectedFilterTagIds.add(tagId);
    else selectedFilterTagIds.delete(tagId);
  };

  const toggleEventFormVisibility = () => {
    isEventFormChanged
      ? alertUnsavedChanges(() => { isEventFormVisible = isEventFormChanged = false; editedEvent = selectedIsodate = null;})
      : (isEventFormVisible = !isEventFormVisible, editedEvent = selectedIsodate = null);
  };

  const handleDateChange = (delta: number) => {
    isEventFormChanged
      ? alertUnsavedChanges(() => { goToDate(delta); isEventFormChanged = false; })
      : goToDate(delta);
  };

  const handleCalendarSelect = (day: CalendarDay) => {
    if (isEventFormChanged) {
      alertUnsavedChanges(() => { editedEvent = null; isEventFormChanged = false; selectedIsodate = day.isodate; });
      return;
    }

    editedEvent = null;
    selectedIsodate = day.isodate;
    isEventFormVisible = true;
  };

  const handleCalendarDayHover = (day: CalendarDay) => {
    let element = "";
    switch (day.isodate) {
      case todayIsodate: element = "calendar-now"; break;
      case selectedIsodate: element = "calendar-selected"; break;
      default: element = ""; break;
    }

    hover.enter({ element: element })
  };

  const handleChangeView = (option: ViewType) => {
    if (currentView === option) return;
    currentView = option;
  };
</script>

<div id="calendar-main-container" class="flex column">
  {#if isEventFormVisible}
    {#key `${editedEvent?.event.id}-${selectedIsodate}`}
      <ModalWrapper
        position={{ left: (NAVBAR_WIDTH + EVENT_LIST_WIDTH + 28), top: 100, isDraggable: true }}
        onOutsideClick={() => isEventFormChanged ? {} : stopEdit()}
        ignorableEls={[...navButtonRefs, openEventFormButton, sidebarCalendar, ...eventButtonRefs]}
      >
        <EventForm
          {editedEvent}
          {stopEdit}
          ignorableEls={navButtonRefs}
          updateChanges={(areChanges: boolean) => { isEventFormChanged = areChanges; }}
          {selectedIsodate}
        />
      </ModalWrapper>
    {/key}
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper
      position={{ isDraggable: true }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      onOutsideClick={() => { isTagsListVisible = false; }}
      ignorableEls={[tagsListToggleButton]}
    >
      <TagsList setListVisibility={(state) => { isTagsListVisible = state; }} />
    </ModalWrapper>
  {/if}

  {#if isFilterVisible}
    <ModalWrapper
      position={{ isDraggable: true }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      onOutsideClick={() => { isFilterVisible = false; }}
      ignorableEls={[filtersToggleButton]}
    >
      <div id="calendar-filter-list-container" class="flex column">
        <div id="calendar-filter-list-top-bar" class="flex row">
          <h2>{i18n.t["filter-list-header"]}</h2>
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

  {#if hover.isHovering && hover.target?.element.trim() !== ''}
    {@const content = (() => {
      switch (hover.target?.element) {
        case "add": return i18n.t[isButtonToggled(0) ? "cancel.button" : "calendar.add-event.header"] as string;
        case "filter": return i18n.t["filter-list-header"] as string;
        case "order": return i18n.t["sorted-by.order"] + ((i18n.t["sorted-by.order.options"] as string[])[sortData.ascending ? 0 : 1] as string);
        case "sort": return i18n.t["sorted-by.title"] as string + i18n.t[`${sortData.type === "date" ? "date" : "title"}-input.description`] as string;
        case "tags": return i18n.t["calendar.tags-list-header"] as string;
        case "nav-back": return (i18n.t["month-transition-buttons"] as string[])[0];
        case "nav-forward": return (i18n.t["month-transition-buttons"] as string[])[1];
        case "event-list": return `${EVENT_LIST_WIDTH}px`;
        case "calendar-now": return i18n.t["calendar.current-day"] as string;
        case "calendar-selected": return (i18n.t["calendar.selected-day"] as string[])[editedEvent ? 0 : 1] as string;
        default: return hover.target?.element;
      }
    })()}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={hover.target?.element === "event-list" ? { isContinuousUpdate: true, centerElement: true, moveTop: -50 } : { moveTop: -30, moveLeft: 10 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      outline={{ width: 1, color: 'var(--outline-color2)'}}
      borderRadius={8}
    >
      <p id="hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  <div id="calendar-toolbar" class="primary-toolbar flex row">
    <ItemSelect
      titleTranslationKey="calendar.views.title"
      data={i18n.t["calendar.views"] as string[]}
      icons={["/calendar.svg", "/horizontal-lines.svg", null]}
      onItemSelect={(item) => { handleChangeView(item.toLowerCase() as ViewType); }}
      position={{ isPositionAbsolute: false }}
    />
    <DateSelect
      onDateSelect={(date) => {
        navDateValue = date;
        handleDateChange(1); // Passing down 1 as a dummy argument. Real value is calculated on-site!
      }}
    />
  </div>

  <div id="calendar-content" class="flex row">
    <div id="calendar-sidebar" class="flex column" style="width: {EVENT_LIST_WIDTH}px;">
      {#if isEventsListVisible}
        {@const options = ["add", "tags", "filter", "sort", "order"] as const}
        <div class="calendar-sidebar-top-bar flex">
          <div class="flex">
            {#each eventListControls as button, i (i)}
              <button
                bind:this={eventListButtonRefs[i]}
                aria-label={button.ariaLabel}
                class="button-primary transparent highlight default-corners lower-padding"
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
          <div id="calendar-nav-buttons" class="flex">
            {#each [...Array(2)] as _, i (i)}
              <button
                bind:this={navButtonRefs[i]}
                aria-label={`${i === 0 ? "Previous" : "Next"} month`}
                class="button-primary transparent highlight lower-padding"
                onclick={() => handleDateChange(i === 0 ? -1 : 1)}
                onmouseenter={() => hover.enter({ element: i === 0 ? "nav-back" : "nav-forward" })}
                onmouseleave={(e) => hover.leave(e)}
              >
                <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({i === 0 ? '90deg' : '-90deg'});"></span>
              </button>
            {/each}
          </div>
        </div>

        <div id="calendar-grid" class="flex column">
          <div id="calendar-weekdays">
            {#each (i18n.t["calendar.weekdays"] as string[]) as day}
              <p>{day}</p>
            {/each}
          </div>
          <div bind:this={sidebarCalendar} id="calendar-grid-wrapper">
            {#key toYearMonth($calendarDate)}
              <div id="calendar-days-grid" in:fade={{ duration: 500, easing: cubicInOut }}>
                {#each $calendarDays as day (day.isodate)}
                  <button
                    class="button-primary transparent highlight calendar-day"
                    class:disabled-day={day.enabled === false}
                    class:today={day.isodate === todayIsodate}
                    class:selected-date={day.isodate === selectedIsodate}
                    onclick={() => handleCalendarSelect(day) }
                    onmouseenter={() => handleCalendarDayHover(day)}
                    onmouseleave={(e) => hover.leave(e)}
                  >
                    <span>
                      {day.number}
                    </span>
                  </button>
                {/each}
              </div>
            {/key}
          </div>
        </div>

        <div class="calendar-sidebar-top-bar flex">
          <SearchBar sendRegexToParent={(regex) => { searchRegex = regex; }} />
        </div>

        <div id="calendar-event-wrapper" class="flex column">
          {#each displayEvents as { event, tags } (event.id)}
            {@const startTime = event.start_time ? `${String(Math.floor(event.start_time / 3600)).padStart(2, '0')}:${String(Math.floor(event.start_time % 3600 / 60)).padStart(2, '0')}` : null}
            {@const endTime = event.end_time ? `${String(Math.floor(event.end_time / 3600)).padStart(2, '0')}:${String(Math.floor(event.end_time % 3600 / 60)).padStart(2, '0')}` : null}
            <div role="button" tabindex="0" class="calendar-event flex column" in:fly={{ x: -EVENT_LIST_WIDTH, duration: 400, easing: cubicInOut }}>
              <div class="flex">
                <DateBox date={event.isodate} theme="darker" noPadding={true} />
                <div class="event-content flex column">
                  <p
                    onmouseenter={() => hover.enter({ element: event.title })}
                    onmouseleave={(e) => hover.leave(e)}
                  >
                    {event.title}
                  </p>
                  {#if startTime && endTime}
                    <div class="event-times-wrapper flex row">
                      {#each [startTime, endTime] as time, i (i)}
                        <div class="event-time-container flex row">
                          <span role="contentinfo" class="span-icon img-small" style="mask-image: url('/{i === 0 ? 'clock' : 'hourglass-end'}.svg');"
                            onmouseenter={() => hover.enter({ element: i18n.t[`calendar.${i === 0 ? 'start' : 'end'}-time.description`] as string })}
                            onmouseleave={(e) => hover.leave(e)}
                          ></span>
                          <p>{time}</p>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
              <div class="event-controls flex row">
                {#each eventControls as button, i (i)}
                  {@const key = `${event.id}-${i}`}
                  <button
                    bind:this={eventButtons[key]}
                    aria-label={button.ariaLabel}
                    class="button-primary transparent highlight default-corners lower-padding"
                    onclick={() => button.onClick(event, tags)}
                  >
                    <span class="span-icon img-small-medium" style="mask-image: url('{button.icon}');"></span>
                  </button>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div role="slider" aria-valuenow={EVENT_LIST_WIDTH} tabindex="0" class="resize-gutter-default flex row" class:highlight={gutter.isHovered}
      use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("calendarPrefs", "eventListWidth", newWidth); },  min: 0, max: 600, threshold: { at: 200, jumpTo: 0 } }}
      onmouseenter={() => { hover.enter({ element: "event-list" }); gutter.enter(); }}
      onmouseleave={(e) => { hover.leave(e); gutter.leave(); }}
    ></div>

    <div id="calendar-main-content" bind:clientWidth={CALENDAR_CONTENT_WIDTH} class="flex column" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
      {#if currentView}
        {@const View = VIEWS[currentView]}
        {#if View}
          <View
            calendarDate={$calendarDate}
            calendarDays={$calendarDays}
            calendarEvents={$calendarEvents}
            {direction}
            parentWidth={CALENDAR_CONTENT_WIDTH}
            {todayIsodate}
          />
        {:else}
          <p>Work in progress / Kehityksen alla</p>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  #calendar-main-container,
  #calendar-content {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #calendar-main-container {
    #hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }
  }

  #calendar-content {
    height: 100%;
    gap: 0.25rem;

    .resize-gutter-default {
      justify-content: center;
    }

    > div {
      height: 100%;
      justify-content: flex-start;
    }
  }

  #calendar-main-content {
    flex: 1 1 auto;
    border-left: 1px solid var(--outline-color2);
    overflow-x: auto;
  }

  #calendar-toolbar {
    gap: 1rem;
  }

  #calendar-content #calendar-sidebar {
    flex-shrink: 0;
    justify-content: flex-start;
    align-items: flex-start;
    border-right: 1px solid var(--outline-color2);
    will-change: width;
    background-color: var(--color-secondary1);

    .calendar-sidebar-top-bar {
      justify-content: space-between;
      width: 100%;
      padding: 0.25rem;
      border-bottom: 1px solid var(--outline-color2);

      > div {
        gap: 0.25rem;
      }

      button {
        &.toggled {
          background-color: var(--color-highlight2);
        }
      }
    }

    #calendar-grid {
      align-items: unset;
      width: 100%;
      padding: 0 0.25rem 0.25rem;
      border-bottom: 1px solid var(--outline-color2);

      #calendar-weekdays {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 1px;

        p {
          margin: 0;
          font-size: 14px;
          text-align: center;
          user-select: none;
        }
      }
      
      #calendar-grid-wrapper {
        overflow: hidden;
      }
      
      #calendar-days-grid {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 1px;

        .calendar-day {
          padding: 0;
          border-radius: 0;
          border: 1px solid var(--outline-color1);

          &.today, &.selected-date {
            background-color: var(--color-highlight2);
            font-weight: bold;
          }

          &.today {
            border: 1px solid var(--color-highlight3);
          }

          &.selected-date {
            border: 1px solid var(--color-highlight1);
          }

          &.disabled-day span {
            opacity: 0.5;
          }
        }
      }
    }

    #calendar-event-wrapper {
      justify-content: flex-start;
      width: 100%;
      height: 100%;
      overflow-y: auto;

      div.calendar-event {
        align-items: flex-end;
        width: 100%;
        gap: 0.5rem;
        padding: 0.5rem;
        border-bottom: 1px solid var(--outline-color2);

        div {
          justify-content: flex-start;
          width: 100%;
        }

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

          .event-content {
            align-items: flex-start;
            width: 100%;
            overflow: hidden;

            .event-times-wrapper {
              gap: 0.5rem;
            }

            > p {
              max-width: 100%;
              font-weight: bold;
            }
          }

          .event-time-container {
            width: unset;
            gap: 0.25rem;
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

  #calendar-filter-list-container {
    padding: 1rem 1.5rem;
    gap: 1rem;
    background-color: var(--color-secondary2);

    #calendar-filter-list-top-bar {
      width: 100%;
      justify-content: space-between;
      min-width: 240px;
      padding-bottom: 1rem;
      border-bottom: 2px solid var(--outline-color2);
    }

    #calendar-filters-wrapper {
      width: 100%;
      gap: 0.25rem;

      label {
        width: 100%;
        justify-content: flex-start;
        padding: 0.5rem;
        gap: 0.75rem;
        background-color: var(--color-secondary3);
        border-radius: 0.25rem;

        &:hover {
          cursor: pointer;
        }

        span {
          user-select: none;
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
</style>