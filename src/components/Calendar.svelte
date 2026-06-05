<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { getContext } from "svelte";

  import type { CalendarDay } from "$lib/types";
  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/functions";

  let {
    setCalendarIsoDate,
    setCalendarVisibility,
    calendarToggle,
  }: {
    setCalendarIsoDate: (day: string) => void;
    setCalendarVisibility: (state: boolean) => void;
    calendarToggle: HTMLButtonElement | null;
  } = $props();

  let current = $state(new Date());
  const today = (() => { return new Date(current.getFullYear(), current.getMonth(), current.getDate()); })();
  const isoDateToday = `${String(today.getFullYear())}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;

  let direction = $state(1);

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
      let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

      daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
    }

    for (let i = 0; i < currentMonthDays; i++) {
      let day = new Date(year, month, i + 1);
      let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

      daysArray.push({ enabled: true, number: '' + (i + 1), date: day, isodate: isodate });
    }

    let i = 0;
    while (daysArray.length < 42) {
      let day = new Date(month === 11 ? year + 1 : year, (month + 1)%12, i + 1);
      let isodate = `${String(day.getFullYear())}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

      daysArray.push({ enabled: false, number: '' + (i + 1), date: day, isodate: isodate });
      i++;
    }

    return daysArray;
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { setCalendarVisibility(false) };
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => { direction = delta; current = new Date(current.getFullYear(), current.getMonth() + delta, 1); };

</script>

<div id="calendar-modal" class="vertical-flex-container" transition:fly={{ x: 30, duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{getIgnoredElements, onOutsideClick: handleOutsideClick, additionalElements: [calendarToggle]}}
>
  <div id="calendar-topbar" class="horizontal-flex-container">
    <button id="close-button" class="transparent-button-highlight" style="margin-right: 6px;" onclick={() => setCalendarVisibility(false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
    <div class="vertical-flex-container">
      <p>{`${$t["calendar.current-day.name"][today.getDay()]}, ${today.getDate()}. ${$t["calendar.monthnames"][today.getMonth()]}${$lang === 'fi' ? "ta" : ""}`}</p>
      <p style="font-weight: bold;">{`${$t["calendar.monthnames"][current.getMonth()]}, ${current.getFullYear()}`}</p>
    </div>
    <div class="horizontal-flex-container" style="justify-content: flex-end; gap: 6px;">
      <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(-1)}><img src="/arrow.svg" alt="Next" class="img-small" style="transform: rotate(90deg);" /></button>
      <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(1)}><img src="/arrow.svg" alt="Back" class="img-small" style="transform: rotate(-90deg);" /></button>
    </div>
  </div>
  <div id="calendar-weekdays">
    {#each $t["calendar.weekdays"] as day}
      <p>{day}</p>
    {/each}
  </div>
  <div id="calendar-grid-wrapper">
    {#key `${current.getFullYear()}-${current.getMonth()}`}
      <div id="calendar-days-grid" in:fly={{ x: direction * 316, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * -316, duration: 300, easing: cubicInOut }}>
        {#each days as day (day.isodate)}
          <button class="transparent-button calendar-day vertical-flex-container" class:disabled-day={day.enabled === false} class:currentDay={day.isodate === isoDateToday} onclick={() => { setCalendarIsoDate(day.isodate); setCalendarVisibility(false); }}>
            {day.number}
          </button>
        {/each}
      </div>
    {/key}
  </div>
</div>

<style>
  #calendar-modal {
    position: absolute;
    z-index: 1000;
    align-self: flex-end;
    margin: 60px 8px 0 0;
    border-radius: 8px;
    gap: 8px;
    background-color: rgb(200, 200, 200);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
    color: black;
    user-select: none;
  }

  #calendar-modal p {
    margin: 0;
    text-align: center;
    font-size: clamp(0.8rem, 0.9cqw, 0.9rem);
  }

  #calendar-topbar {
    width: 100%;
    justify-content: space-between;
    padding: 8px 16px;
    background-color: rgb(180, 180, 180, 0.8);
    border-radius: 8px 8px 0 0;
  }

  #calendar-topbar button {
    height: 32px;
    width: 32px;
  }
  #calendar-topbar button:not(#close-button) {
    border-radius: 6px;
  }

  img {
    filter: brightness(0);
  }

  #calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 40px);
    gap: 6px;
    padding: 0 16px;
  }

  #calendar-grid-wrapper {
    overflow: hidden;
    position: relative;
    height: 286px;
    width: 100%;
    padding: 0 16px 16px;
  }

  #calendar-days-grid {
    position: absolute;
    top: 0;
    left: 0;
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
    border: 1px solid transparent;
  }
  .calendar-day:focus-visible {
    outline: none;
  }

  .calendar-day:hover, #calendar-topbar button:hover {
    cursor: pointer;
    background-color: rgba(165, 165, 165, 0.9);
  }

  .calendar-day.disabled-day {
    color: rgba(0, 0, 0, 0.3);
  }

  .calendar-day.currentDay {
    outline: none;
    background-color: rgba(255, 70, 70, 1);
    color: #f6f6f6;
    font-weight: bold;
  }

  .calendar-day.currentDay.disabled-day {
    background-color: rgba(255, 70, 70, 0.5);
  }
</style>