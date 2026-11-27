<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import type { CalendarEventWithTag } from "#lib/types.ts";
  import { userPrefs } from "#lib/utils/prefsStore.ts";
  import { HoverTitle } from "#lib/classes/hoverTitle.svelte.ts";
  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    /** The parent container's client width. */
    parentWidth: number;
    calendarEvents: CalendarEventWithTag[];
    /** Used to return the event that was clicked. */
    onSelect: (event: CalendarEventWithTag) => void;
  }

  let {
    parentWidth,
    calendarEvents: rawEvents,
    onSelect,
  }: Props = $props();

  const calendarEvents = $derived.by(() => {
    const rows: CalendarEventWithTag[][] = [];

    for (const event of rawEvents) {
      if (event.event.start_time === null || event.event.end_time === null) continue;

      let isPushed = false;
      const start = event.event.start_time;
      const end = event.event.end_time;

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

  const hover = new HoverTitle<{ content: string }>();
  let hoursElementHeight = $state<number>(0);

  const eventColors = {
    red: {
      bg: "var(--color-highlight1-transparent)",
      outline: "var(--color-highlight1-dimmed)",
      hovered: "var(--color-highlight1-transparent-dimmed)",
      text: "var(--color-white-primary1)",
    },
    dark: {
      bg: "var(--color-secondary3)",
      outline: "var(--outline-color3)",
      hovered: "var(--color-secondary4)",
      text: "var(--color-white-primary1)",
    },
    light: {
      bg: "var(--color-white-primary1)",
      outline: "",
      hovered: "var(--hover-color-transparent-white)",
      text: "black",
    }
  };
  const color = $derived(eventColors[$userPrefs.calendarPrefs.eventColor]);
</script>

<div
  id="calendar-horizontal-view-main"
  class="flex column"
  style:--ch={$userPrefs.mainPrefs.hourFormat === "12" ? "1.5ch" : "0.75ch"}
  style:--content-height={`${(calendarEvents.length * EVENT_HEIGHT + 8) + 8}px`}
  style:--hours-bar-height={`${hoursElementHeight}px`}
  in:fly={{ x: -parentWidth / 2, duration: 400, easing: cubicInOut }}
>
  {#if hover.isHovering}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ centerElement: true, moveTop: -40 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      outline={{ width: 1, color: 'var(--outline-color2)'}}
      borderRadius={8}
    >
      <p id="hover-title-content">{hover.target?.content}</p>
    </ModalWrapper>
  {/if}

  <div bind:clientHeight={hoursElementHeight} id="hours-container" class="flex">
    {#each HOURS as hour, i (i)}
      <p>{hour}</p>
    {/each}
  </div>
  <div id="timeline" class="flex">
    {#each calendarEvents as ce (ce.event.id)}
      {@const startSeconds = ce.event.start_time ?? 0}
      {@const duration = (ce.event.end_time ?? 0) - startSeconds}
      <div
        role="button"
        tabindex="0"
        class="event-container"
        style:--top={`${ce.row * EVENT_HEIGHT + 8 + ce.row * 8}px`}
        style:--start={startSeconds / SECONDS_IN_HOUR}
        style:--duration={duration / SECONDS_IN_HOUR}
        style:--event-height={`${EVENT_HEIGHT}px`}
        style:--bg={color.bg}
        style:--outline={color.outline}
        style:--hovered={color.hovered}
        style:--text={color.text}
        onclick={() => onSelect(ce)}
        onkeydown={(e) => {
          switch (e.key) {
            case 'Enter': onSelect(ce); break;
          }
        }}
      >
        <p
          onmouseenter={(e) => hover.enter(e, { content: ce.event.title })}
          onmouseleave={(e) => hover.leave(e)}
        >{ce.event.title}</p>
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

    &::before {
      content: "";
      position: absolute;
      z-index: 0;
      width: calc(23 * var(--hour-width) + 4rem);
      height: max(100%, calc(var(--hours-bar-height) + var(--content-height)));
      background: linear-gradient(90deg, var(--outline-color1) 2px, transparent 1px);
      background-size: var(--hour-width) 100%;
      background-position: calc(2rem + var(--ch)) 100%;
    }

    #hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
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
      flex-shrink: 0;
      height: var(--content-height);
      width: calc(24 * var(--hour-width) + 4rem);
      margin-bottom: 0.5rem;
      overflow: hidden;
    }

    .event-container {
      position: absolute;
      top: var(--top);
      left: calc(var(--start) * var(--hour-width) + 2rem + var(--ch));
      width: calc(var(--duration) * var(--hour-width));
      height: var(--event-height);
      padding: 0.25rem 0.5rem;
      border-radius: 0.25rem;
      background-color: var(--bg);
      border: 1px solid var(--outline);
      color: var(--text);

      &:hover {
        cursor: pointer;
        background-color: var(--hovered);
      }

      p {
        margin: 0;
        white-space: pre;
        overflow: hidden;
        text-overflow: ellipsis;
      }
    }
  }
</style>