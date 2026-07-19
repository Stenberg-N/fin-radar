<script lang="ts">
  import { onMount } from "svelte";
  import { cubicInOut } from "svelte/easing";
  import { slide } from "svelte/transition";

  import { calendarDays, calendarDate } from "$lib/calendar";
  import { t } from "$lib/i18n";

  let isEventsListVisible = $state<boolean>(true);
  
  onMount(() => {
    calendarDate.set(new Date());
  });

</script>

<div id="calendar-main-container" class="vertical-flex-container">
  <div id="calendar-toolbar" class="primary-toolbar horizontal-flex-container">
    <p>placeholder</p>
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
      <div id="calendar-grid">
        {#each $calendarDays as day (day.date)}
          <div class:disabled-day={!day.enabled}>
            <p>{day.isodate}</p>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .disabled-day > * {
    opacity: 0.5;
  }

  #calendar-main-container,
  #calendar-content,
  #calendar-content > div {
    width: 100%;
    height: 100%;
  }

  #calendar-content #calendar-event-container {
    flex-shrink: 0;
    justify-content: flex-start;
    align-items: flex-start;
    padding: 8px;
    border-right: 1px solid #333;
    background-color: #181818;
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
      background-color: #181818;
      
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

  #calendar-grid {
    > div {
      padding: 6px;
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