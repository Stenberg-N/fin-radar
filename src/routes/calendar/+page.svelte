<script lang="ts">
  import { onMount } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fly } from "svelte/transition";
  import { onNavigate } from "$app/navigation";
  import { SvelteSet } from "svelte/reactivity";

  import { calendarDays, calendarDate, getCalendarEvents, calendarEvents, deleteCalendarEvent, getCalendarTags } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n/i18n";
  import { viewport } from "$lib/viewport";
  import type { CalendarEvent, CalendarEventWithTag, CalendarTag } from "$lib/types";
  import { handleClickOutside, capitalizeString } from "$lib/actions";

  import EventForm from "../../components/calendar/EventForm.svelte";
  import TagsList from "../../components/calendar/TagsList.svelte";
  import SearchBar from "../../components/SearchBar.svelte";
  import ModalWrapper from "../../components/ModalWrapper.svelte";

  let isEventsListVisible = $state<boolean>(true);
  let isEventFormVisible = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  let isFilterVisible = $state<boolean>(false);
  const monthTransitionWidth = $derived($viewport.width / 2);
  let direction = $state(1);
  const todayIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`)(new Date());
  const yearMonthString = $derived(((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)($calendarDate));
  let searchRegex = $state<RegExp | null>(null);
  let selectedFilterTagIds = $state<SvelteSet<number>>(new SvelteSet());
  let sortData = $state<{ type: 'date' | 'text', ascending: boolean}>({ type: 'date', ascending: true });

  let editedEvent = $state<CalendarEventWithTag | null>(null);
  const displayEvents = $derived.by(() => {
    const base = searchRegex !== null
    ? $calendarEvents.filter(obj => [obj.event.title, obj.event.description, obj.event.isodate].some((val) => searchRegex?.test(val as string)))
    : selectedFilterTagIds.size > 0
      ? $calendarEvents.filter(obj => obj.tags.some((tag) => selectedFilterTagIds.has(tag.id)))
      : $calendarEvents

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
    if ($calendarDate !== null) {
      const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
      statusBar.textContent = `${($t["calendar.monthnames"] as string[])[$calendarDate.getMonth()]}, ${$calendarDate.getFullYear()}`;
    }
  });

  $effect(() => {
    if (eventListButtonRefs[0]) openEventFormButton = eventListButtonRefs[0];
    if (eventListButtonRefs[1]) tagsListToggleButton = eventListButtonRefs[1];
    if (eventListButtonRefs[2]) filtersToggleButton = eventListButtonRefs[2];
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
    {#key editedEvent?.event.id}
      <ModalWrapper options={{ position: { left: 304, top: 60, isPositionAbsolute: true } }}>
        <EventForm options={{ editedEvent, stopEdit: stopEdit, navButtonRefs, calendarEventRefs, openEventFormButton }} />
      </ModalWrapper>
    {/key}
  {/if}

  {#if isTagsListVisible}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <TagsList options={{
        setListVisibility: (state) => { isTagsListVisible = state; },
        tagsListToggleButton,
        isTagsListVisible,
      }}
      />
    </ModalWrapper>
  {/if}

  {#if isFilterVisible}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" }}}>
      <div id="calendar-filter-list-container" class="flex column" use:handleClickOutside={{ onOutsideClick: () => isFilterVisible = false, additionalElements: [filtersToggleButton] }}>
        <div id="calendar-filter-list-top-bar" class="flex row">
          <h2>{$t["calendar.filter-list-header"]}</h2>
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

  <div id="calendar-toolbar" class="primary-toolbar flex row">
    <div id="calendar-nav-buttons" class="flex row">
      {#each [...Array(2)] as _, i (i)}
        <button bind:this={navButtonRefs[i]} title={($t["month-transition-buttons"] as string[])[i] as string} class="button-primary transparent highlight {i === 1 && 'static'}" onclick={() => goToMonth(i === 0 ? -1 : 1)}>
          <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({i === 0 ? '90deg' : '-90deg'});"></span>
        </button>
      {/each}
    </div>
  </div>

  <div id="calendar-content" class="flex row">
    <div id="calendar-event-container" class="flex column" style="width: {isEventsListVisible ? '300px' : '41px'};">
      <div class="calendar-event-container-top-bar flex row" style="border-bottom: {isEventsListVisible ? '1px solid var(--outline-color1)' : ''};">
        {#if isEventsListVisible}
          <SearchBar options={{ sendRegexToParent: (regex) => { searchRegex = regex; }, mirrorSearchBar: true }} />
        {/if}
        <button aria-label="Toggle event list" class="button-primary transparent highlight static" onclick={() => isEventsListVisible = !isEventsListVisible}>
          <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({isEventsListVisible ? '90deg' : '-90deg'});"></span>
        </button>
      </div>

      {#if isEventsListVisible}
        <div class="calendar-event-container-top-bar sub-bar flex row" style="border-bottom: {isEventsListVisible ? '1px solid var(--outline-color1)' : ''};">
          {#each eventListControls as button, i (i)}
            <button
              bind:this={eventListButtonRefs[i]}
              aria-label={button.ariaLabel}
              class="button-primary transparent highlight static sharper-corners"
              class:toggled={isButtonToggled(i)}
              onclick={button.onClick}
              title={i === 3 ? $t["sorted-by.title"] + capitalizeString(sortData.type) : i === 4 ? $t["sorted-by.order"] + (($t["sorted-by.order.options"] as string[])[sortData.ascending ? 0 : 1] as string) : null}
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
            <div role="button" tabindex="0" bind:this={calendarEventRefs[i]} class="calendar-event flex column" in:fly={{ x: -300, duration: 400, easing: cubicInOut }}
              onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); editEvent({event, tags}) }}}
              onclick={() => editEvent({event, tags})}
            >
              <p>{event.title}</p>
              <p>{event.isodate}</p>
              <button 
                onclick={(e) => {
                  e.stopPropagation();
                  sendAlert({
                    message: "alert.delete-calendar-event.confirmation",
                    isTimer: false,
                    buttons: true,
                    additionalText: [event.title],
                    onConfirm: () => handleEventDelete(event)
                  })
                }}
              >
                DEL
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div id="calendar-days-container" class="flex column">
      <div id="calendar-weekdays">
        {#each ($t["calendar.weekdays"] as string[]) as weekDay (weekDay)}
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
                  <span class="event-indicator" title={$lang === 'en' ? "You have events on this day" : "Sinulla on tapahtumia tässä päivässä"}></span>
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

  #calendar-main-container,
  #calendar-content,
  #calendar-content > div {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #calendar-content {
    height: calc(100% - 3.5rem);
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
      gap: 6px;
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
    transition: width 0.2s;
    will-change: width;

    .calendar-event-container-top-bar {
      justify-content: space-between;
      width: 100%;
      gap: 6px;
      padding: 0.25rem;

      &.sub-bar {
        justify-content: flex-start;
      }

      button {
        &.sharper-corners {
          border-radius: 0.25rem;
        }

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
        background-color: var(--color-secondary1);
        border-bottom: 1px solid var(--outline-color1);
      }
      div.calendar-event:hover {
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.8);
        z-index: 1;
        cursor: pointer;
      }
    }
  }

  #calendar-days-container {
    > div:not(#calendar-weekdays) {
      height: 100%;
    }

    #calendar-weekdays {
      text-align: center;
      border-bottom: 1px solid var(--outline-color1);
      
      > p {
        margin: 0;
        user-select: none;
      }
    }
  }

  #calendar-grid, #calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    width: 100%;
  }

  #calendar-grid-wrapper {
    overflow: hidden;
    position: relative;
    width: 100%;
    height: 100%;
  }

  #calendar-grid {
    position: absolute;
    inset: 0;

    > div {
      justify-content: space-between;
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
      border-right: 1px solid #222;
    }

    > div:not(:nth-last-child(-n+7)) {
      border-bottom: 1px solid #222;
    }
  }

  @keyframes pulse {
    0% {
      transform: scale(1);
      opacity: 0.8;
    }
    70% {
      transform: scale(2.25);
      opacity: 0;
    }
    100% {
      transform: scale(2.25);
      opacity: 0;
    }
  }
</style>