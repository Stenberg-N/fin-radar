<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { untrack } from "svelte";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import { handleClickOutside } from "$lib/utils/actions.svelte";
  import { calendarDays, calendarDate } from "$lib/calendar";
  import { formatMonthYear, toIsoDate, toYearMonth } from "$lib/utils/helpers.svelte";

  interface Props {
    /** Returns the isodate of the clicked calendar day. */
      setCalendarIsoDate: (day: string) => void;
      /**
       * Returns a boolean false when clicked outside of the component or the close button.
       * 
       * Assign this value to a $state to conditionally render the component.
       */
      setCalendarVisibility: (state: boolean) => void;
      /**
       * Used to pass the button reference that you use to toggle the component's visibility.
       * 
       * Gets ignored by the outside click handler.
       */
      calendarToggle: HTMLButtonElement | null;
      /** Used to pass a date from which the calendar should start. Defaults to now. */
      calendarStartDate?: Date;
      /** An array of extra elements the outside click handler should ignore. */
      ignorableEls?: (HTMLElement | null)[];
      /** Used to hide the buttons to switch months in the component. */
      isMonthChangeEnabled?: boolean;
      theme?: "light" | "dark";
  }

  let {
    setCalendarIsoDate,
    setCalendarVisibility,
    calendarToggle,
    calendarStartDate = new Date(),
    ignorableEls,
    isMonthChangeEnabled = true,
    theme = "light",
  }: Props = $props();

  const themes = {
    dark: {
      bg: "var(--color-secondary2)",
      text: "var(--color-white-primary1)",
      "topbar-bg": "var(--color-secondary2-a)",
      hover: "var(--hover-color-transparent)",
      img: "var(--color-white-primary1)",
    },
    light: {
      bg: "var(--color-primary3)",
      text: "black",
      "topbar-bg": "rgb(180, 180, 180, 0.8)",
      hover: "rgba(155, 155, 155, 0.9)",
      img: "black",
    }
  } as const;
  const t = $derived(themes[theme] ?? themes.light);

  untrack(() => calendarDate.set(calendarStartDate));

  const today = new Date();
  const isoDateToday = toIsoDate(today);
  let direction = $state(1);

  const goToMonth = (delta: number) => { direction = delta; calendarDate.set(new Date($calendarDate.getFullYear(), $calendarDate.getMonth() + delta, 1)); };

</script>

<div id="calendar-modal" class="flex column"
  style:--bg={t.bg}
  style:--text={t.text}
  style:--hover={t.hover}
  style:--topbar-bg={t["topbar-bg"]}
  style:--icon-color={t.img}
  use:handleClickOutside={{ onOutsideClick: () => { setCalendarVisibility(false); }, getAdditionalElements: () => ignorableEls ? ignorableEls.concat(calendarToggle) : [calendarToggle] }}
>
  <div id="calendar-topbar" class="flex row">
    <button aria-label="Close calendar" id="close-button" class="button-primary transparent highlight static" style="margin-right: 6px;" onclick={() => setCalendarVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
    </button>
    <div class="flex column">
      <p>{`${(i18n.t["calendar.current-day.name"] as string[])[today.getDay()]}, ${today.getDate()}. ${(i18n.t["calendar.monthnames"] as string[])[today.getMonth()]}${i18n.lang === 'fi' ? "ta" : ""}`}</p>
      <p style="font-weight: bold;">{formatMonthYear($calendarDate)}</p>
    </div>
    {#if isMonthChangeEnabled}
      <div class="flex row" style="justify-content: flex-end;">
        <button aria-label="Previous month" class="button-primary transparent static" onclick={() => goToMonth(-1)}>
          <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transform: rotate(90deg);"></span>
        </button>
        <button aria-label="Next month" class="button-primary transparent static" onclick={() => goToMonth(1)}>
          <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transform: rotate(-90deg);"></span>
        </button>
      </div>
    {/if}
  </div>
  <div id="calendar-weekdays">
    {#each (i18n.t["calendar.weekdays"] as string[]) as day}
      <p>{day}</p>
    {/each}
  </div>
  <div id="calendar-grid-wrapper">
    {#key toYearMonth($calendarDate)}
      <div id="calendar-days-grid" in:fly={{ x: direction * 316, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * -316, duration: 300, easing: cubicInOut }}>
        {#each $calendarDays as day (day.isodate)}
          <button
            class="button-primary transparent calendar-day"
            class:disabled-day={day.enabled === false}
            class:currentDay={day.isodate === isoDateToday}
            onclick={() => { setCalendarIsoDate(day.isodate); setCalendarVisibility(false); }}
          >
            <span>{day.number}</span>
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
    border-radius: 0.5rem;
    gap: 0.5rem;
    background-color: var(--bg);
    color: var(--text);
    user-select: none;

    .span-icon {
      background-color: var(--icon-color);
    }
  }

  #calendar-modal p {
    margin: 0;
    text-align: center;
    font-size: 0.75rem;
  }

  #calendar-topbar {
    position: relative;
    width: 100%;
    height: 4rem;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    background-color: var(--topbar-bg);
    border-radius: 0.5rem 0.5rem 0 0;

    button:not(#close-button) {
      border-radius: 0.25rem;
    }

    div:first-of-type {
      position: absolute;
      top: 0.5rem;
      bottom: 3.5rem;
      left: 50%;
      justify-content: flex-start;
      height: 100%;
      transform: translateX(-50%);
    }
  }

  #calendar-weekdays {
    display: grid;
    grid-template-columns: repeat(7, 40px);
    gap: 6px;
    padding: 0 1rem;
  }

  #calendar-grid-wrapper {
    overflow: hidden;
    position: relative;
    height: 286px;
    width: 100%;
    padding: 0 1rem 1rem;
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

  .calendar-day:hover, #calendar-topbar button:hover {
    background-color: var(--hover);
  }

  .calendar-day {
    height: unset;
    border-radius: 50%;
    color: inherit;
    border: 1px solid transparent;

    &:focus-visible {
      outline: none;
    }

    &.disabled-day span {
      opacity: 0.3;
    }

    &.currentDay {
      outline: 1px solid var(--color-highlight1);

      span {
        font-weight: bold;
      }

      &.disabled-day {
        outline-color: rgba(255, 70, 70, 0.5);
      }
    }
  }
</style>