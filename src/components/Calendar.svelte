<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { calendarDays, calendarDate } from "$lib/calendar";

  let {
    options,
  }: {
    options: {
      setCalendarIsoDate: (day: string) => void;
      setCalendarVisibility: (state: boolean) => void;
      calendarToggle: HTMLButtonElement | null;
      calendarStartDate?: Date;
      ignorableEls?: (HTMLElement | null)[];
      isMonthChangeEnabled?: boolean;
    },
  } = $props();

  // svelte-ignore state_referenced_locally
  calendarDate.set(options.calendarStartDate ? options.calendarStartDate : new Date());

  const today = new Date(new Date().getFullYear(), new Date().getMonth(), new Date().getDate());
  const isoDateToday = `${String(today.getFullYear())}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
  let direction = $state(1);
  let isMonthChangeEnabled = $derived(options.isMonthChangeEnabled ?? true);

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleOutsideClick = () => { options.setCalendarVisibility(false) };
  
  /***********************************************************************************************************************************/

  const goToMonth = (delta: number) => { direction = delta; calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1)); };

</script>

<div id="calendar-modal" class="vertical-flex-container"
  use:handleClickOutside={{ onOutsideClick: handleOutsideClick, additionalElements: options.ignorableEls ? options.ignorableEls.concat(options.calendarToggle) : [options.calendarToggle] }}
>
  <div id="calendar-topbar" class="horizontal-flex-container">
    <button id="close-button" class="transparent-button-highlight" style="margin-right: 6px;" onclick={() => options.setCalendarVisibility(false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
    <div class="vertical-flex-container">
      <p>{`${$t["calendar.current-day.name"][today.getDay()]}, ${today.getDate()}. ${$t["calendar.monthnames"][today.getMonth()]}${$lang === 'fi' ? "ta" : ""}`}</p>
      <p style="font-weight: bold;">{`${$t["calendar.monthnames"][$calendarDate.getMonth()]}, ${$calendarDate.getFullYear()}`}</p>
    </div>
    {#if isMonthChangeEnabled}
      <div class="horizontal-flex-container" style="justify-content: flex-end; gap: 6px;">
        <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(-1)}><img src="/arrow.svg" alt="Next" class="img-small" style="transform: rotate(90deg);" /></button>
        <button class="transparent-button vertical-flex-container" onclick={() => goToMonth(1)}><img src="/arrow.svg" alt="Back" class="img-small" style="transform: rotate(-90deg);" /></button>
      </div>
    {/if}
  </div>
  <div id="calendar-weekdays">
    {#each $t["calendar.weekdays"] as day}
      <p>{day}</p>
    {/each}
  </div>
  <div id="calendar-grid-wrapper">
    {#key `${$calendarDate.getFullYear()}-${$calendarDate.getMonth()}`}
      <div id="calendar-days-grid" in:fly={{ x: direction * 316, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * -316, duration: 300, easing: cubicInOut }}>
        {#each $calendarDays as day (day.isodate)}
          <button
            class="transparent-button calendar-day vertical-flex-container"
            class:disabled-day={day.enabled === false}
            class:currentDay={day.isodate === isoDateToday}
            onclick={() => { options.setCalendarIsoDate(day.isodate); options.setCalendarVisibility(false); }}
          >
            {day.number}
          </button>
        {/each}
      </div>
    {/key}
  </div>
</div>

<style>
  #calendar-modal {
    height: 390px;
    width: 348px;
    flex-shrink: 0;
    border-radius: 8px;
    gap: 8px;
    background-color: rgb(200, 200, 200);
    color: black;
    user-select: none;
  }

  #calendar-modal p {
    margin: 0;
    text-align: center;
    font-size: clamp(0.8rem, 0.9cqw, 0.9rem);
  }

  #calendar-topbar {
    position: relative;
    width: 100%;
    height: 64px;
    justify-content: space-between;
    padding: 8px 16px;
    background-color: rgb(180, 180, 180, 0.8);
    border-radius: 8px 8px 0 0;

    button {
      height: 32px;
      width: 32px;
    }

    button:not(#close-button) {
      border-radius: 4px;
    }

    div:first-of-type {
      position: absolute;
      left: 50%;
      transform: translateX(-50%);
    }
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
    background-color: rgba(155, 155, 155, 0.9);
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