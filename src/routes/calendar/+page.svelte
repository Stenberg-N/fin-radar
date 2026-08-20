<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fly, slide } from "svelte/transition";
  import { onNavigate } from "$app/navigation";

  import { calendarDays, calendarDate, getCalendarEvents, calendarEvents, deleteCalendarEvent, getCalendarTags } from "$lib/calendar";
  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { viewport } from "$lib/viewport";
  import type { CalendarEventWithTag } from "$lib/types";

  import EventForm from "../../components/calendar/EventForm.svelte";
  import TagsList from "../../components/calendar/TagsList.svelte";
  import SearchBar from "../../components/SearchBar.svelte";

  let isEventsListVisible = $state<boolean>(true);
  let isEventFormVisible = $state<boolean>(false);
  let isTagsListVisible = $state<boolean>(false);
  const monthTransitionWidth = $derived($viewport.width / 2);
  let direction = $state(1);
  const todayIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`)(new Date());
  const yearMonthString = $derived(((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}`)($calendarDate));
  let searchRegex = $state<RegExp | null>(null);

  const displayEvents = $derived(searchRegex !== null ? $calendarEvents.filter(obj => [obj.event.title, obj.event.description, obj.event.isodate].some((val) => searchRegex?.test(val as string))) : $calendarEvents);
  let editedEvent = $state<CalendarEventWithTag | null>(null);

  let openEventFormButton = $state<HTMLButtonElement | null>(null);
  let navButtonRefs = $state<HTMLButtonElement[]>([]);
  let tagsListToggleButton = $state<HTMLButtonElement | null>(null);

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
      statusBar.textContent = `${$t["calendar.monthnames"][$calendarDate.getMonth()]}, ${$calendarDate.getFullYear()}`;
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const toggleEventFormVisibility = () => {
    isEventFormVisible = !isEventFormVisible;
    editedEvent = null;
  };
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => {
    direction = delta;
    calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1));
    getCalendarEvents(yearMonthString);
    stopEdit();
  };

  const editEvent = (event: CalendarEventWithTag) => {
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
    <div class="form-wrapper vertical-flex-container" style="top: 60px; left: 304px; max-width: 650px; width: 100%;" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }}
      use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => stopEdit(), additionalElements: [openEventFormButton].concat(navButtonRefs) }}
    >
      <EventForm closeForm={() => stopEdit()} {navButtonRefs} {editedEvent} />
    </div>
  {/if}

  {#if isTagsListVisible}
    <TagsList options={{
      setListVisibility: (state) => isTagsListVisible = state,
      tagsListToggleButton,
      isTagsListVisible,
    }}
    />
  {/if}

  <div id="calendar-toolbar" class="primary-toolbar horizontal-flex-container">
    <div id="calendar-nav-buttons" class="horizontal-flex-container">
      {#each [...Array(2)] as _, i (i)}
        <button bind:this={navButtonRefs[i]} title={$t["month-transition-buttons"][i] as string} class="transparent-button-highlight" onclick={() => goToMonth(i === 0 ? -1 : 1)}>
          <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({i === 0 ? '90deg' : '-90deg'});"></span>
        </button>
      {/each}
    </div>
    <div class="horizontal-flex-container">
      <button aria-label="Open form" class="primary-button" bind:this={openEventFormButton} onclick={() => toggleEventFormVisibility()}>
        <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isEventFormVisible ? '45deg' : '0'}); transition: transform 0.1s;"></span>
      </button>
      <button aria-label="Open tags" bind:this={tagsListToggleButton} class="primary-button" onclick={() => isTagsListVisible = !isTagsListVisible} disabled={isEventFormVisible}>
        <span class="span-icon img-small" style="mask-image: url('tags.svg');"></span>
      </button>
    </div> 
  </div>

  <div id="calendar-content" class="horizontal-flex-container">
    <div id="calendar-event-container" class="vertical-flex-container" style="width: {isEventsListVisible ? '300px' : '48px'};">
      <div id="calendar-event-container-top-bar" class="horizontal-flex-container" style="border-bottom: {isEventsListVisible ? '1px solid #333' : ''};">
        <button aria-label="Toggle event list" class="transparent-button-highlight" onclick={() => isEventsListVisible = !isEventsListVisible}>
          <span class="span-icon img-small" style="mask-image: url('arrow.svg'); transform: rotate({isEventsListVisible ? '90deg' : '-90deg'});"></span>
        </button>
        {#if isEventsListVisible}
          <SearchBar options={{ sendRegexToParent: (regex) => searchRegex = regex }} />
        {/if}
      </div>
      {#if isEventsListVisible}
        <div id="calendar-event-wrapper" class="vertical-flex-container">
          {#each displayEvents as { event, tags } (event.id)}
            <div role="button" tabindex="0" class="calendar-event vertical-flex-container" in:fly={{ x: -300, duration: 400, easing: cubicInOut }}
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
    background-color: rgb(255, 70, 70);
    border-radius: 50%;
    font-weight: bold;
  }

  .form-wrapper {
    max-height: calc(100% - 64px);
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

    > div:not(:first-of-type) {
      gap: 12px;

      button {
        width: 32px;
        height: 32px;
      }
    }
  }

  #calendar-content #calendar-event-container {
    flex-shrink: 0;
    justify-content: flex-start;
    align-items: flex-start;
    border-right: 1px solid #333;
    transition: width 0.2s;

    #calendar-event-container-top-bar {
      justify-content: space-between;
      width: 100%;
      gap: 12px;
      padding: 8px;

      button {
        flex-shrink: 0;
        height: 32px;
        width: 32px;
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