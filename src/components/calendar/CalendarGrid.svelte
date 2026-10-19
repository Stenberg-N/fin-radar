<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import { viewport } from "$lib/utils/viewport";
  import type { CalendarDay, CalendarEventWithTag } from "$lib/types";
  import { toYearMonth } from "$lib/utils/helpers.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";

  interface Props {
    calendarDate: Date;
    calendarDays: CalendarDay[];
    calendarEvents: CalendarEventWithTag[];
    /** The value 1 or -1 for determining the transition animation direction. */
    direction: number;
    /** The parent container's client width. */
    parentWidth: number;
    /** Today's isodate passed from the parent. */
    todayIsodate: string;
  }

  let {
    calendarDate,
    calendarDays,
    calendarEvents,
    direction,
    parentWidth,
    todayIsodate,
  }: Props = $props();

  const MONTH_TRANSITION_WIDTH = $derived($viewport.width / 2);
  const hover = new HoverTitle();
</script>

<div id="calendar-days-container" class="flex column" in:fly={{ x: -parentWidth / 2, duration: 400, easing: cubicInOut }}>
  {#if hover.isHovering}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ moveTop: -32 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      outline={{ width: 1, color: 'var(--outline-color1)'}}
      borderRadius={8}
    >
      <p id="calendar-grid-hover-title-content">
        {i18n.t["calendar.day.has-events"] as string}
      </p>
    </ModalWrapper>
  {/if}

  <div id="calendar-weekdays">
    {#each (i18n.t["calendar.weekdays"] as string[]) as weekDay (weekDay)}
      <p>{weekDay}</p>
    {/each}
  </div>
  <div id="calendar-grid-wrapper">
    {#key toYearMonth(calendarDate)}
      <div id="calendar-grid" in:fly={{ x: direction * -MONTH_TRANSITION_WIDTH, duration: 300, easing: cubicInOut }} out:fly={{ x: direction * MONTH_TRANSITION_WIDTH, duration: 300, easing: cubicInOut }}>
        {#each calendarDays as day (day.date)}
          <div class="flex row" class:disabled-day={!day.enabled}>
            <p class:today={day.isodate === todayIsodate}>
              {day.number}
            </p>
            {#if calendarEvents.some(obj => obj.event.isodate === day.isodate)}
              <span role="tooltip" class="event-indicator"
                onmouseenter={() => hover.enter()}
                onmouseleave={(e) => hover.leave(e)}
              ></span>
            {/if}
          </div>
        {/each}
      </div>
    {/key}
  </div>
</div>

<style>
  #calendar-days-container {
    flex: 1 1 auto;
    align-self: stretch;

    .disabled-day > * {
      opacity: 0.4;
    }

    .today {
      border-radius: 50%;
      background-color: var(--color-white-primary1);
      font-weight: bold;
      color: var(--color-primary2);
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    }

    #calendar-grid-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary1);
    }

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
          align-content: center;
          margin: 0;
          padding: 6px;
          height: 32px;
          width: 32px;
          font-size: 14px;
          line-height: 14px;
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
  }
</style>