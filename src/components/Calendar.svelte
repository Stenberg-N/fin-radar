<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { getContext } from "svelte";

  import type { CalendarDay } from "$lib/types";
  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/functions";
  import { setViewState } from "$lib/viewStore";

  let {
    setCalendarIsoDate,
    calendarToggle,
  }: {
    setCalendarIsoDate: (day: string) => void;
    calendarToggle: HTMLButtonElement | null;
  } = $props();

  // Context, Helper & Wrapper functions
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { setViewState("isCalendar", false) };

  let current = $state(new Date());
  const today = (() => { return new Date(current.getFullYear(), current.getMonth(), current.getDate()); })();
  const isoDateToday = `${String(today.getDate()).padStart(2, '0')}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getFullYear())}`;

  let days = $derived.by(() => {
    const year = current.getFullYear();
    const month = current.getMonth();
    const daysArray: CalendarDay[] = [];

    let firstDayLastMonth = new Date(year, month, 1).getDay();
    let offset = firstDayLastMonth === 0 ? 6 : firstDayLastMonth - 1;
    let currentMonthDays = new Date(year, month + 1, 0).getDate();
    let lastMonthDays = new Date(year, month, 0).getDate();
    let previousMonth = month === 0 ? 11 : month - 1;

    for (let i = lastMonthDays - offset; i < lastMonthDays; i++) {
      let day = new Date(previousMonth === 11 ? year - 1 : year, previousMonth, i + 1);
      let isodate = `${String(day.getDate()).padStart(2, '0')}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getFullYear())}`;

      daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
    }

    for (let i = 0; i < currentMonthDays; i++) {
      let day = new Date(year, month, i + 1);
      let isodate = `${String(day.getDate()).padStart(2, '0')}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getFullYear())}`;

      daysArray.push({ enabled: true, number: '' + (i + 1), date: day, isodate: isodate });
    }

    let i = 0;
    while (daysArray.length < 42) {
      let day = new Date(month === 11 ? year + 1 : year, (month + 1)%12, i + 1);
      let isodate = `${String(day.getDate()).padStart(2, '0')}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getFullYear())}`;

      daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
      i++;
    }

    return daysArray;
  });

  const goToMonth = (delta: number) => { current = new Date(current.getFullYear(), current.getMonth() + delta, 1); };

</script>

<div id="calendar-modal" class="vertical-flex-container" transition:fly={{ x: 20, duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{getIgnoredElements, onOutsideClick: handleOutsideClick, additionalElements: [calendarToggle]}}
>
  <div id="calendar-topbar" class="horizontal-flex-container">
    <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(-1)}><img src="/arrow.svg" alt="Next" class="img-small" style="transform: rotate(90deg);" /></button>
    <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(1)}><img src="/arrow.svg" alt="Back" class="img-small" style="transform: rotate(-90deg);" /></button>
    <p style="margin-left: auto;">{`${$t["calendar.current-day.name"][today.getDay()]}, ${today.getDate()}. ${$t["calendar.monthnames"][today.getMonth()]}${$lang === 'fi' ? "ta" : ""}`}</p>
  </div>
  <div id="calendar-weekdays">
    {#each $t["calendar.weekdays"] as day}
      <p style="font-size: 14px;">{day}</p>
    {/each}
  </div>
  <div id="calendar-days-grid">
    {#each days as day}
      <button class="transparent-button calendar-day vertical-flex-container" class:disabled={day.enabled === false} class:currentDay={day.isodate === isoDateToday} onclick={() => setCalendarIsoDate(day.isodate)}>
        {day.number}
      </button>
    {/each}
  </div>
  <div class="horizontal-flex-container" style="width: 100%; justify-content: space-between;">
    <p style="font-weight: bold;">{`${$t["calendar.monthnames"][current.getMonth()]}, ${current.getFullYear()}`}</p>
    <button id="close-button" class="transparent-button-highlight" onclick={() => setViewState("isCalendar", false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
  </div>
</div>

<style>
  #calendar-modal {
    position: absolute;
    z-index: 1000;
    align-self: flex-end;
    margin: 65px 8px 0 0 ;
    border-radius: 8px;
    gap: 8px;
    padding: 24px;
    background-color: rgba(200, 200, 200, 1);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    color: black;
    user-select: none;
  }

  #calendar-modal p {
    margin: 0;
    text-align: center;
  }

  #calendar-topbar {
    width: 100%;
    justify-content: flex-start;
    gap: 6px;
  }

  #calendar-topbar button {
    height: 32px;
    width: 32px;
    outline: 1px solid transparent;
    border-radius: 6px;
  }

  #calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 40px);
    gap: 6px;
  }

  #calendar-days-grid {
    display: grid;
    grid-template-columns: repeat(7, 40px);
    grid-auto-rows: 40px;
    justify-content: center;
    width: 100%;
    height: 100%;
    gap: 6px;
  }

  .calendar-day {
    border-radius: 50%;
    outline: 1px solid transparent;
  }

  .calendar-day:hover, #calendar-topbar button:hover {
    cursor: pointer;
    outline-color: rgba(255, 70, 70, 1);
    background-color: rgba(165, 165, 165, 0.9);
  }

  .calendar-day.disabled {
    color: rgba(0, 0, 0, 0.3);
  }

  .calendar-day.currentDay {
    outline: none;
    background-color: rgba(255, 70, 70, 1);
    color: #f6f6f6;
    font-weight: bold;
  }

  .calendar-day.currentDay.disabled {
    background-color: rgba(255, 70, 70, 0.5);
  }

  #close-button {
    width: 32px;
    height: 32px;
  }

  #close-button:hover {
    background-color: rgba(165, 165, 165, 0.9);
  }
</style>