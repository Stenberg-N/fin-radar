<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import type { CalendarEventWithTag } from "#lib/types.ts";
  import { userPrefs } from "#lib/utils/prefsStore.ts";

  interface Props {
    /** The parent container's client width. */
    parentWidth: number;
    calendarEvents: CalendarEventWithTag[];
  }

  let {
    parentWidth,
    calendarEvents: rawEvents,
  }: Props = $props();

  const calendarEvents = $derived.by(() => {
    const rows: CalendarEventWithTag[][] = [];

    for (const event of rawEvents) {
      let isPushed = false;
      const start = event.event.start_time ?? 0;
      const end = event.event.end_time ?? 0

      for (const row of rows) {
        const isOverlapping = row.some((e) => !(end <= (e.event.start_time ?? 0) || start >= (e.event.end_time ?? 0)));

        if (!isOverlapping) {
          row.push(event);
          isPushed = true;
          break;
        }
      }

      if (!isPushed) {
        rows.push([event]);
      }
    }

    return rows.flatMap((row, index) => row.map((event) => ({ ...event, row: index })));
  });
  const HOURS = $derived($userPrefs.mainPrefs.hourFormat === "12"
    ? Array.from({ length: 2 }, (_, i) => [...Array(12)].map((_, idx) => String(idx === 0 ? 12 : idx) + (i === 0 ? " AM" : " PM"))).flatMap((num) => num)
    : Array.from({ length: 24 }, (_, i) => String(i).padStart(2, '0')));
  const SECONDS_IN_HOUR = 3600;
  const EVENT_HEIGHT = 64;
</script>

<div
  id="calendar-horizontal-view-main"
  class="flex column"
  style:--ch={$userPrefs.mainPrefs.hourFormat === "12" ? "1.5ch" : "0.75ch"}
  in:fly={{ x: -parentWidth / 2, duration: 400, easing: cubicInOut }}
>
  <div id="hours-container" class="flex">
    {#each HOURS as hour, i (i)}
      <p>{hour}</p>
    {/each}
  </div>
  <div id="timeline">
    {#each calendarEvents as ce (ce.event.id)}
      {@const startSeconds = ce.event.start_time ?? 0}
      {@const duration = (ce.event.end_time ?? 0) - startSeconds}
      <div
        class="event-container"
        style:--top={`${ce.row * EVENT_HEIGHT + 8 + ce.row * 8}px`}
        style:--start={startSeconds / SECONDS_IN_HOUR}
        style:--duration={duration / SECONDS_IN_HOUR}
        style:--event-height={`${EVENT_HEIGHT}px`}
      >
        {ce.event.title}
      </div>
    {/each}
  </div>
</div>

<style>
  #calendar-horizontal-view-main {
    --hour-width: 8rem;
    position: relative;
    align-items: flex-start;
    justify-content: flex-start;
    width: 100%;
    height: 100%;
    overflow-y: hidden;

    &::before {
      content: "";
      position: absolute;
      z-index: 0;
      width: calc(23 * var(--hour-width) + 4rem);
      height: 100%;
      background: linear-gradient(90deg, var(--outline-color1) 2px, transparent 1px);
      background-size: var(--hour-width) 100%;
      background-position: calc(2rem + var(--ch)) 100%;
    }

    #hours-container {
      position: sticky;
      top: 0;
      z-index: 1;
      justify-content: flex-start;
      padding: 0.25rem 2rem;
      background-color: var(--color-secondary1);
      border-bottom: 1px solid var(--outline-color2);

      p {
        flex: 0 0 var(--hour-width);
        margin: 0;
        font-size: 0.75rem;
      }
    }

    #timeline {
      position: relative;
      height: 100%;
      width: 100%;
      margin-bottom: 0.5rem;
      overflow-y: auto;
    }

    .event-container {
      position: absolute;
      top: var(--top);
      left: calc(var(--start) * var(--hour-width) + 2rem + var(--ch));
      width: calc(var(--duration) * var(--hour-width));
      height: var(--event-height);
      background-color: red;
    }
  }
</style>