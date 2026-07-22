<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { fly, slide } from "svelte/transition";
  import { onNavigate } from "$app/navigation";

  import { calendarDays, calendarDate } from "$lib/calendar";
  import { t } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { viewport } from "$lib/viewport";

  import EventForm from "../../components/calendar/EventForm.svelte";

  let isEventsListVisible = $state<boolean>(true);
  let isEventFormVisible = $state<boolean>(false);
  const todayIsodate = ((d: Date) => `${String(d.getFullYear())}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`)(new Date());
  let openEventFormButton = $state<HTMLButtonElement | null>(null);
  const monthTransitionWidth = $derived($viewport.width / 2);
  let direction = $state(1);

  onMount(() => {
    calendarDate.set(new Date());
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
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => { direction = delta; calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1)); };

</script>

<div id="calendar-main-container" class="vertical-flex-container">
  {#if isEventFormVisible}
    <div class="form-wrapper vertical-flex-container" transition:slide={{ axis: "y", duration: 300, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isEventFormVisible = false, additionalElements: [openEventFormButton] }}>
      <EventForm closeForm={() => isEventFormVisible = false} />
    </div>
  {/if}

  <div id="calendar-toolbar" class="primary-toolbar horizontal-flex-container">
    <div id="calendar-nav-buttons" class="horizontal-flex-container">
      {#each [...Array(2)] as _, i (i)}
        <button class="transparent-button-highlight" onclick={() => goToMonth(i === 0 ? -1 : 1)}>
          <img src="arrow.svg" alt="{i === 0 ? 'Back' : '-Forward'} arrow" class="img-small" style="transform: rotate({i === 0 ? '90deg' : '-90deg'});" />
        </button>
      {/each}
    </div>
    <button class="primary-button horizontal-flex-container" bind:this={openEventFormButton} onclick={() => isEventFormVisible = !isEventFormVisible}>
      <img src="plus.svg" alt="Add event" class="img-small" style="transform: rotate({isEventFormVisible ? '45deg' : '0'}); transition: transform 0.1s;" />
      {$t[isEventFormVisible ? "cancel.button" : "add.button"]}
    </button>
  </div>

  <div id="calendar-content" class="horizontal-flex-container">
    <div id="calendar-event-container" class="vertical-flex-container" style="width: {isEventsListVisible ? '300px' : '48px'}">
      <div class="horizontal-flex-container">
        <button class="transparent-button-highlight" onclick={() => isEventsListVisible = !isEventsListVisible}>
          <img src="/arrow.svg" alt="arrow" class="img-small" style="transform: rotate({isEventsListVisible ? '90deg' : '-90deg'});"/>
        </button>
      </div>
      {#if isEventsListVisible}
        <p in:slide={{ axis: "x", duration: 200, delay: 100, easing: cubicInOut }}>placeholder</p>
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
              <div class="vertical-flex-container" class:disabled-day={!day.enabled}>
                <p class:today={day.isodate === todayIsodate}>
                  {day.number}
                </p>
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
    opacity: 0.5;
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
    width: 100%;
    height: 100%;
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
    padding: 8px;
    border-right: 1px solid #333;
    transition: width 0.2s;

    > div:first-of-type {
      justify-content: flex-end;
      width: 100%;

      > button {
        height: 32px;
        width: 32px;
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
      justify-content: flex-start;
      padding: 6px;

      p {
        align-self: flex-end;
        margin: 0;
        padding: 6px;
        height: 30px;
        width: 30px;
        line-height: normal;
        text-align: center;
      }
    }

    > div:hover {
      background-color: #222;
      cursor: pointer;
    }

    > div:not(:nth-child(7n)) {
      border-right: 1px solid #222;
    }

    > div:not(:nth-last-child(-n+7)) {
      border-bottom: 1px solid #222;
    }
  }
</style>