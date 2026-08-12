<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fly, slide } from "svelte/transition";
  import { onNavigate } from "$app/navigation";

  import { calendarDays, calendarDate, getCalendarEvents, calendarEvents, deleteCalendarEvent } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { viewport } from "$lib/viewport";
  import type { CalendarEvent } from "$lib/types";

  import EventForm from "../../components/calendar/EventForm.svelte";

  let isEventsListVisible = $state<boolean>(true);
  let isEventFormVisible = $state<boolean>(false);
  let isSearchVisible = $state<boolean>(false);
  const monthTransitionWidth = $derived($viewport.width / 2);
  let direction = $state(1);
  const todayIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`)(new Date());
  const yearMonthString = $derived(((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)($calendarDate));
  let searchable = $state<string | null>(null);
  let searchRegex = $state<RegExp | null>(null);

  const displayEvents = $derived(searchRegex !== null ? $calendarEvents.filter(e => [e.title, e.description].some((val) => searchRegex?.test(val as string))) : $calendarEvents);
  let editedEvent = $state<CalendarEvent | null>(null);

  let openEventFormButton = $state<HTMLButtonElement | null>(null);
  let navButtonRefs = $state<HTMLButtonElement[]>([]);

  onMount(() => {
    calendarDate.set(new Date());
    getCalendarEvents(yearMonthString);
  });

  onNavigate(() => {
    const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
    statusBar.textContent = null;
  });

  $effect(() => {
    if ($calendarDate !== null) {
      const statusBar = document.getElementById("status-bar")?.firstChild as HTMLParagraphElement;
      statusBar.textContent = `${$t["calendar.monthnames"][$calendarDate.getMonth()]}, ${$calendarDate.getFullYear()}`;
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const clearSearch = () => { searchable = null; searchRegex = null; };
  const toggleEventFormVisibility = () => {
    isEventFormVisible = !isEventFormVisible;
    editedEvent = null;
  };
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => {
    direction = delta;
    calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1));
    getCalendarEvents(yearMonthString);
  };

  const handleSearch = () => {
    if (!isSearchVisible) isSearchVisible = true;
    if (!searchable || searchable.trim() === '') return;

    searchRegex = new RegExp(searchable, 'gi');
  };

  const editEvent = (event: CalendarEvent) => {
    isEventFormVisible = true;
    editedEvent = event;
  };

  const stopEdit = () => {
    isEventFormVisible = false;
    editedEvent = null;
  };

</script>

<div id="calendar-main-container" class="vertical-flex-container">
  {#if isEventFormVisible}
    <div class="form-wrapper vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}
      use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => stopEdit(), additionalElements: [openEventFormButton].concat(navButtonRefs) }}
    >
      <EventForm closeForm={() => stopEdit()} {navButtonRefs} {editedEvent} />
    </div>
  {/if}

  <div id="calendar-toolbar" class="primary-toolbar horizontal-flex-container">
    <div id="calendar-nav-buttons" class="horizontal-flex-container">
      {#each [...Array(2)] as _, i (i)}
        <button bind:this={navButtonRefs[i]} class="transparent-button-highlight" onclick={() => goToMonth(i === 0 ? -1 : 1)}>
          <img src="arrow.svg" alt="{i === 0 ? 'Back' : '-Forward'} arrow" class="img-small" style="transform: rotate({i === 0 ? '90deg' : '-90deg'});" />
        </button>
      {/each}
    </div>
    <button class="primary-button horizontal-flex-container" bind:this={openEventFormButton} onclick={() => toggleEventFormVisibility()}>
      <img src="plus.svg" alt="Add event" class="img-small" style="transform: rotate({isEventFormVisible ? '45deg' : '0'}); transition: transform 0.1s;" />
      {$t[isEventFormVisible ? "cancel.button" : "add.button"]}
    </button>
  </div>

  <div id="calendar-content" class="horizontal-flex-container">
    <div id="calendar-event-container" class="vertical-flex-container" style="width: {isEventsListVisible ? '300px' : '48px'};">
      <div id="calendar-event-container-top-bar" class="horizontal-flex-container" style="border-bottom: {isEventsListVisible ? '1px solid #333' : ''};">
        <button class="transparent-button-highlight" onclick={() => isEventsListVisible = !isEventsListVisible}>
          <img src="/arrow.svg" alt="arrow" class="img-small" style="transform: rotate({isEventsListVisible ? '90deg' : '-90deg'});"/>
        </button>
        {#if isEventsListVisible}
          <div id="calendar-search-container" class="horizontal-flex-container" style="background-color: {isSearchVisible ? '#333' : 'transparent'};"
            use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isSearchVisible = false }}
          >
            {#if isSearchVisible}
              <input type="text" class="primary-input" placeholder={$t["search.placeholder"] as string} bind:value={searchable} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} 
                onkeydown={(e) => { switch (e.key) {
                  case 'Enter': handleSearch(); break;
                  case 'Escape': clearSearch(); break;
                }}}
              />
              <button id="clear-search-button" class="transparent-button-highlight" onclick={() => clearSearch()} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
                <img src="close-x.svg" alt="Close" />
              </button>
            {/if}
            <button class="transparent-button-highlight" style="border-radius: {isSearchVisible ? '0 4px 4px 0' : '50%'};" onclick={() => handleSearch()}>
              <img src="search.svg" alt="Search" class="img-small" />
            </button>
          </div>
        {/if}
      </div>
      {#if isEventsListVisible}
        <div id="calendar-event-wrapper" class="vertical-flex-container">
          {#each displayEvents as event (event.id)}
            <div role="button" tabindex="0" class="calendar-event vertical-flex-container" in:fly={{ x: -300, duration: 400, easing: cubicInOut }}
              onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); editEvent(event) }}}
              onclick={() => editEvent(event)}
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
                    onConfirm: () => deleteCalendarEvent(event)
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

    <div id="calendar-days-container" class="vertical-flex-container">
      <div id="calendar-weekdays">
        {#each $t["calendar.weekdays"] as weekDay (weekDay)}
          <p>{weekDay}</p>
        {/each}
      </div>
      <div id="calendar-grid-wrapper">
        {#key `${$calendarDate.getFullYear()}-${$calendarDate.getMonth()}`}
          <div id="calendar-grid" in:fly={{ x: direction * monthTransitionWidth, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * -monthTransitionWidth, duration: 300, easing: cubicInOut }}>
            {#each $calendarDays as day (day.date)}
              <div class="horizontal-flex-container" class:disabled-day={!day.enabled}>
                <p class:today={day.isodate === todayIsodate}>
                  {day.number}
                </p>
                {#if displayEvents.some(e => e.isodate === day.isodate)}
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
    background-color: rgb(255, 70, 70);
    border-radius: 50%;
    font-weight: bold;
  }

  .form-wrapper {
    position: absolute;
    z-index: 500;
    top: 60px;
    left: 8px;
    max-height: calc(100% - 64px);
    border-radius: 8px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  #calendar-main-container,
  #calendar-content,
  #calendar-content > div {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #calendar-content {
    height: calc(100% - 56px);
  }

  #calendar-toolbar {
    #calendar-nav-buttons {
      gap: 6px;

      button {
        height: 28px;
        width: 28px;
      }
    }

    button {
      gap: 8px;
    }
  }

  #calendar-content #calendar-event-container {
    flex-shrink: 0;
    justify-content: flex-start;
    align-items: flex-start;
    border-right: 1px solid #333;
    transition: width 0.2s;

    #calendar-event-container-top-bar {
      justify-content: flex-start;
      width: 100%;
      gap: 12px;
      padding: 8px;

      button:not(#clear-search-button) {
        flex-shrink: 0;
        height: 32px;
        width: 32px;
      }

      #clear-search-button {
        flex-shrink: 0;
        width: 20px;
        height: 20px;
        margin-right: 6px;

        img {
          width: 10px;
          height: 10px;
        }
      }
    }

    #calendar-event-wrapper {
      justify-content: flex-start;
      width: 100%;
      height: 100%;
      overflow-y: auto;
     
      div.calendar-event:not(:last-child) {
        border-bottom: 1px solid #333;
      }
      div.calendar-event {
        width: 100%;
        background-color: #222;
      }
      div.calendar-event:hover {
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.8);
        z-index: 1;
        cursor: pointer;
      }
    }

    #calendar-search-container {
      justify-content: flex-end;
      border-radius: 4px;
      width: 100%;

      input {
        outline: none;
        color: #f6f6f6;
      }
    }
  }

  #calendar-days-container {
    > div:not(#calendar-weekdays) {
      height: 100%;
    }

    #calendar-weekdays {
      text-align: center;
      border-bottom: 1px solid #333;
      
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
        height: 30px;
        width: 30px;
        line-height: normal;
        text-align: center;
      }

      span.event-indicator {
        position: relative;
        align-self: flex-start;
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background-color: rgb(255, 70, 70);
      }
      span.event-indicator::after {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 2px solid rgb(255, 70, 70);
        animation: pulse 1.5s ease-out infinite;
      }
    }

    > div:hover {
      background-color: #333;
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